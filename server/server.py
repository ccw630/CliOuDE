import json
import os
import shutil
import time
from compiler import Compiler

from flask import Flask, Response, request

from config import (COMPILER_USER_UID, IO_SOCK_DIR, RUN_GROUP_GID,
                    WORKER_WORKSPACE_BASE)
from exception import CompileError, TokenVerificationFailed, KernelClientException
from kernel_client import KernelClient
from utils import logger, server_info, token

app = Flask(__name__)
DEBUG = os.environ.get("worker_debug") == "1"
app.debug = DEBUG


class InitSubmissionEnv:
    def __init__(self, worker_workspace, submission_id):
        self.work_dir = os.path.join(worker_workspace, submission_id)

    def __enter__(self):
        try:
            os.mkdir(self.work_dir)
            os.chown(self.work_dir, COMPILER_USER_UID, RUN_GROUP_GID)
            os.chmod(self.work_dir, 0o711)
        except Exception as e:
            logger.exception(e)
            raise KernelClientException("failed to create runtime dir")
        return self.work_dir

    def __exit__(self, exc_type, exc_val, exc_tb):
        if not DEBUG:
            try:
                shutil.rmtree(self.work_dir)
            except Exception as e:
                logger.exception(e)
                raise KernelClientException("failed to clean runtime dir")


class WorkerServer:
    @classmethod
    def ping(cls):
        data = server_info()
        data["action"] = "pong"
        return data


    @classmethod
    def run(cls, language_config, src, max_cpu_time, max_real_time, max_memory, submission_id):
        # init
        compile_config = language_config.get("compile")
        run_config = language_config["run"]
        # 等待 io sock 的建立
        wait = 10
        io_sock_path = os.path.join(IO_SOCK_DIR, submission_id)
        while wait:
            if not os.path.exists(io_sock_path):
                time.sleep(1)
                wait -= 1
            else:
                break
        else:
            raise ValueError("IO Sock not found")

        with InitSubmissionEnv(WORKER_WORKSPACE_BASE, submission_id=str(submission_id)) as submission_dir:
            exe_path = cls._compile(submission_dir, compile_config=compile_config, run_config=run_config, src=src)

            kernel_client = KernelClient(run_config=language_config["run"],
                                         exe_path=exe_path,
                                         max_cpu_time=max_cpu_time,
                                         max_real_time=max_real_time,
                                         max_memory=max_memory,
                                         io_sock_path="unix:" + io_sock_path)
            run_result = kernel_client.run()
            return run_result


    @classmethod
    def _compile(cls, submission_dir, compile_config, run_config, src):
        if compile_config:
            src_path = os.path.join(submission_dir, compile_config["src_name"])

            # write source code into file
            with open(src_path, "w", encoding="utf-8") as f:
                f.write(src)

            # compile source code, return exe file path
            exe_path = Compiler().compile(compile_config=compile_config,
                                          src_path=src_path,
                                          output_dir=submission_dir)
        else:
            exe_path = os.path.join(submission_dir, run_config["exe_name"])
            with open(exe_path, "w", encoding="utf-8") as f:
                f.write(src)
        return exe_path


@app.route('/', defaults={'path': ''})
@app.route('/<path:path>', methods=["POST"])
def server(path):
    if path in ("run", "ping"):
        _token = request.headers.get("X-Worker-Server-Token")
        try:
            if _token != token:
                raise TokenVerificationFailed("invalid token")
            try:
                data = request.json
            except:
                data = {}
            ret = {"err": None, "data": getattr(WorkerServer, path)(**data)}
        except (CompileError, TokenVerificationFailed) as e:
            logger.exception(e)
            ret = {"err": e.__class__.__name__, "data": e.message}
        except Exception as e:
            logger.exception(e)
            ret = {"err": "KernelClientError", "data": e.__class__.__name__ + " :" + str(e)}
    else:
        ret = {"err": "InvalidRequest", "data": "404"}
    return Response(json.dumps(ret), mimetype='application/json')


if DEBUG:
    logger.info("DEBUG=ON")

# gunicorn -w 4 -b 0.0.0.0:8080 server:app
if __name__ == "__main__":
    app.run(debug=DEBUG)
