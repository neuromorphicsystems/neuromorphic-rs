use neuromorphic_drivers::types::SliceView;
use numpy::IntoPyArray;

use crate::structured_array;
use pyo3::IntoPy;

pub enum Adapter {
    Evt3 {
        inner: neuromorphic_drivers_rs::adapters::evt3::Adapter,
        dvs_events: Vec<u8>,
        trigger_events: Vec<u8>,
    },
}

impl Adapter {
    pub fn current_t(&self) -> u64 {
        match self {
            Adapter::Evt3 { inner, .. } => inner.current_t(),
        }
    }

    pub fn consume(&mut self, slice: &[u8]) {
        match self {
            Adapter::Evt3 { inner, .. } => inner.consume(slice),
        }
    }

    pub fn push(&mut self, slice: &[u8]) {
        match self {
            Adapter::Evt3 {
                inner,
                dvs_events,
                trigger_events,
            } => {
                let events_lengths = inner.events_lengths(slice);
                dvs_events.reserve_exact(events_lengths.dvs);
                trigger_events.reserve_exact(events_lengths.trigger);
                inner.convert(
                    slice,
                    |dvs_event| {
                        dvs_events.extend_from_slice(dvs_event.as_bytes());
                    },
                    |trigger_event| {
                        trigger_events.extend_from_slice(trigger_event.as_bytes());
                    },
                );
            }
        }
    }

    pub fn take_into_dict(&mut self, python: pyo3::Python) -> pyo3::PyResult<pyo3::PyObject> {
        match self {
            Adapter::Evt3 {
                inner: _,
                dvs_events,
                trigger_events,
            } => {
                let dict = pyo3::types::PyDict::new(python);
                if !dvs_events.is_empty() {
                    let dvs_events_array = {
                        let mut taken_dvs_events = Vec::new();
                        std::mem::swap(dvs_events, &mut taken_dvs_events);
                        taken_dvs_events.into_pyarray(python)
                    };
                    let description = structured_array::DVS_EVENTS_DTYPE.into_py(python);
                    let dvs_events_array_pointer = dvs_events_array.as_array_ptr();
                    unsafe {
                        *(*dvs_events_array_pointer).dimensions /=
                            structured_array::DVS_EVENTS_DTYPE.size() as isize;
                        *(*dvs_events_array_pointer).strides =
                            structured_array::DVS_EVENTS_DTYPE.size() as isize;
                        let previous_description = (*dvs_events_array_pointer).descr;
                        (*dvs_events_array_pointer).descr = description;
                        pyo3::ffi::Py_DECREF(previous_description as *mut pyo3::ffi::PyObject);
                    }
                    dict.set_item("dvs_events", dvs_events_array)?;
                }
                if !trigger_events.is_empty() {
                    let trigger_events_array = {
                        let mut taken_trigger_events = Vec::new();
                        std::mem::swap(trigger_events, &mut taken_trigger_events);
                        taken_trigger_events.into_pyarray(python)
                    };
                    let description = structured_array::TRIGGER_EVENTS_DTYPE.into_py(python);
                    let trigger_events_array_pointer = trigger_events_array.as_array_ptr();
                    unsafe {
                        *(*trigger_events_array_pointer).dimensions /=
                            structured_array::TRIGGER_EVENTS_DTYPE.size() as isize;
                        *(*trigger_events_array_pointer).strides =
                            structured_array::TRIGGER_EVENTS_DTYPE.size() as isize;
                        let previous_description = (*trigger_events_array_pointer).descr;
                        (*trigger_events_array_pointer).descr = description;
                        pyo3::ffi::Py_DECREF(previous_description as *mut pyo3::ffi::PyObject);
                    }
                    dict.set_item("trigger_events", trigger_events_array)?;
                }
                Ok(dict.into())
            }
        }
    }
}

impl From<neuromorphic_drivers::Adapter> for Adapter {
    fn from(adapter: neuromorphic_drivers::Adapter) -> Self {
        match adapter {
            neuromorphic_drivers::Adapter::Evt3(inner) => Adapter::Evt3 {
                inner,
                dvs_events: Vec::new(),
                trigger_events: Vec::new(),
            },
        }
    }
}
