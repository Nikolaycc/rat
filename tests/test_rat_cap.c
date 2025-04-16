#include "../rat_lib/rat.h"

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
