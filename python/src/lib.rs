mod adapters;
mod structured_array;
extern crate neuromorphic_drivers as neuromorphic_drivers_rs;
use std::ops::DerefMut;

use adapters::Adapter;
use pyo3::IntoPy;

type ListedDevice = (String, String, Option<String>, Option<String>);

#[pyo3::pyfunction]
fn list_devices() -> pyo3::PyResult<Vec<ListedDevice>> {
    Ok(neuromorphic_drivers_rs::list_devices()
        .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(format!("{error}")))?
        .into_iter()
        .map(|listed_device| {
            let (serial, error) = match listed_device.serial {
                Ok(serial) => (Some(serial), None),
                Err(error) => (None, Some(format!("{error}"))),
            };
            (
                listed_device.device_type.name().to_owned(),
                listed_device.speed.to_string(),
                serial,
                error,
            )
        })
        .collect())
}

#[pyo3::pyclass(subclass)]
struct Device {
    device: Option<std::cell::RefCell<neuromorphic_drivers_rs::Device>>,
    adapter: Option<std::cell::RefCell<neuromorphic_drivers_rs::Adapter>>,
    iterator_timeout: Option<std::time::Duration>,
    error_flag: neuromorphic_drivers_rs::error::Flag<neuromorphic_drivers_rs::Error>,
}

// unsafe workaround until auto traits are stabilized
// see https://docs.rs/pyo3/0.19.0/pyo3/marker/index.html
struct DeviceReference<'a>(pub &'a mut neuromorphic_drivers_rs::Device);
unsafe impl Send for DeviceReference<'_> {}
struct AdapterReference<'a>(pub &'a mut neuromorphic_drivers_rs::Adapter);
unsafe impl Send for AdapterReference<'_> {}

fn next_output(
    python: pyo3::Python,
    buffer_view: Option<neuromorphic_drivers_rs::usb::BufferView<'_>>,
    current_t: Option<u64>,
    packet: Option<pyo3::PyObject>,
) -> pyo3::PyObject {
    (
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs_f64(),
        buffer_view.map(|buffer_view| {
            (
                buffer_view
                    .system_time
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .unwrap_or(std::time::Duration::from_secs(0))
                    .as_secs_f64(),
                buffer_view.read,
                buffer_view.write_range,
                buffer_view.ring_length,
                current_t,
            )
        }),
        packet,
    )
        .into_py(python)
}

