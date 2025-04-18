#include "rat_utils.h"

void rat_require_sudo_privileges(const char* program_path) {
    if (geteuid() != 0) {
        printf("This program requires root privileges.\n");
	char cmd[256];
	snprintf(cmd, sizeof(cmd), "sudo %s", program_path);
	system(cmd);
	exit(EXIT_SUCCESS);
    }
}
