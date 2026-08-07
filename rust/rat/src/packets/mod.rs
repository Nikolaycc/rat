pub mod bpf;
pub mod ethernet;

pub enum Layer {}

pub trait Packet {
    pub fn typ();
}
