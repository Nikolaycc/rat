# 🐀 Rat

A nimble network sniffer that scurries through your traffic

Rat is a lightweight packet sniffing library and CLI tool written in C, designed to help you analyze network traffic with the stealth and precision of its namesake.

> [!WARNING]
> This library is unfinished. Keep your expectations low.

## Building

Dependencies

* meson (build system)
* ninja
* readline (for CLI)

```bash
 git clone https://github.com/nikolaycc/rat.git
 cd rat
 
 meson setup build
 cd build
 ninja
```

## Command Line Interface

```bash
$ ./rat_cli
Welcome to RAT Sniffer CLI. Type 'help' for commands.
rat> list
Available interfaces:
0: eth0 (Ethernet interface)
1: wlan0 (Wireless interface)
rat> start 0
Started sniffing on eth0
[Packet display appears here]
rat> stop
Stopped sniffing
rat> exit
```

## Usage (rat)

Basic Example in C

```c
#include <rat.h>

void packet_handler(rat_packet_t* packet, void* data) {
	// Process packet here
}

int main(void) {
    rat_device_t devices[MAX_INTERFACES];
    int devices_size = rat_device_lookup(devices);
    if (devices_size == 0) {
        printf("No network devices found\n");
        return 1;
    }
    
    int device_idx = rat_device_pick(devices, devices_size);
    if (device_idx < 0) {
	printf("No network devices index found\n");
        return 1;
    }

    rat_cap_t cap = {0};
    rat_cap_create(&cap, &devices[device_idx], NULL, 0);
    
    rat_cap_loop(&cap, cap_cb, 2);
    
    rat_cap_destroy(&cap);
    
    return 0;
}
```

Basic example in rust

```rs
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

        println!("Payload ({} bytes)", packet.payload().len());
    }
    
    Ok(())
}
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## Roadmap

*   Add IPv6 support
*   Add ICMP support
*   Add BPF filter support
*   Add Windows compatibility
*   Add BSD compatibility
