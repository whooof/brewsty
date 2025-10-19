use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub enum AppEvent {
    StatusUpdate(String),
    LogMessage(String),
    LoadingStateChange(bool),
}

pub struct EventBus {
    listeners: Arc<Mutex<Vec<Box<dyn Fn(&AppEvent) + Send + Sync>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn publish(&self, event: AppEvent) {
        let listeners = self.listeners.lock().unwrap();
        for listener in listeners.iter() {
            listener(&event);
        }
    }

    pub fn subscribe<F>(&self, listener: F)
    where
        F: Fn(&AppEvent) + Send + Sync + 'static,
    {
        self.listeners.lock().unwrap().push(Box::new(listener));
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            listeners: Arc::clone(&self.listeners),
        }
    }
}
