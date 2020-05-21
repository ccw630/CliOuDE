import asyncio
import hashlib
import os
import json

from tornado.web import RequestHandler, HTTPError

from orm import Worker

class HeartbeatHandler(RequestHandler):
    async def post(self):
        if self.request.headers.get('X-Worker-Server-Token', None) != hashlib.sha256(os.getenv('WORKER_TOKEN','').encode("utf-8")).hexdigest():
            raise HTTPError(403)
        data = json.loads(self.request.body.decode())
        loop = asyncio.get_running_loop()
        await loop.run_in_executor(None, Worker.upsert_worker, hostname=data['hostname'],
                                                               version=data['kernel_version'],
                                                               cpu_core=data["cpu_core"],
                                                               memory_usage=data["memory"],
                                                               cpu_usage=data["cpu"],
                                                               service_url=data["service_url"])