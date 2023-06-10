pub struct Flagged<Configuration> {
    configuration: Configuration,
    updated: bool,
}

pub struct Updater<Configuration> {
    flagged_configuration_and_condition:
        std::sync::Arc<(std::sync::Mutex<Flagged<Configuration>>, std::sync::Condvar)>,
    thread: Option<std::thread::JoinHandle<()>>,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl<Configuration: Clone + Send + 'static> Updater<Configuration> {
    pub fn new<ContextType, Update>(
        initial_configuration: Configuration,
        context: ContextType,
        update: Update,
    ) -> Self
    where
        ContextType: Send + 'static,
        Update:
            Fn(ContextType, &Configuration, &Configuration) -> ContextType + Send + 'static,
    {
        let previous_configuration = initial_configuration.clone();
        let flagged_configuration_and_condition = std::sync::Arc::new((
            std::sync::Mutex::new(Flagged {
                configuration: initial_configuration,
                updated: false,
            }),
            std::sync::Condvar::new(),
        ));
        let thread_flagged_configuration_and_condition =
            flagged_configuration_and_condition.clone();
        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let thread_running = running.clone();
        Self {
            flagged_configuration_and_condition,
            thread: Some(std::thread::spawn(move || {
                let mut context = context;
                let mut previous_configuration = previous_configuration;
                while thread_running.load(std::sync::atomic::Ordering::Acquire) {
                    let configuration = {
                        let (lock, cvar) = &*thread_flagged_configuration_and_condition;
                        // unwrap: mutex is not poisoned
                        let mut flagged_configuration = lock.lock().unwrap();
                        if !flagged_configuration.updated {
                            // unwrap: mutex is not poisoned
                            flagged_configuration = cvar
                                .wait_timeout(
                                    flagged_configuration,
                                    std::time::Duration::from_millis(100),
                                )
                                .unwrap()
                                .0;
                        }
                        if flagged_configuration.updated {
                            flagged_configuration.updated = false;
                            Some(flagged_configuration.configuration.clone())
                        } else {
                            None
                        }
                    };
                    if let Some(configuration) = configuration {
                        context = update(context, &previous_configuration, &configuration);
                        previous_configuration = configuration;
                    }
                }
            })),
            running,
        }
    }

    pub fn update(&self, configuration: Configuration) {
        let (lock, cvar) = &*self.flagged_configuration_and_condition;
        // unwrap: mutex is not poisoned
        let mut flagged_configuration = lock.lock().unwrap();
        flagged_configuration.configuration = configuration;
        flagged_configuration.updated = true;
        cvar.notify_one();
    }
}

impl<Configuration> Drop for Updater<Configuration> {
    fn drop(&mut self) {
        self.running
            .store(false, std::sync::atomic::Ordering::Release);
        if let Some(thread) = self.thread.take() {
            // unwrap: not joining self
            thread.join().unwrap();
        }
    }
}
