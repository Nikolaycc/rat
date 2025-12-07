use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, SharedString, Window,
    WindowBounds, WindowOptions, uniform_list,
};

#[derive(Clone, Debug)]
struct PacketView {
    view_name: SharedString,
    packet_len: usize,
    more_info: SharedString,
    timestamp: SharedString,
}

fn packet_view(packet: &PacketView) -> impl IntoElement {
    let pk = packet.clone();
    
    div()
	.id(packet.packet_len + 2 * 4)
	.px_2()
	.py_4()
	.h_10()
	.my_5()
	.text_center()
	.shadow_md()
	.border_b_1()
	.border_color(gpui::black())
	.bg(gpui::white())
	.cursor_pointer()
	.child(format!("{} Packet {} {}: {}", pk.timestamp.clone(), pk.packet_len.clone(), pk.view_name.clone(), pk.more_info.clone()))
	.on_click(move |_, _window, _cx| {
	    println!("packet: {:?}", pk);
	})
}


struct PacketList {
    packets: Vec<PacketView>,
}

impl Render for PacketList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
	div().size_full().flex().flex_col().gap_3().bg(rgb(0xffffff)).child(
	    uniform_list(cx.entity(), "entries", self.packets.len(),
			 |this, range, _window, _cx| {
			     let mut items = Vec::new();

			     for idx in range {
				 let item = this.packets[idx].clone();
				 
				 items.push(
				     packet_view(&item),
				 );
			     }

			     items
			 }
	    ).h_full().gap_3()
	)
    }
}

// impl Render for PacketList {
//     fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
//         div()
//             .flex()
//             .flex_col()
//             .gap_3()
//             .bg(rgb(0x505050))
//             .size(px(500.0))
//             .justify_center()
//             .items_center()
//             .shadow_lg()
//             .border_1()
//             .border_color(rgb(0x0000ff))
//             .text_xl()
//             .text_color(rgb(0xffffff))
//             .child("Hello!")
//             .child(
//                 div()
//                     .flex()
//                     .flex_col()
//                     .gap_2()
//                     .child(div().text_color(rgb(0xffffff)).text_lg().items_center().justify_center().p_5().rounded_md().h_10().w_full().bg(gpui::red()).flex().flex_col().child("Ethernet"))
//                     .child(div().text_color(rgb(0xffffff)).text_lg().items_center().justify_center().p_5().rounded_md().h_10().w_full().bg(gpui::green()).flex().flex_col().child("IP"))
//                     .child(div().text_color(rgb(0xffffff)).text_lg().items_center().justify_center().p_5().rounded_md().h_10().w_full().bg(gpui::blue()).flex().flex_col().child("TCP"))
//                     .child(div().text_color(rgb(0xffffff)).text_lg().items_center().justify_center().p_5().rounded_md().h_10().w_full().bg(gpui::red()).flex().flex_col().child("UDP"))
//                     .child(div().text_color(rgb(0xffffff)).text_lg().items_center().justify_center().p_5().rounded_md().h_10().w_full().bg(gpui::black()).flex().flex_col().child("ICMP"))
//                     .child(div().text_color(rgb(0xffffff)).text_lg().items_center().justify_center().p_5().rounded_md().h_10().w_full().bg(gpui::blue()).flex().flex_col().child("ARP")),
//             )
//     }
// }
 
fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);

	let dummy_packets = vec![
	    PacketView {
		view_name: "TCP".into(),
		packet_len: 64,
		more_info: "SOURCE 192.168.0.1 -> 41.132.32.10 TARGET".into(),
		timestamp: "2025.05.01 10:23:53".into(),
	    },
	    PacketView {
		view_name: "ETH".into(),
		packet_len: 128,
		more_info: "SOURCE 192.168.0.1 -> 41.132.32.10 TARGET".into(),
		timestamp: "2025.05.01 10:23:53".into(),
	    },
	    PacketView {
		view_name: "ETH".into(),
		packet_len: 42,
		more_info: "SOURCE 192.168.0.1 -> 41.132.32.10 TARGET".into(),
		timestamp: "2025.05.01 10:23:53".into(),
	    },
	    PacketView {
		view_name: "ARP".into(),
		packet_len: 15,
		more_info: "SOURCE 192.168.0.1 -> 41.132.32.10 TARGET".into(),
		timestamp: "2025.05.01 10:23:53".into(),
	    },
	    PacketView {
		view_name: "UDP".into(),
		packet_len: 54,
		more_info: "SOURCE 192.168.0.1 -> 41.132.32.10 TARGET".into(),
		timestamp: "2025.05.01 10:23:53".into(),
	    },
	    PacketView {
		view_name: "UDP".into(),
		packet_len: 75,
		more_info: "SOURCE 192.168.0.1 -> 41.132.32.10 TARGET".into(),
		timestamp: "2025.05.01 10:23:53".into(),
	    },
	    PacketView {
		view_name: "ICMP".into(),
		packet_len: 51,
		more_info: "SOURCE 192.168.0.1 -> 41.132.32.10 TARGET".into(),
		timestamp: "2025.05.01 10:23:53".into(),
	    },
	];
	
	cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| PacketList {
                    packets: dummy_packets,
                })
            },
        )
        .unwrap();
    });
}
