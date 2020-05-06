import os

import kernel
from config import KERNEL_LOG_PATH, RUN_GROUP_GID, RUN_USER_UID


class KernelClient:
    def __init__(self, run_config, exe_path, max_cpu_time, max_real_time, max_memory, io_sock_path):
        self._run_config = run_config
        self._exe_path = exe_path
        self._max_cpu_time = max_cpu_time
        self._max_real_time = max_real_time
        self._max_memory = max_memory
        self._io_sock_path = io_sock_path

    async def run(self):
        command = self._run_config["command"].format(exe_path=self._exe_path, exe_dir=os.path.dirname(self._exe_path),
                                                     max_memory=int(self._max_memory / 1024)).split(" ")
        env = ["PATH=" + os.environ.get("PATH", "")] + self._run_config.get("env", [])

        run_result = await kernel.run(max_cpu_time=self._max_cpu_time,
                                      max_real_time=self._max_real_time,
                                      max_memory=self._max_memory,
                                      max_stack=128 * 1024 * 1024,
                                      max_output_size=1024 * 1024 * 16,
                                      max_process_number=kernel.UNLIMITED,
                                      exe_path=command[0],
                                      input_path=self._io_sock_path,
                                      output_path=self._io_sock_path,
                                      error_path=self._io_sock_path,
                                      args=command[1::],
                                      env=env,
                                      log_path=KERNEL_LOG_PATH,
                                      seccomp_rule_name=self._run_config["seccomp_rule"],
                                      uid=RUN_USER_UID,
                                      gid=RUN_GROUP_GID,
                                      memory_limit_check_only=self._run_config.get("memory_limit_check_only", 0))
        return run_result

