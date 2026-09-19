# 🐀 Rat

A nimble network sniffer that scurries through your traffic

> [!WARNING]
> This library is unfinished. Keep your expectations low.

Basic example

```rs
    let cap = Capture::new("en1")?;

    let registry = ProtocolRegistry::builder()
        .defaults()
        .register::<OSPFFrame>()
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
                        println!("Bingo!")
                    }
                }
            }
        }
    }

```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## Roadmap

*   Add BPF filter support
*   Add Linux compatibility
*   Make packet capture fully zero-copy
