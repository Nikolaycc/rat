#include <rat.h>

int rat_device_lookup(rat_device_t devices[MAX_INTERFACES]) {
    struct ifaddrs *ifaddr, *ifa;
    int count = 0;
    
    if (getifaddrs(&ifaddr) == -1) {
        perror("Rat getifaddrs Error");
        return -1;
    }
    
    int sock = socket(AF_INET, SOCK_DGRAM, 0);
    if (sock == -1) {
        perror("Rat Socket Error");
        freeifaddrs(ifaddr);
        return -1;
    }
    
    
    for (ifa = ifaddr; ifa != NULL && count < MAX_INTERFACES; ifa = ifa->ifa_next) {
		if (ifa->ifa_addr == NULL) continue;
		
		int is_duplicate = 0;
		for (int i = 0; i < count; i++) {
            if (strcmp(devices[i].name, ifa->ifa_name) == 0) {
				is_duplicate = 1;
				break;
            }
		}
		if (is_duplicate) continue;
		
		if (count >= MAX_INTERFACES) {
            fprintf(stderr, "Warning: Too many interfaces found, stopping at %d\n", MAX_INTERFACES);
            break;
		}

		strncpy(devices[count].name, ifa->ifa_name, IFNAMSIZ - 1);
		devices[count].name[IFNAMSIZ - 1] = '\0'; 

		struct ifreq ifr;
		memset(&ifr, 0, sizeof(ifr));
		strncpy(ifr.ifr_name, ifa->ifa_name, IFNAMSIZ - 1);
		if (ioctl(sock, SIOCGIFMTU, &ifr) == -1) {
            devices[count].mtu = 0;
		} else {
            devices[count].mtu = ifr.ifr_mtu;
		}
		
		count++;
    }
    
    close(sock);
    freeifaddrs(ifaddr);

    return count;
}

int rat_device_pick(rat_device_t devices[], size_t devices_len) {
    if (devices_len == 0) return -1;
    
    int best_score = -1;
    int best_index = 0;
    int sock = socket(AF_INET, SOCK_DGRAM, 0);
    if (sock == -1) return 0;
    
    for (size_t i = 0; i < devices_len; i++) {
        rat_device_t* dev = &devices[i];
        int score = 0;
        struct ifreq ifr;
        memset(&ifr, 0, sizeof(ifr));
        strncpy(ifr.ifr_name, dev->name, IFNAMSIZ - 1);
        
        if (ioctl(sock, SIOCGIFFLAGS, &ifr) == -1) continue;
        
        if (ifr.ifr_flags & IFF_UP) score += 100;
        if (!(ifr.ifr_flags & IFF_LOOPBACK)) score += 50;
        if (dev->mtu > 0) score += dev->mtu / 100;
        
        if (score > best_score) {
            best_score = score;
            best_index = i;
        }
    }
    
    close(sock);
    return best_index;
}

int rat_cap_loop_w(rat_cap_t* cap, rat_packet_t* pk, uint32_t packet_count) {
	uint32_t packets_processed = 0;
	uint8_t buf[cap->buffer_size];
	
    while (packet_count <= 0 || packets_processed < packet_count) {
		rat_capture(cap, buf, pk, NULL);
		packets_processed++;	
    }

    return 0;
}

int rat_cap_loop(rat_cap_t* cap, rat_cap_cb cb, uint32_t packet_count) {
    uint32_t packets_processed = 0;
    uint8_t buf[cap->buffer_size];

    while (packet_count <= 0 || packets_processed < packet_count) {
		memset(buf, 0, sizeof(buf));

		rat_packet_t packet = {0};

		rat_capture(cap, buf, &packet, cb);
			
		packets_processed++;
    }

    return 0;
}

void rat_cap_destroy(rat_cap_t* cap) {
    close(cap->fd);
    cap->buffer_size = 0;
}
