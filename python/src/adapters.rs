use crate::structured_array;
use pyo3::IntoPy;

pub trait Adapter {
    fn current_t(&self) -> u64;

    fn consume(&mut self, slice: &[u8]);

    fn slice_to_dict(
        &mut self,
        python: pyo3::Python,
        slice: &[u8],
    ) -> pyo3::PyResult<pyo3::PyObject>;
}

impl Adapter for neuromorphic_drivers_rs::adapters::evt3::Adapter {
    fn current_t(&self) -> u64 {
        self.current_t()
    }

    fn consume(&mut self, slice: &[u8]) {
        self.consume(slice);
    }

    fn slice_to_dict(
        &mut self,
        python: pyo3::Python,
        slice: &[u8],
    ) -> pyo3::PyResult<pyo3::PyObject> {
        let events_lengths = self.events_lengths(slice);
        let mut dvs_events =
            structured_array::dvs_events(python, events_lengths.0 as numpy::npyffi::npy_intp);
        let mut dvs_events_data = unsafe { dvs_events.data(python) };
        let mut dvs_events_index = 0;
        let mut trigger_events =
            structured_array::trigger_events(python, events_lengths.1 as numpy::npyffi::npy_intp);
        let mut trigger_events_data = unsafe { trigger_events.data(python) };
        let mut trigger_events_index = 0;
        python.allow_threads(|| {
            self.convert(
                slice,
                |dvs_event| {
                    dvs_events_data.set(dvs_events_index, dvs_event);
                    dvs_events_index += 1;
                },
                |trigger_event| {
                    trigger_events_data.set(trigger_events_index, trigger_event);
                    trigger_events_index += 1;
                },
            );
        });
        let dict = pyo3::types::PyDict::new(python);
        if events_lengths.0 > 0 {
            dict.set_item("dvs_events", dvs_events.into_py(python))?;
        }
        if events_lengths.1 > 0 {
            dict.set_item("trigger_events", trigger_events.into_py(python))?;
        }
        Ok(dict.into())
    }
}

impl Adapter for neuromorphic_drivers_rs::Adapter {
    fn current_t(&self) -> u64 {
        match self {
            neuromorphic_drivers::Adapter::Evt3(adapter) => adapter.current_t(),
        }
    }

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