#[pyo3::pymethods]
impl Device {
    #[new]
    fn new(
        raw: bool,
        type_and_configuration: Option<(&str, &[u8])>,
        serial: Option<&str>,
        usb_configuration: Option<&[u8]>,
        iterator_timeout: Option<f64>,
    ) -> pyo3::PyResult<Self> {
        let error_flag = neuromorphic_drivers_rs::error::Flag::new();
        let event_loop = std::sync::Arc::new(
            neuromorphic_drivers_rs::usb::EventLoop::new(
                std::time::Duration::from_millis(100),
                error_flag.clone(),
            )
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(format!("{error}")))?,
        );
        let device = neuromorphic_drivers_rs::open(
            serial,
            match type_and_configuration {
                Some((device_type, configuration)) => Some(
                    neuromorphic_drivers_rs::Configuration::deserialize_bincode(
                        device_type.parse().map_err(|error| {
                            pyo3::exceptions::PyRuntimeError::new_err(format!("{error}"))
                        })?,
                        configuration,
                    )
                    .map_err(|error| {
                        pyo3::exceptions::PyRuntimeError::new_err(format!("{error}"))
                    })?,
                ),
                None => None,
            },
            if let Some(usb_configuration) = usb_configuration {
                Some(
                    bincode::deserialize::<neuromorphic_drivers_rs::UsbConfiguration>(
                        usb_configuration,
                    )
                    .map_err(|error| {
                        pyo3::exceptions::PyRuntimeError::new_err(format!("{error}"))
                    })?,
                )
            } else {
                None
            },
            event_loop,
            error_flag.clone(),
        )
        .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(format!("{error}")))?;
        let adapter = if raw {
            None
        } else {
            Some(std::cell::RefCell::new(device.adapter()))
        };
        Ok(Self {
            device: Some(std::cell::RefCell::new(device)),
            adapter,
            iterator_timeout: match iterator_timeout {
                Some(seconds) => {
                    if seconds < 0.0 {
                        return Err(pyo3::exceptions::PyValueError::new_err(
                            "iterator_timeout must larger than or equal to 0",
                        ));
                    } else {
                        Some(std::time::Duration::from_secs_f64(seconds))
                    }
                }
                None => None,
            },
            error_flag,
        })
    }

    fn __enter__(slf: pyo3::Py<Self>) -> pyo3::Py<Self> {
        slf
    }

    fn __exit__(
        &mut self,
        _exception_type: Option<&pyo3::types::PyType>,
        _value: Option<&pyo3::types::PyAny>,
        _traceback: Option<&pyo3::types::PyAny>,
    ) {
        self.device = None;
    }

    fn __iter__(slf: pyo3::Py<Self>) -> pyo3::Py<Self> {
        slf
    }

    fn __next__(
        slf: pyo3::PyRef<Self>,
        python: pyo3::Python,
    ) -> pyo3::PyResult<Option<pyo3::PyObject>> {
        let error_flag = slf.error_flag.clone();
        let iterator_timeout = slf.iterator_timeout;
        let mut device_reference = slf
            .device
            .as_ref()
            .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
                "__next__ called after __exit__",
            ))?
            .try_borrow_mut()
            .map_err(|_| {
                pyo3::exceptions::PyRuntimeError::new_err(
                    "__next__ called while device is used by a different thread",
                )
            })?;
        let device = DeviceReference(device_reference.deref_mut());
        let mut adapter_reference = match slf.adapter.as_ref() {
            Some(adapter) => Some(adapter.try_borrow_mut().map_err(|_| {
                pyo3::exceptions::PyRuntimeError::new_err(
                    "__next__ called while device is used by a different thread",
                )
            })?),
            None => None,
        };
        let mut adapter = adapter_reference
            .as_mut()
            .map(|adapter| AdapterReference(adapter.deref_mut()));
        python.allow_threads(|| match iterator_timeout {
            Some(iterator_timeout) => {
                if let Some(error) = error_flag.load() {
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "{error:?}"
                    )));
                }
                (match device.0.next_with_timeout(&iterator_timeout) {
                    Some(buffer_view) => {
                        pyo3::Python::with_gil(|python| -> pyo3::PyResult<pyo3::PyObject> {
                            match &mut adapter {
                                Some(adapter) => {
                                    let packet =
                                        adapter.0.slice_to_dict(python, buffer_view.slice)?;
                                    Ok(next_output(
                                        python,
                                        Some(buffer_view),
                                        Some(adapter.0.current_t()),
                                        Some(packet),
                                    ))
                                }
                                None => {
                                    let packet =
                                        pyo3::types::PyBytes::new(python, buffer_view.slice).into();
                                    Ok(next_output(python, Some(buffer_view), None, Some(packet)))
                                }
                            }
                        })
                    }
                    None => pyo3::Python::with_gil(|python| match &mut adapter {
                        Some(_) => Ok(next_output(
                            python,
                            None,
                            None,
                            Some(pyo3::types::PyDict::new(python).into()),
                        )),
                        None => Ok(next_output(python, None, None, None)),
                    }),
                })
                .map(Some)
            }
            None => loop {
                if let Some(error) = error_flag.load() {
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "{error:?}"
                    )));
                }
                if let Some(buffer_view) = device
                    .0
                    .next_with_timeout(&std::time::Duration::from_millis(100))
                {
                    return pyo3::Python::with_gil(|python| match adapter {
                        Some(adapter) => {
                            let packet = adapter.0.slice_to_dict(python, buffer_view.slice)?;
                            Ok(next_output(
                                python,
                                Some(buffer_view),
                                Some(adapter.0.current_t()),
                                Some(packet),
                            ))
                        }
                        None => {
                            let packet =
                                pyo3::types::PyBytes::new(python, buffer_view.slice).into();
                            Ok(next_output(python, Some(buffer_view), None, Some(packet)))
                        }
                    })
                    .map(Some);
                }
            },
        })
    }

    fn clear_backlog(
        slf: pyo3::PyRef<Self>,
        python: pyo3::Python,
        until: usize,
    ) -> pyo3::PyResult<()> {
        let error_flag = slf.error_flag.clone();
        let mut device_reference = slf
            .device
            .as_ref()
            .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
                "__next__ called after __exit__",
            ))?
            .try_borrow_mut()
            .map_err(|_| {
                pyo3::exceptions::PyRuntimeError::new_err(
                    "__next__ called while device is used by a different thread",
                )
            })?;
        let device = DeviceReference(device_reference.deref_mut());
        let mut adapter_reference = match slf.adapter.as_ref() {
            Some(adapter) => Some(adapter.try_borrow_mut().map_err(|_| {
                pyo3::exceptions::PyRuntimeError::new_err(
                    "__next__ called while device is used by a different thread",
                )
            })?),
            None => None,
        };
        let mut adapter = adapter_reference
            .as_mut()
            .map(|adapter| AdapterReference(adapter.deref_mut()));
        python.allow_threads(|| loop {
            if let Some(error) = error_flag.load() {
                return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "{error:?}"
                )));
            }
            if let Some(buffer_view) = device
                .0
                .next_with_timeout(&std::time::Duration::from_millis(0))
            {
                if buffer_view.backlog() < until {
                    return Ok(());
                }
                match &mut adapter {
                    Some(adapter) => {
                        adapter.0.consume(buffer_view.slice);
                    }
                    None => (),
                }
            } else {
                return Ok(());
            }
        })
    }

    fn name(slf: pyo3::PyRef<Self>) -> pyo3::PyResult<String> {
        Ok(slf
            .device
            .as_ref()
            .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
                "name called after __exit__",
            ))?
            .try_borrow()
            .map_err(|_| {
                pyo3::exceptions::PyRuntimeError::new_err(
                    "name called while device is used by a different thread",
                )
            })?
            .name()
            .to_owned())
    }

    fn serial(slf: pyo3::PyRef<Self>) -> pyo3::PyResult<String> {
        Ok(slf
            .device
            .as_ref()
            .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
                "serial called after __exit__",
            ))?
            .try_borrow()
            .map_err(|_| {
                pyo3::exceptions::PyRuntimeError::new_err(
                    "serial called while device is used by a different thread",
                )
            })?
            .serial())
    }

    fn speed(slf: pyo3::PyRef<Self>) -> pyo3::PyResult<String> {
        Ok(slf
            .device
            .as_ref()
            .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
                "speed called after __exit__",
            ))?
            .try_borrow()
            .map_err(|_| {
                pyo3::exceptions::PyRuntimeError::new_err(
                    "speed called while device is used by a different thread",
                )
            })?
            .speed()
            .to_string())
    }
}

#[pyo3::pymodule]
fn neuromorphic_drivers(
    _py: pyo3::Python<'_>,
    module: &pyo3::types::PyModule,
) -> pyo3::PyResult<()> {
    module.add_class::<Device>()?;
    module.add_function(pyo3::wrap_pyfunction!(list_devices, module)?)?;
    Ok(())
}
