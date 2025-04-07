#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>

#include <sys/socket.h>
#include <arpa/inet.h>
#include <net/ethernet.h>
#include <netinet/ether.h>
#include <netinet/ip.h>
#include <netinet/tcp.h>
#include <netinet/udp.h>
#include <net/if_arp.h>

#define PROTO_TCP IPPROTO_TCP
#define PROTO_UDP IPPROTO_UDP

struct arp_header {
    unsigned short int ar_hrd;		/* Format of hardware address.  */
    unsigned short int ar_pro;		/* Format of protocol address.  */
    unsigned char ar_hln;		/* Length of hardware address.  */
    unsigned char ar_pln;		/* Length of protocol address.  */
    unsigned short int ar_op;		/* ARP opcode (command).  */
    unsigned char __ar_sha[ETH_ALEN];	/* Sender hardware address.  */
    unsigned char __ar_sip[4];		/* Sender IP address.  */
    unsigned char __ar_tha[ETH_ALEN];	/* Target hardware address.  */
    unsigned char __ar_tip[4];		/* Target IP address.  */
};

int main(void) {
    int sock = socket(AF_PACKET, SOCK_RAW, htons(ETH_P_ALL));

    if (sock < 0) {
	perror("Error");
	exit(1);
    }

    const char* interface = "enp3s0";
    int result = setsockopt(sock, SOL_SOCKET, SO_BINDTODEVICE, interface, strlen(interface));
    if (result != 0) {
        perror("Error");
	exit(2);
    }

    unsigned char buf[1280] = {0};

    while (1) {
	memset(buf, 0, sizeof(buf));
	
	size_t n = recvfrom(sock, buf, sizeof(buf), 0, NULL, NULL);
	printf("\nPacket Size %zu\n", n);

	printf("Packet: \n");
	printf("  Ethernet:\n");
	const struct ether_header* ethhdr = (struct ether_header*)(buf);

	const char* ether_src_addr = ether_ntoa((struct ether_addr*)ethhdr->ether_shost);
	printf("\t  source_addr: %s\n", ether_src_addr);

	const char* ether_dst_addr = ether_ntoa((struct ether_addr*)ethhdr->ether_dhost);
	printf("\t  destination_addr: %s\n", ether_dst_addr);

	printf("\t  type: %d\n", ntohs(ethhdr->ether_type));

	if (ethhdr->ether_type == htons(ETHERTYPE_IP)) {
	    printf("\t  Internet Protocol:\n");
	    const struct iphdr* iphdr = (struct iphdr*)(buf + sizeof(struct ether_header));

	    const char* ip_src_addr = inet_ntoa((struct in_addr){iphdr->saddr});
	    printf("\t\t  source_addr: %s\n", ip_src_addr);

	    const char* ip_dst_addr = inet_ntoa((struct in_addr){iphdr->daddr});
	    printf("\t\t  destination_addr: %s\n", ip_dst_addr);

	    printf("\t\t  protocol: %d\n", iphdr->protocol);

	    int ip_header_len = iphdr->ihl * 4;
	    if (iphdr->protocol == PROTO_TCP) {
		const struct tcphdr* tcphdr = (struct tcphdr*)(buf + sizeof(struct ether_header) + ip_header_len);

		printf("\t\t  TCP:\n");
		printf("\t\t\t  source_port: %u\n", ntohs(tcphdr->source));
                printf("\t\t\t  destination_port: %u\n", ntohs(tcphdr->dest));
                printf("\t\t\t  sequence_number: %u\n", ntohl(tcphdr->seq));
                printf("\t\t\t  acknowledgment_number: %u\n", ntohl(tcphdr->ack_seq));
                printf("\t\t\t  data_offset: %u (bytes: %u)\n", tcphdr->doff, tcphdr->doff * 4);
                printf("\t\t\t  flags: ");
                if (tcphdr->syn) printf("SYN ");
                if (tcphdr->ack) printf("ACK ");
                if (tcphdr->fin) printf("FIN ");
                if (tcphdr->rst) printf("RST ");
                if (tcphdr->psh) printf("PSH ");
                if (tcphdr->urg) printf("URG ");
                printf("\n");
                printf("\t\t\t  window_size: %u\n", ntohs(tcphdr->window));
                printf("\t\t\t  checksum: 0x%04x\n", ntohs(tcphdr->check));
                printf("\t\t\t  urgent_pointer: %u\n", ntohs(tcphdr->urg_ptr));

		printf("\t\t\t  Raw TCP header bytes: ");
		for (size_t i = 0; i < sizeof(struct tcphdr); i++) {
		    printf("%02x ", buf[sizeof(struct ether_header) + ip_header_len + i]);
		}
		printf("\n");
	    } else if (iphdr->protocol == PROTO_UDP) {
		const struct udphdr* udphdr = (struct udphdr*)(buf + sizeof(struct ether_header) + ip_header_len);

		printf("\t\t UDP:\n");
		printf("\t\t\t  source_port: %u\n", ntohs(udphdr->source));
                printf("\t\t\t  destination_port: %u\n", ntohs(udphdr->dest));
                printf("\t\t\t  length: %u\n", ntohs(udphdr->len));
                printf("\t\t\t  checksum: 0x%04x\n", ntohs(udphdr->check));

		int payload_len = ntohs(udphdr->len) - sizeof(struct udphdr);
		printf("UDP payload (%d bytes): ", payload_len);
		for (int i = 0; i < payload_len && i < 16; i++) {
		    printf("%02x ", buf[sizeof(struct ether_header) + ip_header_len + sizeof(struct udphdr) + i]);
		}
		printf("\n");
	    }
	} else if (ethhdr->ether_type == htons(ETHERTYPE_ARP)) {
	    const struct arp_header* arphdr = (struct arp_header*)(buf + sizeof(struct ether_header));

	    const char* arp_sender_mac = ether_ntoa((struct ether_addr*)arphdr->__ar_sha);
	    printf("arp sender mac addr: %s\n", arp_sender_mac);
	}
    }
    
    close(sock);
    
    return 0;
}
