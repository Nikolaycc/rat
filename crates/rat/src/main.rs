use rat::capture::Capture;
use rat::packets::Packet;
use rat::packets::bpf::BPFFrame;
use rat::packets::ethernet::{EtherType, EthernetFrame};

use rat::addrs::MacAddr;

fn main() -> std::io::Result<()> {
    println!(
        "BPFFrame sizeof = {}\n libc::bpf_hdr sizeof = {}",
        size_of::<BPFFrame>(),
        size_of::<libc::bpf_hdr>()
    );

    let mut cap = Capture::new("en1")?;

    cap.run_loop(|packet| match EthernetFrame::parse(packet) {
        Ok(ethernet) => {
            println!(
                "EthernetFrame src: {}, dst {}, type: {}",
                MacAddr::new(ethernet.source_addr),
                MacAddr::new(ethernet.dest_addr),
                EtherType::from(ethernet.ty.get())
            );
        }
        Err(error) => {
            eprintln!("Ethernet parse error: {error:?}");
        }
    });

    Ok(())
}
