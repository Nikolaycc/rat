#include "../src/rat.h"

int main(void) {
    rat_device_t devices[MAX_INTERFACES];
    int devices_size = rat_device_lookup(devices);
    if (devices_size == 0) {
        printf("No network devices found\n");
        return 1;
    }

    for (int i = 0; i < devices_size; i++) {
        printf("%d: %s (MTU: %u)\n", i, devices[i].name, devices[i].mtu);
    }
    
    int device_idx = rat_device_pick(devices, devices_size);
    if (device_idx < 0) {
	printf("No network devices index found\n");
        return 1;
    }

    printf("device idx picked %d: %s\n", device_idx, devices[device_idx].name);
    
    rat_cap_t cap = {0};
    rat_cap_create(&cap, &devices[device_idx], NULL, 0);

    rat_cap_destroy(&cap);
    
    return 0;
}
