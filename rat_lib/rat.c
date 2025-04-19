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

void rat_cap_create(rat_cap_t* cap, const rat_device_t* device, void* user_data, uint32_t timeout) {
    cap->user_data = user_data;
    
    cap->sock_fd = socket(AF_PACKET, SOCK_RAW, htons(ETH_P_ALL));
    if (cap->sock_fd < 0) {
        perror("Rat Socket Error");
        exit(1);
    }
    
    if (device->name[0] != '\0' || device->mtu > 0) {
        memcpy(&cap->device, device, sizeof(rat_device_t));
        cap->buffer_size = device->mtu;
        
        int result = setsockopt(cap->sock_fd, SOL_SOCKET, SO_BINDTODEVICE, 
                               device->name, strlen(device->name));
        if (result != 0) {
            perror("Rat Setsockopt SO_BINDTODEVICE Error");
            exit(2);
        }
    } else {
        fprintf(stderr, "Rat Device Error: device not provided\n");
        close(cap->sock_fd);
        exit(3);
    }
    
    cap->timeout = timeout;
    if (timeout > 0) {
        struct timeval tv;
        tv.tv_sec = timeout / 1000;
        tv.tv_usec = (timeout % 1000) * 1000;
        if (setsockopt(cap->sock_fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv)) == -1) {
            perror("Rat Setsockopt SO_RCVTIMEO Error");
            exit(4);
        }
    }
}

int rat_cap_loop(rat_cap_t* cap, rat_cap_cb cb, uint32_t packet_count) {
    uint32_t packets_processed = 0;
    uint8_t buf[cap->buffer_size];
    memset(buf, 0, cap->buffer_size);

    while (packet_count <= 0 || packets_processed < packet_count) {
	memset(buf, 0, sizeof(buf));
	ssize_t packet_size = recvfrom(cap->sock_fd, buf, sizeof(buf), 0, NULL, NULL);

	if (packet_size < 0) {
	    if (errno == EAGAIN || errno == EWOULDBLOCK) {
                continue;
            }
            perror("Error receiving packet");
            return -1;
	}

	rat_packet_t packet = {0};
        packet.raw_data = buf;
        packet.length = packet_size;
        gettimeofday(&packet.timestamp, NULL);
        
        rat_packet_parse(&packet);
        
        cb(&packet, cap->user_data);
        
        packets_processed++;
    }

    return 0;
}

void rat_cap_destroy(rat_cap_t* cap) {
    close(cap->sock_fd);
    cap->buffer_size = 0;
}
