pub struct Bytes {
    inner: *mut pyo3::ffi::PyObject,
}

impl Bytes {
    pub fn new() -> Self {
        Self {
            inner: std::ptr::null_mut(),
        }
    }

    pub fn extend_from_slice(&mut self, _python: pyo3::Python, slice: &[u8]) {
        if self.inner.is_null() {
            self.inner = unsafe {
                pyo3::ffi::PyBytes_FromStringAndSize(
                    slice.as_ptr() as *const std::os::raw::c_char,
                    slice.len() as pyo3::ffi::Py_ssize_t,
                )
            };
        } else {
            unsafe {
                let length = pyo3::ffi::PyBytes_Size(self.inner);
                assert!(
                    pyo3::ffi::_PyBytes_Resize(
                        &mut self.inner as *mut *mut pyo3::ffi::PyObject,
                        length + slice.len() as isize,
                    ) == 0,
                    "memory error"
                );
                std::ptr::copy_nonoverlapping(
                    slice.as_ptr(),
                    (pyo3::ffi::PyBytes_AsString(self.inner) as *mut u8).offset(length),
                    slice.len(),
                );
            }
        }
    }

    pub fn take<'p>(&mut self, python: pyo3::Python<'p>) -> Option<&'p pyo3::types::PyBytes> {
        if self.inner.is_null() {
            None
        } else {
            let py_bytes = Some(unsafe { python.from_owned_ptr(self.inner) });
            self.inner = std::ptr::null_mut();
            py_bytes
        }
    }
}

impl Drop for Bytes {
    fn drop(&mut self) {
        unsafe {
            pyo3::ffi::Py_XDECREF(self.inner);
        }
        self.inner = std::ptr::null_mut();
    }
}
