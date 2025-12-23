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
        // For synchronous execution blocking on a future (be careful not to block UI thread main loop too long)
        // Ideally we shouldn't use this in the main UI thread for long tasks
        tokio::task::block_in_place(|| self.handle.block_on(future))
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.handle.spawn(future);
    }
}
