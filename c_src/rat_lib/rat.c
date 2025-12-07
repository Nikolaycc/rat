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

#if defined(RAT_BSD) || defined(RAT_MACOS)
int __rat_cap_create_bsd_macos(rat_cap_t* cap, const rat_device_t* device, void* user_data, uint32_t timeout) {
	cap->user_data = user_data;

	cap->fd = open("/dev/bpf0", O_RDWR);
    if (cap->fd == -1) {
        perror("Rat open /dev/bpf Error");
        return -1;
    }

	if (device->name[0] != '\0' || device->mtu > 0) {
		uint32_t buflen;
		
		if (ioctl(cap->fd, BIOCGBLEN, &buflen) < 0) {
			perror("Rat BIOCGBLEN Error");
			return -5;
		}

		cap->buffer_size = buflen;
		
		struct ifreq ifr;
		strncpy(ifr.ifr_name, device->name, IFNAMSIZ); // Replace "em0" with your interface
		if (ioctl(cap->fd, BIOCSETIF, &ifr) == -1) {
			perror("Rat ioctl BIOCSETIF Error");
			return -2;
		}
	} else {
        fprintf(stderr, "Rat Device Error: device not provided\n");
        close(cap->fd);
        return -3;
    }

	cap->timeout = timeout;
	if (timeout > 0) {
		struct timeval tm;
		tm.tv_sec = timeout / 1000;
		tm.tv_usec = (timeout % 1000) * 1000;
		if (ioctl(cap->fd, BIOCSRTIMEOUT, &tm) == -1) {
			perror("Rat ioctl BIOCSRTIMEOUT Error");
			return -4;
		}
	}
	
	return 0;
}
#endif

#if defined(RAT_LINUX)
int __rat_cap_create_linux(rat_cap_t* cap, const rat_device_t* device, void* user_data, uint32_t timeout) {
    cap->user_data = user_data;
    
    cap->fd = socket(AF_PACKET, SOCK_RAW, htons(ETH_P_ALL));
    if (cap->fd < 0) {
        perror("Rat Socket Error");
        return -1;
    }
    
    if (device->name[0] != '\0' || device->mtu > 0) {
        memcpy(&cap->device, device, sizeof(rat_device_t));
        cap->buffer_size = device->mtu;
        
        int result = setsockopt(cap->fd, SOL_SOCKET, SO_BINDTODEVICE, 
        device->name, strlen(device->name));
        if (result != 0) {
            perror("Rat Setsockopt SO_BINDTODEVICE Error");
            return -2;
        }
    } else {
        fprintf(stderr, "Rat Device Error: device not provided\n");
        close(cap->fd);
        return -3;
    }
    
    cap->timeout = timeout;
    if (timeout > 0) {
        struct timeval tv;
        tv.tv_sec = timeout / 1000;
        tv.tv_usec = (timeout % 1000) * 1000;
        if (setsockopt(cap->fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv)) == -1) {
            perror("Rat Setsockopt SO_RCVTIMEO Error");
            return -4;
        }
    }

    return 0;
}
#endif

int rat_cap_create(rat_cap_t* cap, const rat_device_t* device, void* user_data, uint32_t timeout) {
    #if defined(RAT_BSD) || defined(RAT_MACOS)
	return __rat_cap_create_bsd_macos(cap, device, user_data, timeout);
	#elif RAT_LINUX
	return __rat_cap_create_linux(cap, device, user_data, timeout);
	#else
	#error "Unsupported platform"
	#endif
	
}

