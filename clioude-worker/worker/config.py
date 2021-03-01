import os
import pwd

import grp

WORKER_WORKSPACE_BASE = "/worker/run"
LOG_BASE = "/log"

COMPILER_LOG_PATH = os.path.join(LOG_BASE, "compile.log")
KERNEL_LOG_PATH = os.path.join(LOG_BASE, "kernel.log")
WORKER_LOG_PATH = os.path.join(LOG_BASE, "worker.log")

RUN_USER_UID = pwd.getpwnam("code").pw_uid
RUN_GROUP_GID = grp.getgrnam("code").gr_gid

COMPILER_USER_UID = pwd.getpwnam("compiler").pw_uid
COMPILER_GROUP_GID = grp.getgrnam("compiler").gr_gid

IO_SOCK_DIR = "/worker/socks"
