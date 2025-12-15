#ifndef _RAT_PACKETS_H
#define _RAT_PACKETS_H

#include <sys/time.h>
#include <stdio.h>
#include <stdint.h>
#include <stddef.h>

#include <arpa/inet.h>

#define RATAPI __attribute__((visibility("default")))

#if defined(__GNUC__) || defined(__clang__)
#define PACKED __attribute__((packed))
#else
#define PACKED
#endif

#define RAT_HA_ADDR_LEN     6

#define RAT_ETH_PACKET_TYPE 0
#define RAT_ARP_PACKET_TYPE 1
#define RAT_IP_PACKET_TYPE  2
#define RAT_TCP_PACKET_TYPE 3
#define RAT_UDP_PACKET_TYPE 4
#define RAT_DONE            69

#define rat_packet_parse(PACKET) __rat_packet_parse(PACKET, RAT_ETH_PACKET_TYPE, *(PACKET.length));

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

typedef struct PACKED {
    uint8_t  version_ihl;
    uint8_t  tos;
    uint16_t total_length;
    uint16_t id;
    uint16_t flags_fragment;
    uint8_t  ttl;
    uint8_t  protocol;
    uint16_t checksum;
    uint32_t src_addr;
    uint32_t dst_addr;
} rat_ip_header_t;

typedef struct PACKED {
    uint16_t src_port;
    uint16_t dst_port;
    uint32_t seq_num;
    uint32_t ack_num;
    uint8_t  data_offset_reserved;
    uint8_t  flags;
    uint16_t window_size;
    uint16_t checksum;
    uint16_t urgent_ptr;
} rat_tcp_header_t;

typedef struct PACKED {
    uint16_t src_port;
    uint16_t dst_port;
    uint16_t length;
    uint16_t checksum;
} rat_udp_header_t;

typedef struct {
    // packet info
    uint8_t* raw_data;
    size_t   length;
    struct   timeval timestamp;

    // headers
    rat_ethernet_header_t* eth;
    rat_arp_header_t*      arp;
    rat_ip_header_t*       ip;
    rat_tcp_header_t*      tcp;
    rat_udp_header_t*      udp;
    
    // payload
    uint8_t* payload;
    size_t   payload_length;
} rat_packet_t;

// TODO: make better data structure

// typedef struct {
// 	void* header;
// 	uint8_t type;
// } rat_header;

// typedef struct {
// 	rat_header* headers;
// 	size_t count;
// 	size_t capacity;
// } rat_headers;

// typedef struct {
//     // packet info
//     uint8_t* raw_data;
//     size_t   length;
//     struct   timeval timestamp;

//     // headers
// 	   rat_headers* hdrs;
	
//     // payload
//     uint8_t* payload;
//     size_t   payload_length;
// } rat_packet_t;

void __rat_packet_parse(rat_packet_t*, uint8_t, size_t);

#endif
