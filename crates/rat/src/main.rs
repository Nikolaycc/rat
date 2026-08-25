use rat::capture::Capture;
use rat::packets::{
    Packet,
    arp::ARPFrame,
    bpf::BPFFrame,
    ethernet::{EtherType, EthernetFrame},
    ip::IPFrame,
    tcp::TCPFrame,
    udp::UDPFrame,
};

fn main() -> std::io::Result<()> {
    dbg!(
        size_of::<BPFFrame>(),
        align_of::<BPFFrame>(),
        size_of::<libc::bpf_hdr>(),
        align_of::<libc::bpf_hdr>()
    );

    let cap = Capture::new("en1")?;

    for batch in cap {
        for mut packet in batch {
            match EthernetFrame::parse(packet.data()) {
                Ok(ethernet) => {
                    let next_protocol = EtherType::from(ethernet.ty.get());
                    println!(
                        "EthernetFrame src: {}, dst {}, type: {}",
                        ethernet.src, ethernet.dst, next_protocol,
                    );

                    let b = packet.0.split_off(size_of::<EthernetFrame>());

                    match next_protocol {
                        EtherType::ARP => match ARPFrame::parse(&b) {
                            Ok(arp) => {
                                println!(
                                    "ARPFrame sender IP: {} sender MAC Address: {} -> target IP: {} target MAC Address: {}",
                                    arp.sender_ip_address,
                                    arp.sender_hardware_address,
                                    arp.target_ip_address,
                                    arp.target_hardware_address
                                );
                            }
                            Err(error) => {
                                eprintln!("ARP parse error: {error:?}");
                            }
                        },
                        EtherType::IP => match IPFrame::parse(&b) {
                            Ok(ip) => {
                                let c = b.clone().split_off(size_of::<IPFrame>());

                                println!(
                                    "IPFrame source IP: {} -> destination IP: {}",
                                    ip.source_address, ip.destination_address
                                );

                                match ip.protocol {
                                    6 => {
                                        if let Ok(tcp) = TCPFrame::parse(&c) {
                                            println!(
                                                "TCPFrame: source port {} -> destination port: {}",
                                                tcp.source_port, tcp.destination_port
                                            );
                                        } else {
                                            eprintln!("TCP parse error");
                                        }
                                    }
                                    17 => {
                                        if let Ok(udp) = UDPFrame::parse(&c) {
                                            println!(
                                                "UDPFrame: source port {} -> destination port {} lenght {}",
                                                udp.src, udp.dst, udp.lenght
                                            );
                                        } else {
                                            eprintln!("UDP parse error");
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Err(error) => {
                                eprintln!("IP parse error: {error:?}");
                            }
                        },
                        _ => {}
                    }
                }
                Err(error) => {
                    eprintln!("Ethernet parse error: {error:?}");
                }
            }
        }
    }

    Ok(())
}
