import asyncio
import os
import shutil
import json

import tornado.web
import tornado.websocket
import tornado.httpserver
import tornado.ioloop

from aiofile import AIOFile

from compiler import Compiler
from config import (COMPILER_USER_UID, IO_SOCK_DIR, RUN_GROUP_GID,
                    WORKER_WORKSPACE_BASE)
from exception import CompileError, KernelClientException
from kernel_client import KernelClient
from utils import logger, server_info, token


DEBUG = os.environ.get("worker_debug") == "1"


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


class PingHandler(tornado.web.RequestHandler):
    def post(self):
        if self.request.headers.get('X-Worker-Server-Token', None) != token:
            raise tornado.web.HTTPError(403)
        data = server_info()
        data["action"] = "pong"
        self.set_status(200)
        self.finish(data)


class WebSocketHandler(tornado.websocket.WebSocketHandler):

    def __init__(self, *args, **kwargs):
        super(WebSocketHandler, self).__init__(*args, **kwargs)
        self.connected = False
        self.started = False
        self.reader = None
        self.writer = None
        self.scanner = None
        self.unix_server = None


    async def _compile(self, submission_dir, compile_config, run_config, src):
        if compile_config:
            src_path = os.path.join(submission_dir, compile_config["src_name"])

            # write source code into file
            async with AIOFile(src_path, "w", encoding="utf-8") as f:
                await f.write(src)

            # compile source code, return exe file path
            exe_path = await Compiler().compile(compile_config=compile_config,
                                                src_path=src_path,
                                                output_dir=submission_dir)
        else:
            exe_path = os.path.join(submission_dir, run_config["exe_name"])
            async with AIOFile(exe_path, "w", encoding="utf-8") as f:
                f.write(src)
        return exe_path


    async def _run(self, language_config, src, max_cpu_time, max_real_time, max_memory, submission_id, input_content=None):
        # init
        compile_config = language_config.get("compile")
        run_config = language_config["run"]
        # 等待 io sock 的建立
        io_sock_path = os.path.join(IO_SOCK_DIR, submission_id)
        if not os.path.exists(io_sock_path):
            raise ValueError("IO Sock not found")

        with InitSubmissionEnv(WORKER_WORKSPACE_BASE, submission_id=str(submission_id)) as submission_dir:
            self.write_message(json.dumps({'type': 'result', 'data': {'result': -5}}))
            exe_path = await self._compile(submission_dir, compile_config=compile_config, run_config=run_config, src=src)
            self.write_message(json.dumps({'type': 'result', 'data': {'result': -6}}))

            input_path = None

            if input_content:
                input_path = os.path.join(submission_dir, 'input')

                async with AIOFile(input_path, "w", encoding="utf-8") as f:
                    await f.write(input_content)

            kernel_client = KernelClient(run_config=language_config["run"],
                                         exe_path=exe_path,
                                         max_cpu_time=max_cpu_time,
                                         max_real_time=max_real_time,
                                         max_memory=max_memory,
                                         io_sock_path="unix:" + io_sock_path,
                                         input_path=input_path)
            run_result = await kernel_client.run()
            return run_result


    def check_origin(self, origin):
        return True


    async def handle_connect(self, r, w):
        self.reader = r
        self.writer = w
        self.connected = True
        async def check():
            while self.connected:
                await asyncio.sleep(0.1)
                data = await self.reader.read(1024)
                if data:
                    message = data.decode()
                    self.write_message(json.dumps({'type': 'output', 'data': message}))
        self.scanner = asyncio.create_task(check())


    async def listen_socket(self, path):
        server = await asyncio.start_unix_server(
            self.handle_connect, os.path.join(IO_SOCK_DIR, path))

        async with server:
            await server.serve_forever()


    def open(self, data):
        if self.get_argument('token') != token:
            raise tornado.web.HTTPError(403)
        self.unix_server = asyncio.create_task(self.listen_socket(data))


    def _run_callback(self, future):
        try:
            self.write_message(json.dumps({'type': 'result', 'data': future.result()}))
        except CompileError as e:
            logger.exception(e)
            self.write_message(json.dumps({'type': 'result', 'data': {'result': -3, 'err': e.message}}))
        except tornado.websocket.WebSocketClosedError as e:
            logger.warn('Websocket already closed, ignore..')
        except Exception as e:
            logger.exception(e)
            self.write_message(json.dumps({'type': 'result', 'data': {'result': 5, 'err': e.__class__.__name__ + ": " + str(e)}}))
        async def delay_close():
            await asyncio.sleep(2)
            self.close(1000)
        asyncio.create_task(delay_close())
        self.started = False


    async def on_message(self, message):
        data = json.loads(message)
        msg_type = data.get('type', None)
        if msg_type == 'code' and not self.started:
            runner = asyncio.create_task(self._run(**data['data']))
            runner.add_done_callback(self._run_callback)
            self.started = True
        elif msg_type == 'input':
            while not self.connected or not self.started:
                await asyncio.sleep(0.1)
            self.writer.write(data['data'].encode())
            await self.writer.drain()


    def on_close(self):
        self.connected = False
        if self.writer:
            self.writer.close()
        if self.scanner:
            self.scanner.cancel()
        if self.unix_server:
            self.unix_server.cancel()


class Application(tornado.web.Application):
    def __init__(self):
        handlers = [
            (r'/ping', PingHandler),
            (r'/(\w+)', WebSocketHandler)
        ]

        tornado.web.Application.__init__(self, handlers)


if __name__ == '__main__':
    app = Application()
    app_server = tornado.httpserver.HTTPServer(app)
    app_server.listen(8080)
    tornado.ioloop.IOLoop.instance().start()
