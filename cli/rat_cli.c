#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <readline/readline.h>
#include <readline/history.h>
#include <assert.h>
#include <unistd.h>
#include <signal.h>
#include <uv.h>

#include <rat.h>

#ifndef RAT_VERSION
#define RAT_VERSION "unknown"
#endif

const char HISTORY_NAME[] = ".rat_history";
char history_path[1024];

typedef struct {
    rat_cap_t cap;
    rat_device_t devices[MAX_INTERFACES];
    int devices_size;
    int running;
    int selected_idx;
    uv_loop_t* loop;
    uv_poll_t poll_handle;
    uv_idle_t idle_handle;
    int readline_ready;
    char* pending_input;
} sniffer_state_t;

sniffer_state_t sniffer = {0};

void cleanup() {
    write_history(history_path);
    if (sniffer.running) {
        rat_cap_destroy(&sniffer.cap);
    }
    if (sniffer.pending_input) {
        free(sniffer.pending_input);
    }
}

void print_packet(rat_packet_t* packet) {
    printf("\n[%ld.%06d] Packet captured (%zu bytes)\n", 
           packet->timestamp.tv_sec, 
           packet->timestamp.tv_usec,
           packet->length);
    
    if (packet->eth) {
        printf("  Ethernet: ");
        printf("%02x:%02x:%02x:%02x:%02x:%02x -> ",
               packet->eth->ether_shost[0], packet->eth->ether_shost[1],
               packet->eth->ether_shost[2], packet->eth->ether_shost[3],
               packet->eth->ether_shost[4], packet->eth->ether_shost[5]);
        printf("%02x:%02x:%02x:%02x:%02x:%02x ",
               packet->eth->ether_dhost[0], packet->eth->ether_dhost[1],
               packet->eth->ether_dhost[2], packet->eth->ether_dhost[3],
               packet->eth->ether_dhost[4], packet->eth->ether_dhost[5]);
        printf("(type: 0x%04x)\n", ntohs(packet->eth->ether_type));
    }

    if (packet->arp) {
        printf("  ARP: ");
        printf("%d.%d.%d.%d -> %d.%d.%d.%d ",
               packet->arp->sender_ip_addr[0], packet->arp->sender_ip_addr[1],
               packet->arp->sender_ip_addr[2], packet->arp->sender_ip_addr[3],
               packet->arp->target_ip_addr[0], packet->arp->target_ip_addr[1],
               packet->arp->target_ip_addr[2], packet->arp->target_ip_addr[3]);
        printf("(op: %d)\n", ntohs(packet->arp->operation));
    }

    if (packet->ip) {
        struct in_addr src_addr = {packet->ip->src_addr};
        struct in_addr dst_addr = {packet->ip->dst_addr};
        printf("  IP: %s -> %s ", 
               inet_ntoa(src_addr), 
               inet_ntoa(dst_addr));
        printf("(proto: %d, ttl: %d)\n", packet->ip->protocol, packet->ip->ttl);
    }

    if (packet->tcp) {
        printf("  TCP: %d -> %d ",
               ntohs(packet->tcp->src_port),
               ntohs(packet->tcp->dst_port));
        printf("(seq: %u, ack: %u, flags: 0x%02x)\n",
               ntohl(packet->tcp->seq_num),
               ntohl(packet->tcp->ack_num),
               packet->tcp->flags);
    }

    if (packet->udp) {
        printf("  UDP: %d -> %d (len: %d)\n",
               ntohs(packet->udp->src_port),
               ntohs(packet->udp->dst_port),
               ntohs(packet->udp->length));
    }

    if (packet->payload_length > 0) {
        printf("  Payload: %zu bytes\n", packet->payload_length);
    }
}

void start_sniffing(int device_idx);
void stop_sniffing();

void on_packet_ready(uv_poll_t* handle, int status, int events) {
    if (status < 0) {
        fprintf(stderr, "Poll error: %s\n", uv_strerror(status));
        return;
    }

    if (events & UV_READABLE) {
        rat_packet_t packet = {0};
        
        int result = rat_cap_loop_w(&sniffer.cap, &packet, 1);
        
        if (result == 0 && packet.raw_data) {
            print_packet(&packet);
            
            if (sniffer.selected_idx >= 0) {
                char prompt[120];
                snprintf(prompt, sizeof(prompt), "rat[sniffing %s]> ", 
                        sniffer.devices[sniffer.selected_idx].name);
                rl_forced_update_display();
            }
        }
    }
}

void start_sniffing(int device_idx) {
    if (sniffer.running) {
        printf("Sniffer is already running\n");
        return;
    }

    if (device_idx < 0 || device_idx >= sniffer.devices_size) {
        printf("Invalid device index\n");
        return;
    }

    sniffer.selected_idx = device_idx;
    
    int result = rat_cap_create(&sniffer.cap, &sniffer.devices[device_idx], NULL, 0);
    if (result < 0) {
        printf("Failed to create capture\n");
        sniffer.selected_idx = -1;
        return;
    }

    sniffer.running = 1;
    
    uv_poll_init(sniffer.loop, &sniffer.poll_handle, sniffer.cap.fd);
    sniffer.poll_handle.data = &sniffer;
    
    uv_poll_start(&sniffer.poll_handle, UV_READABLE, on_packet_ready);
    
    printf("Started sniffing on %s\n", sniffer.devices[device_idx].name);
}

