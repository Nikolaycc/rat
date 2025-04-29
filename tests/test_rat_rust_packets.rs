extern crate Rat;

use Rat::RatCap;
use Rat::packets::*;

use std::net::Ipv4Addr;

fn main() -> Result<(), String> {
    let rat = RatCap::new()?;

    for raw_packet in rat.take(5) {
	println!("{} Captured packet length: {}", raw_packet.timestamp, raw_packet.length);

	let packet = unsafe { ParsedPacket::from_raw(&raw_packet) };

        for header in &packet {
            match header {
                PacketHeader::Ethernet(e) => println!("Ethernet: src={:02x?} -> dst={:02x?}", e.source, e.destination),
                PacketHeader::Arp(a) => println!("ARP: {} -> {} op={}", Ipv4Addr::from(a.sender_ip), Ipv4Addr::from(a.target_ip), a.operation),
                PacketHeader::Ip(i) => println!("IP: {:?} -> {:?} proto={} ttl={}", Ipv4Addr::from(i.src), Ipv4Addr::from(i.dst), i.protocol, i.ttl),
                PacketHeader::Transport(t) => match t {
                    TransportHeader::Tcp(tcp) => println!("TCP: {} -> {} flags=0x{:02x}", tcp.src_port, tcp.dst_port, tcp.flags),
                    TransportHeader::Udp(udp) => println!("UDP: {} -> {} len={}", udp.src_port, udp.dst_port, udp.length),
                }
            }
        }

        // Access payload
        println!("Payload ({} bytes)", packet.payload().len());
    }
    
    Ok(())
}
