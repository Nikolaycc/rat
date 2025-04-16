#include "rat_packets.h"

void __rat_packet_parse(rat_packet_t* packet, uint8_t header_type, size_t remaining) {
    uint8_t* current_data = packet->raw_data;
    
    #if defined(DEBUG)
    printf("DEBUG: Starting parse with header_type=%d, remaining=%zu\n", header_type, remaining);
    #endif
    
    if (header_type != RAT_ETH_PACKET_TYPE) {
        if (packet->eth) current_data += sizeof(rat_ethernet_header_t);
        if (packet->arp) current_data += sizeof(rat_arp_header_t);
        
        if (packet->ip) {
            uint8_t ihl = (packet->ip->version_ihl & 0x0F);
            current_data += ihl * 4;
        }
        
        if (packet->tcp) {
            uint8_t data_offset = (packet->tcp->data_offset_reserved >> 4);
            current_data += data_offset * 4;
        }
        if (packet->udp) current_data += sizeof(rat_udp_header_t);
    }

    #if defined(DEBUG)
    printf("DEBUG: Current data offset: %td bytes from start\n", current_data - packet->raw_data);
    #endif
    
    switch (header_type) {
    case RAT_ETH_PACKET_TYPE:
        #if defined(DEBUG)
        printf("DEBUG: Parsing Ethernet header\n");
        #endif
        
        if (remaining >= sizeof(rat_ethernet_header_t)) {
            packet->eth = (rat_ethernet_header_t*)current_data;
            remaining -= sizeof(rat_ethernet_header_t);
            uint16_t ether_type = ntohs(packet->eth->ether_type);
            uint8_t next_type = RAT_DONE;
            
            #if defined(DEBUG)
            printf("DEBUG: Ethernet type: 0x%04x\n", ether_type);
            printf("DEBUG: Source MAC: %02x:%02x:%02x:%02x:%02x:%02x\n",
                   packet->eth->ether_shost[0], packet->eth->ether_shost[1],
                   packet->eth->ether_shost[2], packet->eth->ether_shost[3], 
                   packet->eth->ether_shost[4], packet->eth->ether_shost[5]);
            printf("DEBUG: Dest MAC: %02x:%02x:%02x:%02x:%02x:%02x\n",
                   packet->eth->ether_dhost[0], packet->eth->ether_dhost[1],
                   packet->eth->ether_dhost[2], packet->eth->ether_dhost[3], 
                   packet->eth->ether_dhost[4], packet->eth->ether_dhost[5]);
            #endif
            
            if (ether_type == 0x0800) {
                next_type = RAT_IP_PACKET_TYPE;
                #if defined(DEBUG)
                printf("DEBUG: Next protocol is IPv4\n");
                #endif
            } else if (ether_type == 0x0806) {
                next_type = RAT_ARP_PACKET_TYPE;
                #if defined(DEBUG)
                printf("DEBUG: Next protocol is ARP\n");
                #endif
            }
            
            __rat_packet_parse(packet, next_type, remaining);
        } else {
            #if defined(DEBUG)
            printf("DEBUG: Not enough data for Ethernet header\n");
            #endif
            __rat_packet_parse(packet, RAT_DONE, 0);
        }
        break;
        
    case RAT_ARP_PACKET_TYPE:
        #if defined(DEBUG)
        printf("DEBUG: Parsing ARP header\n");
        #endif
        
        if (remaining >= sizeof(rat_arp_header_t)) {
            packet->arp = (rat_arp_header_t*)current_data;
            remaining -= sizeof(rat_arp_header_t);
            
            #if defined(DEBUG)
            printf("DEBUG: ARP Operation: %d\n", ntohs(packet->arp->operation));
            printf("DEBUG: Sender IP: %d.%d.%d.%d\n",
                   packet->arp->sender_ip_addr[0], packet->arp->sender_ip_addr[1],
                   packet->arp->sender_ip_addr[2], packet->arp->sender_ip_addr[3]);
            printf("DEBUG: Target IP: %d.%d.%d.%d\n",
                   packet->arp->target_ip_addr[0], packet->arp->target_ip_addr[1],
                   packet->arp->target_ip_addr[2], packet->arp->target_ip_addr[3]);
            #endif
            
            __rat_packet_parse(packet, RAT_DONE, remaining);
        } else {
            #if defined(DEBUG)
            printf("DEBUG: Not enough data for ARP header\n");
            #endif
            __rat_packet_parse(packet, RAT_DONE, 0);
        }
        break;
        
    case RAT_IP_PACKET_TYPE:
        #if defined(DEBUG)
        printf("DEBUG: Parsing IP header\n");
        #endif
        
        if (remaining >= sizeof(rat_ip_header_t)) {
            packet->ip = (rat_ip_header_t*)current_data;
            
            uint8_t ihl = (packet->ip->version_ihl & 0x0F);
            size_t ip_header_size = ihl * 4;
            
            #if defined(DEBUG)
            printf("DEBUG: IP version: %d, header length: %ld bytes\n", 
                   (packet->ip->version_ihl >> 4), ip_header_size);
            printf("DEBUG: Source IP: %d.%d.%d.%d\n",
                   (packet->ip->src_addr >> 0) & 0xFF, (packet->ip->src_addr >> 8) & 0xFF,
                   (packet->ip->src_addr >> 16) & 0xFF, (packet->ip->src_addr >> 24) & 0xFF);
            printf("DEBUG: Dest IP: %d.%d.%d.%d\n",
                   (packet->ip->dst_addr >> 0) & 0xFF, (packet->ip->dst_addr >> 8) & 0xFF,
                   (packet->ip->dst_addr >> 16) & 0xFF, (packet->ip->dst_addr >> 24) & 0xFF);
            printf("DEBUG: Protocol: %d\n", packet->ip->protocol);
            #endif
            
            if (remaining < ip_header_size) {
                #if defined(DEBUG)
                printf("DEBUG: IP header size exceeds remaining data\n");
                #endif
                __rat_packet_parse(packet, RAT_DONE, 0);
                break;
            }
            
            remaining -= ip_header_size;
            
            uint8_t next_type = RAT_DONE;
            switch (packet->ip->protocol) {
            case 6:
                next_type = RAT_TCP_PACKET_TYPE;
                #if defined(DEBUG)
                printf("DEBUG: Next protocol is TCP\n");
                #endif
                break;
            case 17:
                next_type = RAT_UDP_PACKET_TYPE;
                #if defined(DEBUG)
                printf("DEBUG: Next protocol is UDP\n");
                #endif
                break;
            default:
                #if defined(DEBUG)
                printf("DEBUG: Unsupported IP protocol: %d\n", packet->ip->protocol);
                #endif
                break;
            }
            
            __rat_packet_parse(packet, next_type, remaining);
        } else {
            #if defined(DEBUG)
            printf("DEBUG: Not enough data for IP header\n");
            #endif
            __rat_packet_parse(packet, RAT_DONE, 0);
        }
        break;
        
    case RAT_TCP_PACKET_TYPE:
        #if defined(DEBUG)
        printf("DEBUG: Parsing TCP header\n");
        #endif
        
        if (remaining >= sizeof(rat_tcp_header_t)) {
            packet->tcp = (rat_tcp_header_t*)current_data;
            
            uint8_t data_offset = (packet->tcp->data_offset_reserved >> 4);
            size_t tcp_header_size = data_offset * 4;
            
            #if defined(DEBUG)
            printf("DEBUG: TCP source port: %d\n", ntohs(packet->tcp->src_port));
            printf("DEBUG: TCP dest port: %d\n", ntohs(packet->tcp->dst_port));
            printf("DEBUG: TCP seq num: %u\n", ntohl(packet->tcp->seq_num));
            printf("DEBUG: TCP flags: 0x%02x\n", packet->tcp->flags);
            printf("DEBUG: TCP header length: %ld bytes\n", tcp_header_size);
            #endif
            
            if (remaining < tcp_header_size) {
                #if defined(DEBUG)
                printf("DEBUG: TCP header size exceeds remaining data\n");
                #endif
                __rat_packet_parse(packet, RAT_DONE, 0);
                break;
            }
            
            remaining -= tcp_header_size;
            __rat_packet_parse(packet, RAT_DONE, remaining);
        } else {
            #if defined(DEBUG)
            printf("DEBUG: Not enough data for TCP header\n");
            #endif
            __rat_packet_parse(packet, RAT_DONE, 0);
        }
        break;
        
    case RAT_UDP_PACKET_TYPE:
        #if defined(DEBUG)
        printf("DEBUG: Parsing UDP header\n");
        #endif
        
        if (remaining >= sizeof(rat_udp_header_t)) {
            packet->udp = (rat_udp_header_t*)current_data;
            remaining -= sizeof(rat_udp_header_t);
            
            #if defined(DEBUG)
            printf("DEBUG: UDP source port: %d\n", ntohs(packet->udp->src_port));
            printf("DEBUG: UDP dest port: %d\n", ntohs(packet->udp->dst_port));
            printf("DEBUG: UDP length: %d\n", ntohs(packet->udp->length));
            #endif
            
            __rat_packet_parse(packet, RAT_DONE, remaining);
        } else {
            #if defined(DEBUG)
            printf("DEBUG: Not enough data for UDP header\n");
            #endif
            __rat_packet_parse(packet, RAT_DONE, 0);
        }
        break;
        
    case RAT_DONE:
        #if defined(DEBUG)
        printf("DEBUG: Finished parsing, payload size: %zu bytes\n", remaining);
        #endif
        
        packet->payload = current_data;
        packet->payload_length = remaining;
        break;
        
    default:
        #if defined(DEBUG)
        printf("DEBUG: Unknown header type: %d\n", header_type);
        #endif
        printf("Rat Error %s(): unknown header type %d\n", __func__, header_type);
        return;
    }
}
