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

    pub fn execute<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T> + Send,
        T: Send + 'static,
    {
        // Warning: blocks the calling thread. Avoid for long tasks on UI thread.
        tokio::task::block_in_place(|| self.handle.block_on(future))
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.handle.spawn(future);
    }
}
