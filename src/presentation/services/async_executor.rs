use std::future::Future;
use std::sync::Arc;

pub struct AsyncExecutor {
    runtime: Arc<tokio::runtime::Runtime>,
}

impl AsyncExecutor {
    pub fn new() -> Self {
        Self {
            runtime: Arc::new(tokio::runtime::Runtime::new().unwrap()),
        }
    }

    pub fn execute<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T>,
    {
        self.runtime.block_on(future)
    }
}

impl Clone for AsyncExecutor {
    fn clone(&self) -> Self {
        Self {
            runtime: Arc::clone(&self.runtime),
        }
    }
}

impl Default for AsyncExecutor {
    fn default() -> Self {
        Self::new()
    }
}
