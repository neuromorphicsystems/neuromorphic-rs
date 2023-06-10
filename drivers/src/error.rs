#[derive(Debug, Clone)]
pub struct Flag<IntoError>(std::sync::Arc<std::sync::Mutex<Option<IntoError>>>)
where
    IntoError: Clone + Send;

impl<IntoError> Flag<IntoError>
where
    IntoError: Clone + Send,
{
    pub fn new() -> Self {
        Self(std::sync::Arc::new(std::sync::Mutex::new(None)))
    }

    pub fn store_if_not_set<Error>(&self, error: Error)
    where
        Error: Into<IntoError>,
    {
        // unwrap: mutex is not poisoned
        self.0.lock().unwrap().get_or_insert(error.into());
    }

    pub fn load(&self) -> Option<IntoError> {
        // unwrap: mutex is not poisoned
        self.0.lock().unwrap().clone()
    }
}
