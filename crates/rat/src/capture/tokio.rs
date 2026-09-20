use bytes::Bytes;
use object_pool::{Pool, ReusableOwned};
use std::{io, os::fd::OwnedFd, sync::Arc};
use thiserror::Error;
use tokio::{
    io::{Interest, unix::AsyncFd},
    sync::{OwnedSemaphorePermit, Semaphore},
    task::JoinSet,
};

use crate::{
    capture::sync::{Active, Capture, CaptureIter, RawPacket},
    io::{read, set_nonblocking},
    parser::Parser,
};

const DEFAULT_MAX_WORKER: usize = 4;

#[derive(Debug, Error)]
pub enum RunError<E> {
    #[error("capture failed: {0}")]
    Capture(#[source] io::Error),

    #[error("batch worker failed")]
    Worker(E),

    #[error("batch task failed: {0}")]
    Join(#[source] tokio::task::JoinError),
}

type CapturePool = Arc<Pool<Box<[u8]>>>;

pub struct AsyncBatch {
    buf: ReusableOwned<Box<[u8]>>,
    len: usize,
    _permit: OwnedSemaphorePermit,
}

pub struct AsyncCapture {
    fd: AsyncFd<OwnedFd>,
    pool: CapturePool,
    available: Arc<Semaphore>,
}

impl AsRef<[u8]> for AsyncBatch {
    fn as_ref(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl AsyncCapture {
    #[must_use]
    #[inline]
    pub fn from(cap: Capture<Active>) -> io::Result<Self> {
        AsyncCapture::with_workers(cap, DEFAULT_MAX_WORKER)
    }

    #[must_use]
    pub fn with_workers(cap: Capture<Active>, workers: usize) -> io::Result<Self> {
        set_nonblocking(&cap.fd)?;

        let fd = AsyncFd::with_interest(cap.fd, Interest::READABLE)?;

        Ok(Self {
            fd,
            pool: Arc::new(Pool::new(workers, || {
                vec![0u8; cap.buf_len].into_boxed_slice()
            })),
            available: Arc::new(Semaphore::new(workers)),
        })
    }

    pub async fn next_batch(&mut self) -> io::Result<AsyncBatch> {
        loop {
            let mut ready = self.fd.readable().await?;

            let permit = self
                .available
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| io::Error::other("capture buffer pool closed"))?;

            let mut buffer = self
                .pool
                .try_pull_owned()
                .expect("semaphore and capture pool are out of sync");

            match ready.try_io(|fd| read(fd.get_ref(), &mut buffer)) {
                Ok(Ok(size)) => {
                    return Ok(AsyncBatch {
                        buf: buffer,
                        len: size,
                        _permit: permit,
                    });
                }

                Ok(Err(error)) => {
                    return Err(error);
                }

                Err(_) => {
                    // EWOULDBLOCK:
                    // wait for readiness again.
                    continue;
                }
            }
        }
    }

    pub async fn run_loop<F, Fut, E>(
        &mut self,
        parser: Arc<Parser>,
        mut callback: F,
    ) -> Result<(), RunError<E>>
    where
        F: FnMut(Arc<Parser>, AsyncBatch) -> Fut,
        Fut: Future<Output = Result<(), E>> + Send + 'static,
        E: Send + 'static,
    {
        let mut tasks = JoinSet::new();

        loop {
            let batch = self.next_batch().await.map_err(RunError::Capture)?;

            tasks.spawn(callback(Arc::clone(&parser), batch));

            while let Some(res) = tasks.try_join_next() {
                match res {
                    Ok(Ok(())) => {}

                    Ok(Err(error)) => return Err(RunError::Worker(error)),

                    Err(error) => return Err(RunError::Join(error)),
                }
            }
        }
    }
}

impl IntoIterator for AsyncBatch {
    type Item = RawPacket;
    type IntoIter = CaptureIter;

    fn into_iter(self) -> Self::IntoIter {
        CaptureIter {
            data: Bytes::from_owner(self),
            offset: 0usize,
        }
    }
}
