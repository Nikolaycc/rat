use rat::capture::Capture;
use rat::packets::bpf::BPFFrame;
use rat::packets::ethernet::EthernetFrame;

const ETH_ALEN: libc::c_int = 6;

#[repr(C, packed)]
pub struct ethhdr {
    pub h_dest: [libc::c_uchar; ETH_ALEN as usize],
    pub h_source: [libc::c_uchar; ETH_ALEN as usize],
    pub h_proto: u16,
}

fn main() -> std::io::Result<()> {
    println!(
        "BPFFrame sizeof = {}\n libc::bpf_hdr sizeof = {}",
        size_of::<BPFFrame>(),
        size_of::<libc::bpf_hdr>()
    );

    println!(
        "EthernetFrame sizeof = {} align = {}\n libc::eth_hdr sizeof = {} align = {}",
        size_of::<EthernetFrame>(),
        align_of::<EthernetFrame>(),
        size_of::<ethhdr>(),
        align_of::<ethhdr>()
    );

    let mut cap = Capture::new("en1")?;

    cap.run_loop(|packet| match EthernetFrame::parse(packet) {
        Ok(ethernet) => {
            println!(
                "EthernetFrame src: {:?}, dst {:?}, type: {}",
                ethernet.source_addr,
                ethernet.dest_addr,
                { ethernet.ty }
            );
        }
        Err(error) => {
            eprintln!("Ethernet parse error: {error:?}");
        }
    });

    Ok(())
}
