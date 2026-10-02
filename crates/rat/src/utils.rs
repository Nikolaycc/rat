// reference: https://github.com/rust-lang/socket2/blob/master/src/sys/unix.rs#L346
/// Helper macro to execute a system call that returns an `io::Result`.
///
/// # Safety
///
/// This macro wraps an 'unsafe' call to 'libc', so the caller must uphold the
/// requirements of the system call being invoked. For example, pointers must
/// be vaild for the access the call performs, lengths must match the buffers,
/// and file descriptors must be vaild and in the expected state.
///
/// The system call must report failure by returning '-1' and setting 'errno'.
/// Do not use this macro for calls that signal errors differently.
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        // SAFETY: the caller guarantees that the arguments are vaild for
        // the system call (see the `# Safety` section above).
        #[allow(unused_unsafe)]
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            // `errno` is read immediately, so nothing can overwrite it
            // between the system call and this line.
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

/// Helper macro to execute a system call that returns an `io::Result`.
///
/// # Safety
///
/// This macro wraps an 'unsafe' call to 'libc', so the caller must uphold the
/// requirements of the system call being invoked. For example, pointers must
/// be vaild for the access the call performs, lengths must match the buffers,
/// and file descriptors must be vaild and in the expected state.
///
/// The system call must report failure by returning '0' and setting 'errno'.
/// Do not use this macro for calls that signal errors differently.
macro_rules! syscallu {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        // SAFETY: the caller guarantees that the arguments are vaild for
        // the system call (see the `# Safety` section above).
        #[allow(unused_unsafe)]
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == 0 {
            // `errno` is read immediately, so nothing can overwrite it
            // between the system call and this line.
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

pub(crate) use {syscall, syscallu};
