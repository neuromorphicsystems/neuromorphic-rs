mod adapters;
mod structured_array;
extern crate neuromorphic_drivers as neuromorphic_drivers_rs;
use adapters::Adapter;
use pyo3::IntoPy;

#[pyo3::pyfunction]
fn list_devices() -> pyo3::PyResult<Vec<(String, String, Option<String>, Option<String>)>> {
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

    fn __next__(slf: pyo3::PyRef<Self>) -> pyo3::PyResult<Option<pyo3::PyObject>> {
        let error_flag = slf.error_flag.clone();
        let mut device = slf
            .device
            .as_ref()
            .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
                "__next__ called after __exit__",
            ))?
            .borrow_mut();
        match slf.iterator_timeout.as_ref() {
            Some(iterator_timeout) => {
                if let Some(error) = error_flag.load() {
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "{error:?}"
                    )));
                }
                device
                    .next_with_timeout(iterator_timeout)
                    .map_or_else(
                        || {
                            pyo3::Python::with_gil(|python| match slf.adapter.as_ref() {
                                Some(_) => Ok((
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                        .unwrap_or(std::time::Duration::from_secs(0))
                                        .as_secs_f64(),
                                    None::<(f64, usize, (usize, usize), usize)>,
                                    pyo3::types::PyDict::new(python),
                                )
                                    .into_py(python)),
                                None => Ok((
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                        .unwrap_or(std::time::Duration::from_secs(0))
                                        .as_secs_f64(),
                                    None::<(f64, usize, (usize, usize), usize)>,
                                    None::<&pyo3::types::PyBytes>,
                                )
                                    .into_py(python)),
                            })
                        },
                        |buffer_view| {
                            pyo3::Python::with_gil(|python| -> pyo3::PyResult<pyo3::PyObject> {
                                match slf.adapter.as_ref() {
                                    Some(adapter) => Ok((
                                        (
                                            std::time::SystemTime::now()
                                                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                                .unwrap_or(std::time::Duration::from_secs(0))
                                                .as_secs_f64(),
                                            Some((
                                                buffer_view
                                                    .system_time
                                                    .duration_since(
                                                        std::time::SystemTime::UNIX_EPOCH,
                                                    )
                                                    .unwrap_or(std::time::Duration::from_secs(0))
                                                    .as_secs_f64(),
                                                buffer_view.read,
                                                buffer_view.write_range,
                                                buffer_view.ring_length,
                                            )),
                                        ),
                                        adapter
                                            .borrow_mut()
                                            .slice_to_dict(python, buffer_view.slice)?,
                                    )
                                        .into_py(python)),
                                    None => Ok((
                                        (
                                            std::time::SystemTime::now()
                                                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                                .unwrap_or(std::time::Duration::from_secs(0))
                                                .as_secs_f64(),
                                            Some((
                                                buffer_view
                                                    .system_time
                                                    .duration_since(
                                                        std::time::SystemTime::UNIX_EPOCH,
                                                    )
                                                    .unwrap_or(std::time::Duration::from_secs(0))
                                                    .as_secs_f64(),
                                                buffer_view.read,
                                                buffer_view.write_range,
                                                buffer_view.ring_length,
                                            )),
                                        ),
                                        pyo3::types::PyBytes::new(python, buffer_view.slice),
                                    )
                                        .into_py(python)),
                                }
                            })
                        },
                    )
                    .map(|object| Some(object))
            }
            None => loop {
                if let Some(error) = error_flag.load() {
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "{error:?}"
                    )));
                }
                if let Some(buffer_view) =
                    device.next_with_timeout(&std::time::Duration::from_millis(100))
                {
                    return pyo3::Python::with_gil(|python| match slf.adapter.as_ref() {
                        Some(adapter) => Ok((
                            (
                                std::time::SystemTime::now()
                                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                    .unwrap_or(std::time::Duration::from_secs(0))
                                    .as_secs_f64(),
                                Some((
                                    buffer_view
                                        .system_time
                                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                        .unwrap_or(std::time::Duration::from_secs(0))
                                        .as_secs_f64(),
                                    buffer_view.read,
                                    buffer_view.write_range,
                                    buffer_view.ring_length,
                                )),
                            ),
                            adapter
                                .borrow_mut()
                                .slice_to_dict(python, buffer_view.slice)?,
                        )
                            .into_py(python)),
                        None => Ok((
                            (
                                std::time::SystemTime::now()
                                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                    .unwrap_or(std::time::Duration::from_secs(0))
                                    .as_secs_f64(),
                                Some((
                                    buffer_view
                                        .system_time
                                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                        .unwrap_or(std::time::Duration::from_secs(0))
                                        .as_secs_f64(),
                                    buffer_view.read,
                                    buffer_view.write_range,
                                    buffer_view.ring_length,
                                )),
                            ),
                            pyo3::types::PyBytes::new(python, buffer_view.slice),
                        )
                            .into_py(python)),
                    })
                    .map(|object| Some(object));
                }
            },
        }
    }

    fn clear_backlog(slf: pyo3::PyRef<Self>, until: usize) -> pyo3::PyResult<()> {
        let error_flag = slf.error_flag.clone();
        let mut device = slf
            .device
            .as_ref()
            .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
                "__next__ called after __exit__",
            ))?
            .borrow_mut();
        loop {
            if let Some(error) = error_flag.load() {
                return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "{error:?}"
                )));
            }
            if let Some(buffer_view) =
                device.next_with_timeout(&std::time::Duration::from_millis(0))
            {
                if buffer_view.backlog() < until {
                    return Ok(());
                }
                match slf.adapter.as_ref() {
                    Some(adapter) => {
                        adapter.borrow_mut().consume(buffer_view.slice);
                    }
                    None => (),
                }
            } else {
                return Ok(());
            }
        }
    }

    fn name(slf: pyo3::PyRef<Self>) -> pyo3::PyResult<String> {
        Ok(slf
            .device
            .as_ref()
            .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
                "name called after __exit__",
            ))?
            .borrow()
            .name()
            .to_owned())
    }

    fn serial(slf: pyo3::PyRef<Self>) -> pyo3::PyResult<String> {
        Ok(slf
            .device
            .as_ref()
            .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
                "name called after __exit__",
            ))?
            .borrow()
            .serial())
    }

    fn speed(slf: pyo3::PyRef<Self>) -> pyo3::PyResult<String> {
        Ok(slf
            .device
            .as_ref()
            .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
                "name called after __exit__",
            ))?
            .borrow()
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
