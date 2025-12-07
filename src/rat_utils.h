#ifndef _RAT_UTILS_H
#define _RAT_UTILS_H

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>

#define RATAPI __attribute__((visibility("default")))

RATAPI void rat_require_sudo_privileges(const char* program_path);

#endif
