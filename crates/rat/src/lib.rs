pub mod addrs;
pub mod bpf;
pub mod capture;
pub mod io;
pub mod packet;
pub mod protocols;
pub mod registry;
pub mod utils;

#[cfg(test)]
mod tests {
    use crate::bpf::BPFFrame;

    #[test]
    fn size_of_bpf_frame() {
        assert_eq!(size_of::<BPFFrame>(), size_of::<libc::bpf_hdr>());
        assert_eq!(align_of::<BPFFrame>(), align_of::<libc::bpf_hdr>());
    }
}
