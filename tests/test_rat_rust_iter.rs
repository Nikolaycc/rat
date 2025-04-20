extern crate Rat;

use Rat::RatCap;
    
fn main() -> Result<(), String> {
    let rat = RatCap::new()?;

    for packet in rat.take(5) {
	println!("Captured packet length: {}", packet.length);
    }
    
    Ok(())
}
