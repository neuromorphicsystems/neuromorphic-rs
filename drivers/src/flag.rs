#[derive(Debug, Clone)]
pub struct Inner<IntoError, IntoWarning>
where
    IntoError: Clone + Send,
    IntoWarning: Clone + Send,
{
    pub error: Option<IntoError>,
    pub warning: Option<IntoWarning>,
}

#[derive(Debug, Clone)]
pub struct Flag<IntoError, IntoWarning>(
    std::sync::Arc<std::sync::Mutex<Inner<IntoError, IntoWarning>>>,
)
where
    IntoError: Clone + Send,
    IntoWarning: Clone + Send;

impl<IntoError, IntoWarning> Flag<IntoError, IntoWarning>
where
    IntoError: Clone + Send,
    IntoWarning: Clone + Send,
{
    pub fn new() -> Self {
        Self(std::sync::Arc::new(std::sync::Mutex::new(Inner {
            error: None,
            warning: None,
        })))
    }

    pub fn store_error_if_not_set<Error>(&self, error: Error)
    where
        Error: Into<IntoError>,
    {
        self.0
            .lock()
            .expect("mutex is not poisoned")
            .error
            .get_or_insert(error.into());
    }

    pub fn store_warning_if_not_set<Warning>(&self, warning: Warning)
    where
        Warning: Into<IntoWarning>,
    {
        self.0
            .lock()
            .expect("mutex is not poisoned")
            .warning
            .get_or_insert(warning.into());
    }

    pub fn load_error(&self) -> Result<(), IntoError> {
        match self.0.lock().expect("mutex is not poisoned").error.take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub fn load_warning(&self) -> Option<IntoWarning> {
        self.0.lock().expect("mutex is not poisoned").warning.take()
    }
}

impl<IntoError, IntoWarning> Default for Flag<IntoError, IntoWarning>
where
    IntoError: Clone + Send,
    IntoWarning: Clone + Send,
{
    fn default() -> Self {
        Self(std::sync::Arc::new(std::sync::Mutex::new(Inner {
            error: None,
            warning: None,
        })))
    }
}
