use rat::capture::Capture;
use rat::capture::parallel::ParallelCapture;
use rat::interface::IFaceMap;
use rat::parser::Parser;
use rat::registry::ProtocolRegistry;

use clap::Parser as ClapParser;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.show_interfaces {
        let ifaces = IFaceMap::new()?;
        println!("{ifaces}");

        return Ok(());
    }

    let cap = Capture::open(&args.interface.unwrap())?;

    let registry = ProtocolRegistry::builder()
        .defaults()
        .build()
        .expect("Failed to build ProtocolRegistry");
    let parser = Parser::new(registry);

    let mut cap = ParallelCapture::from(cap, parser, args.workers)?;

    cap.run_loop(|batch, parser| {
        for raw in batch {
            for layer in parser.parse(raw.bytes()) {
                let layer = layer.unwrap();

                print!("{layer}");
            }
        }
    })?;

    Ok(())
}
