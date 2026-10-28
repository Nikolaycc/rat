use std::{
    fs::File,
    io::{self, Read},
    panic::AssertUnwindSafe,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use bytes::Bytes;
use object_pool::{Pool, ReusableOwned};

use crate::{
    Capture, Parser,
    capture::{BufferPool, CaptureIter, RawPacket},
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
    captured: usize,
    running: AtomicBool,
}

impl ParallelCapture {
    #[inline]
    pub fn from(cap: Capture, parser: Parser, size: usize) -> Result<Self, PoolCreationError> {
        Ok(Self {
            fd: cap.fd,
            buf_pool: Arc::new(Pool::new(size, || {
                vec![0u8; cap.buf_len].into_boxed_slice()
            })),
            thread_pool: ThreadPool::new(size + 4)?,
            parser,
            captured: 0,
            running: AtomicBool::new(false),
        })
    }

    pub fn run_loop<F>(&mut self, callback: F) -> Result<(), CaptureLoopError>
    where
        F: Fn(ParallelBatch, &Parser) + Sync + Send + 'static,
    {
        let parser = self.parser.clone();
        let callback = Arc::new(move |batch: ParallelBatch| callback(batch, &parser));
        self.running.store(true, Ordering::Relaxed);

        while self.running.load(Ordering::Relaxed) {
            let mut attempt = 0u32;

            let mut buf = loop {
                if let Some(b) = self.buf_pool.try_pull_owned() {
                    break b;
                }

                if !self.running.load(Ordering::Relaxed) {
                    return Ok(());
                }

                match attempt {
                    0..=10 => std::hint::spin_loop(),
                    11..=20 => std::thread::yield_now(),
                    _ => std::thread::sleep(Duration::from_micros(50)),
                }

                attempt += 1;
            };

            let size = match self.fd.read(&mut buf[..]) {
                Ok(0) => continue,
                Ok(n) => {
                    self.captured += 1;
                    n
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) if e.kind() == io::ErrorKind::TimedOut => continue, // lets you re-check `running`
                Err(e) => return Err(CaptureLoopError::IO(e)),
            };

            let cb = Arc::clone(&callback);

            self.thread_pool.execute(move || {
                let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    cb(ParallelBatch { buf, len: size });
                }));
            });
        }

        Ok(())
    }
}
