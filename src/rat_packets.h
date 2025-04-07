#ifndef _RAT_PACKETS_H
#define _RAT_PACKETS_H

#include <sys/time.h>
#include <stdint.h>
#include <stddef.h>

#if defined(__GNUC__) || defined(__clang__)
#define PACKED __attribute__((packed))
#else
#define PACKED
#endif

#define RAT_HA_ADDR_LEN 6

typedef struct PACKED {
  uint8_t hardware_addr_octet[RAT_HA_ADDR_LEN];
} rat_hardware_addr_t;

typedef struct PACKED {
  uint8_t  ether_dhost[RAT_HA_ADDR_LEN];
  uint8_t  ether_shost[RAT_HA_ADDR_LEN];
  uint16_t ether_type;
} rat_ethernet_header_t;

typedef struct PACKED {
    uint16_t hardware_type;
    uint16_t protocol_type;
    uint8_t  hardware_addr_len;
    uint8_t  protocol_addr_len;
    uint16_t operation;
    uint8_t  sender_hardware_addr[RAT_HA_ADDR_LEN];
    uint8_t  sender_ip_addr[4];
    uint8_t  target_hardware_addr[RAT_HA_ADDR_LEN];
    uint8_t  target_ip_addr[4];
} rat_arp_header_t;

typedef struct {
    // packet info
    uint8_t* raw_data;
    size_t   length;
    struct timeval timestamp;

    // headers
    rat_ethernet_header_t*  eth;
    rat_arp_header_t* arp;

    // payload
    uint8_t* payload;
    size_t   payload_length;
} rat_packet_t;

void rat_packet_parse(rat_packet_t* packet);

#endif
