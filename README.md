# 🐀 Rat

A nimble network sniffer that scurries through your traffic

> [!WARNING]
> This library is unfinished. Keep your expectations low.

Basic example

```rs
#[packet(
    layer = Network,
    parent = IPv4Frame,
    selector = 89
)]
pub struct OSPFFrame {
    pub version: u8,
    pub typ: u8,
    pub packet_length: u16,
    pub router_id: u32,
    pub area_id: u32,
    pub checksum: u16,
    pub autype: u16,
    pub authentication: u64,
}
    
fn main() -> Result<(), dyn Error> {
    let capture = Capture::open("eth0")?;
    
    let registry = ProtocolRegistry::builder()
        .defaults()
        .register::<OSPFFrame>()
        .build()?;
    let parser = Parser::new(registry);
    
    let mut parallel = ParallelCapture::from(capture, parser, 4)?;
    
    parallel.run_loop(|batch, parser| {
        for raw in batch {
            for layer in parser.parse(raw.bytes()) {
                let Ok(layer) = layer else { continue };

                if let Some(tcp) = layer.get::<TCPFrame>() {
                    println!("TCP: {tcp:?}");
                }
            }
        }
    })?;

    Ok(())
}
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## Roadmap

*   Add BPF filter support
*   Add Linux compatibility
*   Make packet capture fully zero-copy