void stop_sniffing() {
    if (!sniffer.running) {
        printf("Sniffer is not running\n");
        return;
    }

    uv_poll_stop(&sniffer.poll_handle);
    uv_close((uv_handle_t*)&sniffer.poll_handle, NULL);
    
    sniffer.selected_idx = -1;
    rat_cap_destroy(&sniffer.cap);
    sniffer.running = 0;
    
    printf("Stopped sniffing\n");
}

void list_interfaces() {
    printf("Available interfaces:\n");
    for (int i = 0; i < sniffer.devices_size; i++) {
        printf("%d: %s (MTU: %d)\n", i, sniffer.devices[i].name, sniffer.devices[i].mtu);
    }
}

void process_command(char* input) {
    char* cmd = strtok(input, " ");
    if (!cmd) return;
    
    char* arg = strtok(NULL, " ");

    if (strcmp(cmd, "start") == 0) {
        if (!arg) {
            printf("Usage: start <interface_index>\n");
            return;
        }
        start_sniffing(atoi(arg));
    }
    else if (strcmp(cmd, "stop") == 0) {
        stop_sniffing();
    }
    else if (strcmp(cmd, "list") == 0) {
        list_interfaces();
    }
    else if (strcmp(cmd, "help") == 0) {
        printf("Available commands:\n");
        printf("  list               - List available interfaces\n");
        printf("  start <interface>  - Start sniffing on interface\n");
        printf("  stop               - Stop sniffing\n");
        printf("  help               - Show this help message\n");
        printf("  exit               - Exit program\n");
    }
    else {
        printf("Unknown command '%s'. Type 'help' for commands.\n", cmd);
    }
}

void on_idle(uv_idle_t* handle) {
    if (rl_pending_input) {
        sniffer.readline_ready = 1;
    }
    
    if (sniffer.pending_input) {
        char* input = sniffer.pending_input;
        sniffer.pending_input = NULL;
        
        if (*input) {
            add_history(input);
            process_command(input);
        }
        
        free(input);

		rl_on_new_line();
        rl_redisplay();
    }
}

void readline_handler(char* line) {
    if (!line) {
        printf("\n");
        uv_stop(sniffer.loop);
        return;
    }
    
    if (strcmp(line, "exit") == 0) {
        free(line);
        uv_stop(sniffer.loop);
        return;
    }
    
    sniffer.pending_input = line;
}

void stdin_read_cb(uv_poll_t* handle, int status, int events) {
    if (status < 0) {
        fprintf(stderr, "Stdin poll error: %s\n", uv_strerror(status));
        return;
    }

    if (events & UV_READABLE) {
        rl_callback_read_char();
    }
}

int main(int argc, char** argv) {
    rat_require_sudo_privileges(argv[0]);

    sniffer.selected_idx = -1;
    sniffer.readline_ready = 0;
    sniffer.pending_input = NULL;
    
    printf("Welcome to RAT CLI %s. Type 'help' for commands.\n", RAT_VERSION);
    
    char* home_path = getenv("HOME");
    assert(home_path != NULL);
    sprintf(history_path, "%s/%s", home_path, HISTORY_NAME);
    read_history(history_path);
    atexit(cleanup);

    sniffer.devices_size = rat_device_lookup(sniffer.devices);
    if (sniffer.devices_size == 0) {
        printf("No network devices found\n");
        return 1;
    }

    uv_loop_t loop;
    uv_loop_init(&loop);
    sniffer.loop = &loop;

    uv_idle_init(&loop, &sniffer.idle_handle);
    sniffer.idle_handle.data = &sniffer;
    uv_idle_start(&sniffer.idle_handle, on_idle);

    uv_poll_t stdin_poll;
    uv_poll_init(&loop, &stdin_poll, STDIN_FILENO);
    uv_poll_start(&stdin_poll, UV_READABLE, stdin_read_cb);

    char prompt[120] = "rat> ";
    rl_callback_handler_install(prompt, readline_handler);

    uv_run(&loop, UV_RUN_DEFAULT);

    rl_callback_handler_remove();
    
    if (sniffer.running) {
        uv_poll_stop(&sniffer.poll_handle);
        uv_close((uv_handle_t*)&sniffer.poll_handle, NULL);
    }
    
    uv_idle_stop(&sniffer.idle_handle);
    uv_close((uv_handle_t*)&sniffer.idle_handle, NULL);
    uv_close((uv_handle_t*)&stdin_poll, NULL);
    
    uv_run(&loop, UV_RUN_DEFAULT);
    
    uv_loop_close(&loop);
    
    printf("Goodbye!\n");
    return 0;
}
