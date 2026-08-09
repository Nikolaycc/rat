use rat::capture::Capture;
use rat::packets::Packet;
use rat::packets::bpf::BPFFrame;
use rat::packets::ethernet::{EtherType, EthernetFrame};

fn main() -> std::io::Result<()> {
    dbg!(
        size_of::<BPFFrame>(),
        align_of::<BPFFrame>(),
        size_of::<libc::bpf_hdr>(),
        align_of::<libc::bpf_hdr>()
    );

    let cap = Capture::new("en1")?;

    for batch in cap {
        for packet in batch {
            match EthernetFrame::parse(packet.data()) {
                Ok(ethernet) => {
                    println!(
                        "EthernetFrame src: {}, dst {}, type: {}",
                        ethernet.src,
                        ethernet.dst,
                        EtherType::from(ethernet.ty.get())
                    );
                }
                Err(error) => {
                    eprintln!("Ethernet parse error: {error:?}");
                }
            }
        }
    }

    Ok(())
}
