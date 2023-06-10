use crate::structured_array;
use pyo3::IntoPy;

pub trait Adapter {
    fn consume(&mut self, slice: &[u8]);

    fn slice_to_dict(
        &mut self,
        python: pyo3::Python,
        slice: &[u8],
    ) -> pyo3::PyResult<pyo3::PyObject>;
}

impl Adapter for neuromorphic_drivers_rs::adapters::evt3::Adapter {
    fn consume(&mut self, slice: &[u8]) {
        self.consume(slice);
    }

    fn slice_to_dict(
        &mut self,
        python: pyo3::Python,
        slice: &[u8],
    ) -> pyo3::PyResult<pyo3::PyObject> {
        let mut dvs_events = structured_array::dvs_events(
            python,
            neuromorphic_drivers_rs::adapters::evt3::Adapter::estimate_dvs_events_length(slice)
                as numpy::npyffi::npy_intp,
        );
        let mut trigger_events = structured_array::trigger_events(python, 1);
        self.convert(
            slice,
            |dvs_event| {
                dvs_events.push(python, dvs_event);
            },
            |trigger_event| {
                trigger_events.push(python, trigger_event);
            },
        );
        let dict = pyo3::types::PyDict::new(python);
        if !dvs_events.is_empty() {
            dict.set_item("dvs_events", dvs_events.into_py(python))?;
        }
        if !trigger_events.is_empty() {
            dict.set_item("trigger_events", trigger_events.into_py(python))?;
        }
        Ok(dict.into())
    }
}

impl Adapter for neuromorphic_drivers_rs::Adapter {
    fn consume(&mut self, slice: &[u8]) {
        match self {
            neuromorphic_drivers::Adapter::Evt3(adapter) => adapter.consume(slice),
        }
    }

    fn slice_to_dict(
        &mut self,
        python: pyo3::Python,
        slice: &[u8],
    ) -> pyo3::PyResult<pyo3::PyObject> {
        match self {
            neuromorphic_drivers::Adapter::Evt3(adapter) => adapter.slice_to_dict(python, slice),
        }
    }
}
