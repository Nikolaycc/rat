# Rat

Basic network packet sniffer

> [!WARNING]
> This library is unfinished. Keep your expectations low.

## Building

Dependencies

* meson (build system)
* ninja
* libreadline-dev (for CLI)

```bash
 git clone https://github.com/nikolaycc/rat.git
 cd rat
 
 meson setup build
 cd build
 ninja
```

## Command Line Interface

```bash
$ ./rat_cli
Welcome to RAT Sniffer CLI. Type 'help' for commands.
rat> list
Available interfaces:
0: eth0 (Ethernet interface)
1: wlan0 (Wireless interface)
rat> start 0
Started sniffing on eth0
[Packet display appears here]
rat> stop
Stopped sniffing
rat> exit
```

## Usage (rat_lib)

Basic Example

```c
#include <rat.h>

void packet_handler(rat_packet_t* packet, void* data) {
	// Process packet here
}

int main(void) {
    rat_device_t devices[MAX_INTERFACES];
    int devices_size = rat_device_lookup(devices);
    if (devices_size == 0) {
        printf("No network devices found\n");
        return 1;
    }
    
    int device_idx = rat_device_pick(devices, devices_size);
    if (device_idx < 0) {
	printf("No network devices index found\n");
        return 1;
    }

    rat_cap_t cap = {0};
    rat_cap_create(&cap, &devices[device_idx], NULL, 0);
    
    rat_cap_loop(&cap, cap_cb, 2);
    
    rat_cap_destroy(&cap);
    
    return 0;
}
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## Roadmap

*   Add IPv6 support
*   Add ICMP support
*   Add BPF filter support
*   Add Windows compatibility
