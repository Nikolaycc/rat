# 🐀 Rat

A nimble network sniffer that scurries through your traffic

> [!WARNING]
> This library is unfinished. Keep your expectations low.

Basic example

```rs
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

```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## Roadmap

*   Add BPF filter support
*   Add Linux compatibility
*   Make packet capture fully zero-copy
