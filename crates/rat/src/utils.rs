// reference: https://github.com/rust-lang/socket2/blob/master/src/sys/unix.rs#L346
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        #[allow(unused_unsafe)]
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

macro_rules! syscallu {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        #[allow(unused_unsafe)]
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

pub(crate) use {syscall, syscallu};

#[derive(Debug)]
pub enum ParseError {
    PacketTooShort { expected: usize, actual: usize },
    InvalidValue,
    UnsupportedProtocol(u16),
}
