use rat::capture::Capture;
use rat::capture::tokio::AsyncCapture;
use rat::interface::IFaceMap;
use rat::parser::{Parser, ParserError};
use rat::registry::ProtocolRegistry;

use clap::Parser as ClapParser;
use std::sync::Arc;

#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    show_interfaces: bool,

    #[arg(
        short,
        long,
        required_unless_present = "show_interfaces",
        conflicts_with = "show_interfaces"
    )]
    interface: Option<String>,

    #[arg(short, long, default_value_t = 4)]
    workers: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.show_interfaces {
        let ifaces = IFaceMap::new()?;
        println!("{ifaces}");

        return Ok(());
    }

    let cap = Capture::open(&args.interface.unwrap())?;
    let mut cap = AsyncCapture::with_workers(cap, args.workers)?;

    let registry = ProtocolRegistry::builder()
        .defaults()
        .build()
        .expect("Failed to build ProtocolRegistry");
    let parser = Arc::new(Parser::new(registry));

    cap.run_loop(parser, {
        async move |parser, batch| {
            for raw in batch {
                for layer in parser.parse(raw.bytes()) {
                    let layer = layer.unwrap();

                    print!("{layer}");
                }
            }

            Ok::<(), ParserError>(())
        }
    })
    .await?;

    Ok(())
}
