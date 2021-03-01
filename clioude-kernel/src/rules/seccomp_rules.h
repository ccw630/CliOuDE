#ifndef KERNEL_SECCOMP_RULES_H
#define KERNEL_SECCOMP_RULES_H
#include <stdbool.h>
#include "../runner.h"

int _c_cpp_seccomp_rules(struct config *_config, bool allow_write_file);
int c_cpp_seccomp_rules(struct config *_config);
int general_seccomp_rules(struct config *_config);

#endif //KERNEL_SECCOMP_RULES_H
