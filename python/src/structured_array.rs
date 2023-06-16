use numpy::Element;
use pyo3::IntoPy;

macro_rules! dtype_base {
    ($($type:ident),+) => {
        paste::paste! {
            #[derive(Debug)]
            enum DtypeBase {
                $(
                    #[allow(dead_code)]
                    [<$type:camel>],
                )+
            }

            impl pyo3::IntoPy<core::ffi::c_int> for DtypeBase {
                fn into_py(self, python: pyo3::Python) -> core::ffi::c_int {
                    match self {
                        $(
                            Self::[<$type:camel>] => $type::get_dtype(python).num(),
                        )+
                    }
                }
            }
        }
    }
}

dtype_base![bool, i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64];

#[derive(Debug)]
struct DtypeField {
    name: &'static str,
    base: DtypeBase,
}

impl DtypeField {
    const fn new(name: &'static str, base: DtypeBase) -> Self {
        Self { name, base }
    }
}

#[derive(Debug)]
struct Dtype<const N: usize>([DtypeField; N]);

impl<const N: usize> pyo3::IntoPy<*mut numpy::npyffi::PyArray_Descr> for Dtype<N> {
    fn into_py(self, python: pyo3::Python) -> *mut numpy::npyffi::PyArray_Descr {
        let dtype_description = unsafe { pyo3::ffi::PyList_New(N as pyo3::ffi::Py_ssize_t) };
        for (index, field) in self.0.into_iter().enumerate() {
            let tuple = unsafe { pyo3::ffi::PyTuple_New(2) };
            assert!(
                unsafe {
                    pyo3::ffi::PyTuple_SetItem(
                        tuple,
                        0 as pyo3::ffi::Py_ssize_t,
                        pyo3::ffi::PyUnicode_FromStringAndSize(
                            field.name.as_ptr() as *const core::ffi::c_char,
                            field.name.len() as pyo3::ffi::Py_ssize_t,
                        ),
                    )
                } == 0,
                "PyTuple_SetItem 0 failed"
            );
            assert!(
                unsafe {
                    pyo3::ffi::PyTuple_SetItem(
                        tuple,
                        1 as pyo3::ffi::Py_ssize_t,
                        numpy::PY_ARRAY_API
                            .PyArray_TypeObjectFromType(python, field.base.into_py(python)),
                    )
                } == 0,
                "PyTuple_SetItem 1 failed"
            );
            assert!(
                unsafe {
                    pyo3::ffi::PyList_SetItem(
                        dtype_description,
                        index as pyo3::ffi::Py_ssize_t,
                        tuple,
                    )
                } == 0,
                "PyList_SetItem failed"
            );
        }
        let mut dtype: *mut numpy::npyffi::PyArray_Descr = std::ptr::null_mut();
        assert!(
            unsafe {
                numpy::PY_ARRAY_API.PyArray_DescrConverter(python, dtype_description, &mut dtype)
            } != 0, // numpy uses 0 for error and 1 for success
            "PyArray_DescrConverter failed"
        );
        dtype
    }
}

pub trait SetCell {
    fn set_cell(self, cell: *mut u8);
}

pub struct StructuredArray<Event>
where
    Event: SetCell,
{
    array: *mut numpy::npyffi::objects::PyArrayObject,
    phantom_data: std::marker::PhantomData<Event>,
}

pub struct StructuredArrayData<Event>
where
    Event: SetCell,
{
    pointer: *mut u8,
    phantom_data: std::marker::PhantomData<Event>,
}

unsafe impl<Event> Send for StructuredArrayData<Event> where Event: SetCell {}

impl<Event> StructuredArrayData<Event>
where
    Event: SetCell,
{
    #[inline]
    pub fn set(&mut self, index: usize, event: Event) {
        unsafe {
            self.pointer
                .offset((index * core::mem::size_of::<Event>()) as isize)
                .copy_from(
                    (&event as *const Event) as *const u8,
                    core::mem::size_of::<Event>(),
                )
        }
    }
}

