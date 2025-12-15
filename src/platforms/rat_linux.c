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

int rat_capture(rat_cap_t* cap, uint8_t* buf, rat_packet_t* pk, rat_cap_cb cb) {
    memset(buf, 0, cap->buffer_size);

	ssize_t packet_size = recvfrom(cap->fd, buf, sizeof(buf), 0, NULL, NULL);

	if (packet_size < 0) {
		if (errno == EAGAIN || errno == EWOULDBLOCK) {
            continue;
        }
        perror("Error receiving packet");
        return -1;
	}

	packet->raw_data = buf;
    packet->length = packet_size;

	gettimeofday(&pk->timestamp, NULL);
	rat_packet_parse(&(*pk));
	
	if (cb != NULL) cb(pk, cap->user_data);

    return 0;
}