#if defined(RAT_BSD) || defined(RAT_MACOS)
int __rat_cap_loop_w_bsd_macos(rat_cap_t* cap, rat_packet_t* pk, uint32_t packet_count) {
	uint32_t packets_processed = 0;
	uint8_t buf[cap->buffer_size];
	
    while (packet_count <= 0 || packets_processed < packet_count) {
		memset(buf, 0, sizeof(buf));

		ssize_t n = read(cap->fd, buf, cap->buffer_size);
        if (n < 0) {
            perror("Rat read Error");
            break;
        }
        
        u_char *ptr = buf;
		while (ptr < buf + n) {
            struct bpf_hdr *bh = (struct bpf_hdr *)ptr;
            
			rat_packet_t packet = {0};
			packet.raw_data = ptr + bh->bh_hdrlen;
			packet.length = bh->bh_datalen;

			gettimeofday(&packet.timestamp, NULL);
			rat_packet_parse(&packet);

			*pk = packet;
			
			packets_processed++;
			
            // Move to next packet (BPF_WORDALIGN rounds up to word boundary)
            ptr += BPF_WORDALIGN(bh->bh_hdrlen + bh->bh_caplen);
        }
    }

    return 0;
}
#endif

#if defined(RAT_LINUX)
int __rat_cap_loop_w_linux(rat_cap_t* cap, rat_packet_t* pk, uint32_t packet_count) {
    uint32_t packets_processed = 0;
    uint8_t buf[cap->buffer_size];
    memset(buf, 0, cap->buffer_size);

    while (packet_count <= 0 || packets_processed < packet_count) {
		memset(buf, 0, sizeof(buf));
		ssize_t packet_size = recvfrom(cap->fd, buf, sizeof(buf), 0, NULL, NULL);

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

		*pk = packet;
        
        packets_processed++;
    }

    return 0;
}
#endif

int rat_cap_loop_w(rat_cap_t* cap, rat_packet_t* pk, uint32_t packet_count) {
	#if defined(RAT_BSD) || defined(RAT_MACOS)
	return __rat_cap_loop_w_bsd_macos(cap, pk, packet_count);
	#elif defined(RAT_MACOS)
	return __rat_cap_loop_w_linux(cap, pk, packet_count);
	#else
	#error "Unsupported platform"
	#endif
}

#if defined(RAT_BSD) || defined(RAT_MACOS)
int __rat_cap_loop_bsd_macos(rat_cap_t* cap, rat_cap_cb cb, uint32_t packet_count) {
    uint32_t packets_processed = 0;
    uint8_t buf[cap->buffer_size];

    while (packet_count <= 0 || packets_processed < packet_count) {
		memset(buf, 0, sizeof(buf));

		ssize_t n = read(cap->fd, buf, cap->buffer_size);
        if (n < 0) {
            perror("Rat read Error");
            break;
        }
        
        u_char *ptr = buf;
        while (ptr < buf + n) {
            struct bpf_hdr *bh = (struct bpf_hdr *)ptr;
            
			rat_packet_t packet = {0};
			packet.raw_data = ptr + bh->bh_hdrlen;
			packet.length = bh->bh_datalen;

			gettimeofday(&packet.timestamp, NULL);
			
			rat_packet_parse(&packet);
			cb(&packet, cap->user_data);
			
			packets_processed++;
			
            // Move to next packet (BPF_WORDALIGN rounds up to word boundary)
            ptr += BPF_WORDALIGN(bh->bh_hdrlen + bh->bh_caplen);
        }

    }

    return 0;
}
#endif

#if defined(RAT_LINUX)
int __rat_cap_loop_linux(rat_cap_t* cap, rat_cap_cb cb, uint32_t packet_count) {
    uint32_t packets_processed = 0;
    uint8_t buf[cap->buffer_size];
    memset(buf, 0, cap->buffer_size);

    while (packet_count <= 0 || packets_processed < packet_count) {
		memset(buf, 0, sizeof(buf));
		ssize_t packet_size = recvfrom(cap->fd, buf, sizeof(buf), 0, NULL, NULL);

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
#endif

int rat_cap_loop(rat_cap_t* cap, rat_cap_cb cb, uint32_t packet_count) {
	#if defined(RAT_BSD) || defined(RAT_MACOS)
	return __rat_cap_loop_bsd_macos(cap, cb, packet_count);
	#elif defined(RAT_LINUX)
	return __rat_cap_loop_linux(cap, cb, packet_count);
	#else
	#error "Unsupported platform"
	#endif
}

void rat_cap_destroy(rat_cap_t* cap) {
    close(cap->fd);
    cap->buffer_size = 0;
}
