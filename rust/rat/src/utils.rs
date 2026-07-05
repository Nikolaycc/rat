use std::ptr;

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

pub(crate) use syscall;

#[derive(Debug)]
pub enum ParseError {
    PacketTooShort { expected: usize, actual: usize },
    InvalidValue,
    UnsupportedProtocol(u16),
}

// convert in bytes array first next we checking array size if is over 16 we need to return error
// next creating buf and fill zeros and we iterate with zip bcs zips giving tuple but same value and same address and we can cast into i8
// and return buf
pub(crate) fn str_to_ifname(name: &str) -> Result<[i8; 16], std::io::Error> {
    let bytes = name.as_bytes();

    // must fit with room for the trailing '\0' → max 15 chars
    if bytes.len() >= 16 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "interface name too long",
        ));
    }

    let mut buf = [0i8; 16]; // zero-filled → null terminator is automatic
    for (dst, &src) in buf.iter_mut().zip(bytes) {
        *dst = src as i8; // u8 → i8, same bits
    }
    Ok(buf)
}

pub(crate) fn bpf_wordalign(length: usize) -> usize {
    let alignment = libc::BPF_ALIGNMENT as usize;
    (length + alignment - 1) & !(alignment - 1)
}

pub(crate) fn inspect_bpf_buffer<F>(buf: &[u8], mut inspect_packet: F)
where
    F: FnMut(&[u8]),
{
    let mut offset = 0usize;

    while offset + size_of::<libc::bpf_hdr>() <= buf.len() {
        let header =
            unsafe { ptr::read_unaligned(buf.as_ptr().add(offset).cast::<libc::bpf_hdr>()) };

        let header_len = header.bh_hdrlen as usize;
        let captured_len = header.bh_caplen as usize;

        if header_len == 0 || captured_len == 0 {
            break;
        }

        let Some(packet_start) = offset.checked_add(header_len) else {
            break;
        };

        let Some(packet_end) = packet_start.checked_add(captured_len) else {
            break;
        };

        if packet_end > buf.len() {
            eprintln!(
                "invalid BPF record: packet_end={packet_end}, buffer_len={}",
                buf.len()
            );
            break;
        }

        println!(
            "captured={}, original={}, header_len={}",
            header.bh_caplen, header.bh_datalen, header.bh_hdrlen,
        );

        let packet = &buf[packet_start..packet_end];

        inspect_packet(packet);

        let record_len = bpf_wordalign(header_len + captured_len);

        if record_len == 0 {
            break;
        }

        let Some(next_offset) = offset.checked_add(record_len) else {
            break;
        };

        if next_offset > buf.len() {
            break;
        }

        offset = next_offset;
    }
}
