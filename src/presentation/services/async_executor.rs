use std::future::Future;
use tokio::runtime::Handle;

#[derive(Clone)]
pub struct AsyncExecutor {
    handle: Handle,
}

impl AsyncExecutor {
    pub fn new(handle: Handle) -> Self {
        Self { handle }
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.handle.spawn(future);
    }
}
