#ifndef _RAT_H
#define _RAT_H

#if defined(__FreeBSD__) || defined(__NetBSD__) || defined(__OpenBSD__) || defined(__DragonFly__)
#define RAT_BSD
#elif defined(__APPLE__)
#define RAT_MACOS
#elif defined(__linux__)
#define RAT_LINUX
#else
#error "Unsupported platform"
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/ioctl.h>
#include <arpa/inet.h>
#include <net/if.h>
#include <ifaddrs.h>

#if defined(RAT_BSD) || defined(RAT_MACOS)
#include <net/ethernet.h>
#include <net/bpf.h>
#endif

#ifdef RAT_LINUX
#include <netinet/ether.h>
#endif

#include <rat_packets.h>
#include <rat_utils.h>

#define RATAPI __attribute__((visibility("default")))

#define MAX_INTERFACES 32

typedef void (*rat_cap_cb)(rat_packet_t*, void*);

typedef struct {
    char name[IFNAMSIZ];
    uint32_t mtu;
} rat_device_t;

typedef struct {
    int32_t fd;
    rat_device_t device;
    uint32_t timeout;
    size_t buffer_size;
    void* user_data;
} rat_cap_t;

RATAPI int rat_device_lookup(rat_device_t devices[MAX_INTERFACES]);
RATAPI int rat_device_pick(rat_device_t devices[], size_t devices_len);
RATAPI int rat_cap_create(rat_cap_t* cap, const rat_device_t* device, void* user_data, uint32_t timeout);
RATAPI int rat_cap_loop(rat_cap_t* cap, rat_cap_cb cb, uint32_t packet_count);
RATAPI int rat_cap_loop_w(rat_cap_t* cap, rat_packet_t* pk, uint32_t packet_count);
RATAPI void rat_cap_destroy(rat_cap_t* cap);

#endif
