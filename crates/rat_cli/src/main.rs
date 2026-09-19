use rat::capture::Capture;
use rat::parser::Parser;
use rat::protocols::{
    arp::ARPFrame, ethernet::EthernetFrame, icmp::ICMPFrame, ipv4::IPv4Frame, ipv6::IPv6Frame,
    ospf::OSPFFrame, tcp::TCPFrame, udp::UDPFrame,
};
use rat::registry::ProtocolRegistry;

use clap::Parser as ClapParser;

#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let cap = Capture::new(&args.name)?;

    let registry = ProtocolRegistry::builder()
        .root::<EthernetFrame>()
        .register::<IPv4Frame>()
        .register::<IPv6Frame>()
        .register::<ARPFrame>()
        .register::<ICMPFrame>()
        .register::<OSPFFrame>()
        .register::<TCPFrame>()
        .register::<UDPFrame>()
        .build()
        .expect("Failed to build ProtocolRegistry");
    let parser = Parser::new(&registry);

    for batch in cap {
        for packet in batch {
            for layer in parser.parse(packet.bytes()) {
                let layer = layer.unwrap();
                println!("{layer}");

                if let Some(tcp) = layer.get::<TCPFrame>() {
                    if tcp.source_port == 443 || tcp.destination_port == 443 {
                        println!("Bingo! i catch http packet here.")
                    }
                }
            }
        }
    }

    Ok(())
}
