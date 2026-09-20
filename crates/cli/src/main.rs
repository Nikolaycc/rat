use rat::capture::sync::Capture;
use rat::capture::tokio::AsyncCapture;
use rat::parser::Parser;
use rat::registry::ProtocolRegistry;
use rat::utils::ParseError;

use clap::Parser as ClapParser;
use std::sync::Arc;

#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    interface: String,

    #[arg(short, long, default_value_t = 4)]
    workers: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let cap = Capture::new(&args.interface)?;
    let mut cap = AsyncCapture::with_workers(cap, args.workers)?;

    let registry = ProtocolRegistry::builder()
        .defaults()
        .build()
        .expect("Failed to build ProtocolRegistry");
    let parser = Arc::new(Parser::new(registry));

    cap.run_loop(parser, {
        async move |parser, batch| {
            let spawned_id = tokio::task::id();
            println!("Spawned task ID: {:?}", spawned_id);

            for raw in batch {
                for layer in parser.parse(raw.bytes()) {
                    let layer = layer?;

                    println!("{layer}");
                }
            }

            Ok::<(), ParseError>(())
        }
    })
    .await?;

    Ok(())
}
