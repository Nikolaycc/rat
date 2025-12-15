extern crate Rat;

use Rat::RatCap;

fn main() -> Result<(), String> {
    let rat = RatCap::from("en1")?;

    for packet in rat.take(5) {
		println!("{} Captured packet length: {}", packet.timestamp, packet.length);
		dbg!(packet);
    }
    
    Ok(())
}
