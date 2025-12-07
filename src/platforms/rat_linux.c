#include <rat.h>

int rat_cap_create(rat_cap_t* cap, const rat_device_t* device, void* user_data, uint32_t timeout) {
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

int rat_cap_loop_w(rat_cap_t* cap, rat_packet_t* pk, uint32_t packet_count) {
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

int rat_cap_loop(rat_cap_t* cap, rat_cap_cb cb, uint32_t packet_count) {
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
