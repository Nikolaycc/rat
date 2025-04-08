# Rat

Basic network packet sniffer

> [!WARNING]
> This library is unfinished. Keep your expectations low.

## Building

```bash
 git clone https://github.com/nikolaycc/rat.git
 cd rat
 make
```

## Debug Build

```bash
 make DEBUG=1
```

## Usage

Basic Example

```c
#include "rat.h"

void cap_cb(rat_packet_t* packet, void* data) {
    (void)data;

    if (packet->eth) {
	// ....
    }

    if (packet->arp) {
	// ....
    }

    if (packet->ip) {
	// ....
    }

    if (packet->tcp) {
	// ....
    }

    if (packet->udp) {
	// ....
    }
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