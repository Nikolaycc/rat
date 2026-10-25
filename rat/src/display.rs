use std::time::Duration;

use gpui_shared_string::SharedString;

use crate::addrs::Addr;

pub struct PacketView {
    pub no: usize,
    pub time: Duration,
    pub source: Box<dyn Addr>,
    pub destination: Box<dyn Addr>,
    pub protocol: SharedString,
    pub length: usize,
}

pub trait DisplayGPUI {
    fn as_packet_view(&self) -> PacketView;
}
