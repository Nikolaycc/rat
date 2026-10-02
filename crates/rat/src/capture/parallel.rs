use std::{fs::File, io::Read, sync::Arc};

use bytes::Bytes;
use object_pool::{Pool, ReusableOwned};

use crate::{
    Capture, Parser,
    capture::{Active, BufferPool, CaptureIter, RawPacket},
    error::{CaptureLoopError, PoolCreationError},
    thread_pool::ThreadPool,
};

pub struct ParallelBatch {
    buf: ReusableOwned<Box<[u8]>>,
    len: usize,
}

impl AsRef<[u8]> for ParallelBatch {
    fn as_ref(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl IntoIterator for ParallelBatch {
    type Item = RawPacket;
    type IntoIter = CaptureIter;

    fn into_iter(self) -> Self::IntoIter {
        CaptureIter {
            data: Bytes::from_owner(self),
            offset: 0usize,
        }
    }
}

pub struct ParallelCapture {
    fd: File,
    buf_pool: BufferPool,
    thread_pool: ThreadPool,
    parser: Parser,
}

impl ParallelCapture {
    #[inline]
    pub fn from(
        cap: Capture<Active>,
        parser: Parser,
        size: usize,
    ) -> Result<Self, PoolCreationError> {
        Ok(Self {
            fd: cap.fd,
            buf_pool: Arc::new(Pool::new(size, || {
                vec![0u8; cap.buf_len].into_boxed_slice()
            })),
            thread_pool: ThreadPool::new(size)?,
            parser,
        })
    }

    pub fn run_loop<F>(&mut self, callback: F) -> Result<(), CaptureLoopError>
    where
        F: Fn(ParallelBatch, &Parser) + Sync + Send + 'static,
    {
        let parser = self.parser.clone();
        let callback = Arc::new(move |batch: ParallelBatch| callback(batch, &parser));

        loop {
            let mut buf = self
                .buf_pool
                .try_pull_owned()
                .ok_or(CaptureLoopError::PoolFailed)?;

            let size = match self.fd.read(&mut buf[..]) {
                Ok(n) => n,
                Err(_) => break,
            };

            let cb = Arc::clone(&callback);

            self.thread_pool.execute(move || {
                cb(ParallelBatch { buf, len: size });
            });
        }

        Ok(())
    }
}
