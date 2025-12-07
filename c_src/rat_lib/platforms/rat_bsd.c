#include <rat.h>

int rat_cap_create(rat_cap_t* cap, const rat_device_t* device, void* user_data, uint32_t timeout) {
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

int rat_cap_loop_w(rat_cap_t* cap, rat_packet_t* pk, uint32_t packet_count) {
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

int rat_cap_loop(rat_cap_t* cap, rat_cap_cb cb, uint32_t packet_count) {
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