impl<Event> StructuredArray<Event>
where
    Event: SetCell,
{
    fn new<const N: usize>(
        python: pyo3::Python,
        dtype: Dtype<N>,
        mut length: numpy::npyffi::npy_intp,
    ) -> Self {
        let array = unsafe {
            numpy::PY_ARRAY_API.PyArray_NewFromDescr(
                python,
                numpy::PY_ARRAY_API
                    .get_type_object(python, numpy::npyffi::array::NpyTypes::PyArray_Type),
                dtype.into_py(python),
                1,
                &mut length as *mut numpy::npyffi::npy_intp,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
            ) as *mut numpy::npyffi::objects::PyArrayObject
        };
        assert!(!array.is_null(), "PyArray_NewFromDescr failed");
        Self {
            array,
            phantom_data: std::marker::PhantomData,
        }
    }

    pub unsafe fn data(&mut self, python: pyo3::Python) -> StructuredArrayData<Event> {
        let mut index = 0;
        StructuredArrayData {
            pointer: numpy::PY_ARRAY_API.PyArray_GetPtr(python, self.array, &mut index) as *mut u8,
            phantom_data: std::marker::PhantomData,
        }
    }
}

impl<Event> pyo3::IntoPy<pyo3::PyObject> for StructuredArray<Event>
where
    Event: SetCell,
{
    fn into_py(mut self, python: pyo3::Python) -> pyo3::PyObject {
        let object = unsafe {
            pyo3::PyObject::from_owned_ptr(python, self.array as *mut pyo3::ffi::PyObject)
        };
        self.array = std::ptr::null_mut();
        object
    }
}

impl<Event> Drop for StructuredArray<Event>
where
    Event: SetCell,
{
    fn drop(&mut self) {
        if !self.array.is_null() {
            unsafe { pyo3::ffi::Py_DECREF(self.array as *mut pyo3::ffi::PyObject) };
            self.array = std::ptr::null_mut();
        }
    }
}

const DVS_EVENTS_DTYPE: Dtype<4> = Dtype([
    DtypeField::new("t", DtypeBase::U64),
    DtypeField::new("x", DtypeBase::U16),
    DtypeField::new("y", DtypeBase::U16),
    DtypeField::new("on", DtypeBase::Bool),
]);

impl SetCell for neuromorphic_drivers::types::DvsEvent<u64, u16, u16> {
    #[inline]
    fn set_cell(self, cell: *mut u8) {
        unsafe {
            cell.copy_from(
                (&self as *const Self) as *const u8,
                core::mem::size_of::<Self>(),
            );
        }
    }
}

pub fn dvs_events(
    python: pyo3::Python,
    length: numpy::npyffi::npy_intp,
) -> StructuredArray<neuromorphic_drivers::types::DvsEvent<u64, u16, u16>> {
    StructuredArray::new(python, DVS_EVENTS_DTYPE, length)
}

const TRIGGER_EVENTS_DTYPE: Dtype<3> = Dtype([
    DtypeField::new("t", DtypeBase::U64),
    DtypeField::new("id", DtypeBase::U8),
    DtypeField::new("rising", DtypeBase::Bool),
]);

impl SetCell for neuromorphic_drivers::types::TriggerEvent<u64, u8> {
    #[inline]
    fn set_cell(self, cell: *mut u8) {
        unsafe {
            cell.copy_from(
                (&self as *const Self) as *const u8,
                core::mem::size_of::<Self>(),
            );
        }
    }
}

pub fn trigger_events(
    python: pyo3::Python,
    length: numpy::npyffi::npy_intp,
) -> StructuredArray<neuromorphic_drivers::types::TriggerEvent<u64, u8>> {
    StructuredArray::new(python, TRIGGER_EVENTS_DTYPE, length)
}
