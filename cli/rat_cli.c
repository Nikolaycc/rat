#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <readline/readline.h>
#include <readline/history.h>
#include <assert.h>
#include <unistd.h>
#include <signal.h>

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
} sniffer_state_t;

sniffer_state_t sniffer = {0};

void cleanup() {
    write_history(history_path);
    if (sniffer.running) {
        rat_cap_destroy(&sniffer.cap);
    }
}

void cap_cb(rat_packet_t* packet, void* data) {
    (void)data;
    
    // todo: ahh...    
    // just i will use debug info
    
    if (packet->eth) {
    }

    if (packet->ip) {
    }

    if (packet->tcp) {
    }
    else if (packet->udp) {
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
    
    rat_cap_create(&sniffer.cap, &sniffer.devices[device_idx], NULL, 0);

    sniffer.running = 1;
    printf("Started sniffing on %s\n", sniffer.devices[device_idx].name);
}

void stop_sniffing() {
    if (!sniffer.running) {
        printf("Sniffer is not running\n");
        return;
    }

    sniffer.selected_idx = -1;
    
    rat_cap_destroy(&sniffer.cap);
    sniffer.running = 0;
    printf("Stopped sniffing\n");
}

void list_interfaces() {
    printf("Available interfaces:\n");
    for (int i = 0; i < sniffer.devices_size; i++) {
        printf("%d: %s\n", i, sniffer.devices[i].name);
    }
}

void process_command(char* input) {
    char* cmd = strtok(input, " ");
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
    else {
        printf("Unknown command. Available commands:\n");
        printf("  list               - List available interfaces\n");
        printf("  start <interface>  - Start sniffing on interface\n");
        printf("  stop               - Stop sniffing\n");
        printf("  exit               - Exit program\n");
    }
}

int main(int argc, char** argv) {
    rat_require_sudo_privileges(argv[0]);

    sniffer.selected_idx = -1;
    
    printf("Welcome to RAT CLI %s. Type 'help' for commands.\n", RAT_VERSION);
    // Setup history
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

    char* input;
    while (1) {
		char in[120] = "rat> ";
		if (sniffer.selected_idx >= 0) snprintf(in, sizeof(in), "rat[sniffing %s]> ", sniffer.devices[sniffer.selected_idx].name);
        input = readline(in);

        if (!input || strcmp(input, "exit") == 0) {
            free(input);
            break;
        }
        
        if (*input) {
            add_history(input);
            process_command(input);
        }
        
        free(input);
        
        if (sniffer.running) {
            rat_cap_loop(&sniffer.cap, cap_cb, 1);
        }
    }
    
    printf("Goodbye!\n");
    return 0;
}
