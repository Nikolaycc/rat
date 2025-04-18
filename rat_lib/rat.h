#ifndef _RAT_H
#define _RAT_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>

#include <arpa/inet.h>
#include <ifaddrs.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <net/if.h>
#include <net/ethernet.h>

#include "rat_packets.h"
#include "rat_utils.h"

#define MAX_INTERFACES 32

typedef void (*rat_cap_cb)(rat_packet_t*, void*);

typedef struct {
    char name[IFNAMSIZ];
    uint32_t mtu;
} rat_device_t;

typedef struct {
    int32_t sock_fd;
    rat_device_t device;
    uint32_t timeout;
    size_t buffer_size;
    void* user_data;
} rat_cap_t;

int      rat_device_lookup(rat_device_t devices[MAX_INTERFACES]);
int      rat_device_pick(rat_device_t devices[], size_t devices_len);
void     rat_cap_create(rat_cap_t* cap, const rat_device_t* device, void* user_data, uint32_t timeout);
int      rat_cap_loop(rat_cap_t* cap, rat_cap_cb cb, uint32_t packet_count);
void     rat_cap_destroy(rat_cap_t* cap);

#endif
