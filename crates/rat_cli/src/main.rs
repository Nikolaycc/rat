use rat::capture::sync::Capture;
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let cap = Capture::new(&args.interface)?;
    let mut cap = cap.as_async()?;

    let registry = ProtocolRegistry::builder()
        .defaults()
        .build()
        .expect("Failed to build ProtocolRegistry");
    let parser = Arc::new(Parser::new(registry));

    cap.run_loop(parser, {
        async move |parser, batch| {
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
