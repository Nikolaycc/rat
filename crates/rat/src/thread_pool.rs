use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PoolCreationError {
    #[error("size can't be zero")]
    ZeroSize,
}

#[derive(Debug, PartialEq)]
pub struct ThreadPool;

impl ThreadPool {
    /// Create a new `ThreadPool`.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// #Errors
    ///
    /// Will return 'Err' if the size is zero.
    pub fn new(size: usize) -> Result<Self, PoolCreationError> {
        if size == 0 {
            return Err(PoolCreationError::ZeroSize);
        }

        Ok(ThreadPool)
    }

    /// Executes a function on a worker thread.
    ///
    /// The function contains the logic that runs in the pool.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
    }
}

#[cfg(test)]
mod tests {
    use crate::thread_pool::{PoolCreationError, ThreadPool};

    #[test]
    fn zero_size_pool() {
        let pool = ThreadPool::new(0);

        assert_eq!(pool, Err(PoolCreationError::ZeroSize))
    }

    #[test]
    fn execute_thread() {
        let pool = ThreadPool::new(5).unwrap();

        pool.execute(|| {})
    }
}
