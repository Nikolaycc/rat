use rat::capture::Capture;
use rat::packet::Packet;
use rat::protocols::{
    arp::ARPFrame,
    ethernet::{EtherType, EthernetFrame},
    ipv4::IPv4Frame,
    ipv6::IPv6Frame,
    tcp::TCPFrame,
    udp::UDPFrame,
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let cap = Capture::new(&args.name)?;

    for batch in cap {
        for mut packet in batch {
            match EthernetFrame::parse(packet.data()) {
                Ok(ethernet) => {
                    let next_protocol = EtherType::from(ethernet.ty.get());

                    if EtherType::IPv6 == next_protocol {
                        println!("{ethernet}");
                    }

                    let b = packet.0.split_off(size_of::<EthernetFrame>());

                    match next_protocol {
                        EtherType::ARP => match ARPFrame::parse(&b) {
                            Ok(arp) => {
                                println!("{arp}");
                            }
                            Err(error) => {
                                eprintln!("ARP parse error: {error:?}");
                            }
                        },
                        EtherType::IPv6 => match IPv6Frame::parse(&b) {
                            Ok(ipv6) => {
                                println!("{ipv6}");
                            }
                            Err(error) => {
                                eprintln!("IPv6 parse error: {error:?}");
                            }
                        },
                        EtherType::IP => match IPv4Frame::parse(&b) {
                            Ok(ip) => {
                                let c = b.clone().split_off(size_of::<IPv4Frame>());

                                println!("{ip}");

                                match ip.protocol {
                                    6 => {
                                        if let Ok(tcp) = TCPFrame::parse(&c) {
                                            println!("{tcp}");
                                        }
                                    }
                                    17 => {
                                        if let Ok(udp) = UDPFrame::parse(&c) {
                                            println!("{udp}");
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
