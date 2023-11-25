mod adapters;
mod bytes;
mod structured_array;
extern crate neuromorphic_drivers as neuromorphic_drivers_rs;
use pyo3::IntoPy;
use std::ops::DerefMut;

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
    device: Option<neuromorphic_drivers_rs::Device>,
    adapter: Option<std::cell::RefCell<adapters::Adapter>>,
    iterator_timeout: Option<std::time::Duration>,
    iterator_maximum_raw_packets: usize,
    error_flag: neuromorphic_drivers_rs::error::Flag<neuromorphic_drivers_rs::Error>,
}

// unsafe workaround until auto traits are stabilized
// see https://docs.rs/pyo3/0.19.0/pyo3/marker/index.html
struct DeviceReference<'a>(pub &'a neuromorphic_drivers_rs::Device);
unsafe impl Send for DeviceReference<'_> {}
unsafe impl Sync for DeviceReference<'_> {}
enum Buffer<'a> {
    Adapter(&'a mut adapters::Adapter),
    Bytes(bytes::Bytes),
}
unsafe impl Send for Buffer<'_> {}

struct Status {
    instant: std::time::Instant,
    read_range: (usize, usize),
    write_range: (usize, usize),
    ring_length: usize,
    current_t: Option<u64>,
}

#[pyo3::pymethods]
impl Device {
    #[new]
    fn new(
        raw: bool,
        iterator_maximum_raw_packets: usize,
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
                    neuromorphic_drivers_rs::UsbConfiguration::deserialize_bincode(
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
            Some(std::cell::RefCell::new(device.adapter().into()))
        };
        Ok(Self {
            device: Some(device),
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
            iterator_maximum_raw_packets,
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
        let start = std::time::Instant::now();
        let error_flag = slf.error_flag.clone();
        let iterator_timeout = slf.iterator_timeout;
        let iterator_maximum_raw_packets = slf.iterator_maximum_raw_packets;
        let device = DeviceReference(slf.device.as_ref().ok_or(
            pyo3::exceptions::PyRuntimeError::new_err("__next__ called after __exit__"),
        )?);
        let mut adapter_reference = match slf.adapter.as_ref() {
            Some(adapter) => Some(adapter.try_borrow_mut().map_err(|_| {
                pyo3::exceptions::PyRuntimeError::new_err(
                    "__next__ called while device is used by a different thread",
                )
            })?),
            None => None,
        };
        let mut buffer = adapter_reference.as_mut().map_or_else(
            || Buffer::Bytes(bytes::Bytes::new()),
            |adapter| Buffer::Adapter(adapter.deref_mut()),
        );
        python.allow_threads(|| -> pyo3::PyResult<Option<pyo3::PyObject>> {
            let mut status = None;
            let mut raw_packets = 0;
            let mut available_raw_packets = None;
            let buffer_timeout = iterator_timeout.unwrap_or(std::time::Duration::from_millis(100));
            loop {
                if let Some(error) = error_flag.load() {
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "{error:?}"
                    )));
                }
                if let Some(buffer_view) = device.0.next_with_timeout(&buffer_timeout) {
                    let current_status = status.get_or_insert(Status {
                        instant: buffer_view.instant,
                        read_range: (buffer_view.read, buffer_view.read + 1),
                        write_range: buffer_view.write_range,
                        ring_length: buffer_view.ring_length,
                        current_t: None,
                    });
                    current_status.read_range = (current_status.read_range.0, buffer_view.read + 1);
                    current_status.write_range = buffer_view.write_range;
                    let _ = available_raw_packets.get_or_insert_with(|| {
                        if iterator_maximum_raw_packets == 0 {
                            buffer_view.backlog() + 1
                        } else {
                            iterator_maximum_raw_packets.min(buffer_view.backlog() + 1)
                        }
                    });
                    raw_packets += 1;
                    match &mut buffer {
                        Buffer::Adapter(adapter) => {
                            adapter.push(buffer_view.slice);
                            current_status.current_t = Some(adapter.current_t());
                        }
                        Buffer::Bytes(bytes) => pyo3::Python::with_gil(|python| {
                            bytes.extend_from_slice(python, buffer_view.slice);
                        }),
                    }
                }
                if iterator_timeout.map_or_else(|| false, |timeout| start.elapsed() >= timeout)
                    || available_raw_packets.map_or_else(
                        || false,
                        |available_raw_packets| raw_packets >= available_raw_packets,
                    )
                {
                    return pyo3::Python::with_gil(|python| {
                        let packet = match &mut buffer {
                            Buffer::Adapter(adapter) => Some(adapter.take_into_dict(python)?),
                            Buffer::Bytes(bytes) => pyo3::Python::with_gil(|python| {
                                bytes.take(python).map(|bytes| bytes.into())
                            }),
                        };
                        let duration_since_epoch = std::time::SystemTime::now()
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .unwrap_or(std::time::Duration::from_secs(0));
                        Ok((
                            duration_since_epoch.as_secs_f64(),
                            status.map(|status| {
                                (
                                    (status.instant.elapsed() + duration_since_epoch).as_secs_f64(),
                                    status.read_range,
                                    status.write_range,
                                    status.ring_length,
                                    status.current_t,
                                )
                            }),
                            packet,
                        )
                            .into_py(python))
                    })
                    .map(Some);
                }
            }
        })
    }

    fn clear_backlog(
        slf: pyo3::PyRef<Self>,
        python: pyo3::Python,
        until: usize,
    ) -> pyo3::PyResult<()> {
        let error_flag = slf.error_flag.clone();
        let device = DeviceReference(slf.device.as_ref().ok_or(
            pyo3::exceptions::PyRuntimeError::new_err("__next__ called after __exit__"),
        )?);
        let mut adapter_reference = match slf.adapter.as_ref() {
            Some(adapter) => Some(adapter.try_borrow_mut().map_err(|_| {
                pyo3::exceptions::PyRuntimeError::new_err(
                    "__next__ called while device is used by a different thread",
                )
            })?),
            None => None,
        };
        let mut buffer = adapter_reference.as_mut().map_or_else(
            || Buffer::Bytes(bytes::Bytes::new()),
            |adapter| Buffer::Adapter(adapter.deref_mut()),
        );
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
                if let Buffer::Adapter(adapter) = &mut buffer {
                    adapter.consume(buffer_view.slice);
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
            .serial())
    }

    fn chip_firmware_configuration(
        slf: pyo3::PyRef<Self>,
        python: pyo3::Python,
    ) -> pyo3::PyResult<pyo3::PyObject> {
        Ok(pyo3::types::PyBytes::new(
            python,
            &slf.device
                .as_ref()
                .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
                    "serial called after __exit__",
                ))?
                .chip_firmware_configuration()
                .serialize_bincode()
                .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(format!("{error}")))?,
        )
        .into_py(python))
    }

    fn speed(slf: pyo3::PyRef<Self>) -> pyo3::PyResult<String> {
        Ok(slf
            .device
            .as_ref()
            .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
                "speed called after __exit__",
            ))?
            .speed()
            .to_string())
    }

    fn update_configuration(
        slf: pyo3::PyRef<Self>,
        type_and_configuration: (&str, &[u8]),
    ) -> pyo3::PyResult<()> {
        let configuration = neuromorphic_drivers_rs::Configuration::deserialize_bincode(
            type_and_configuration
                .0
                .parse()
                .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(format!("{error}")))?,
            type_and_configuration.1,
        )
        .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(format!("{error}")))?;
        slf.device
            .as_ref()
            .ok_or(pyo3::exceptions::PyRuntimeError::new_err(
                "__next__ called after __exit__",
            ))?
            .update_configuration(configuration)
            .map_err(|_| {
                pyo3::exceptions::PyRuntimeError::new_err(
                    "update_configuration called while device is used by a different thread",
                )
            })
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
