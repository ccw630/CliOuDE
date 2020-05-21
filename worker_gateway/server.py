from tornado.websocket import WebSocketHandler
from tornado.web import HTTPError

import asyncio
import json
import uuid

from worker_gateway.client import WebSocketClient
from languages import languages
from orm import Worker


class WebSocketChannelHandler(WebSocketHandler):

    def __init__(self, *args, **kwargs):
        super(WebSocketChannelHandler, self).__init__(*args, **kwargs)
        self.client = None
        self.submission_id = None


    def check_origin(self, origin):
        return True
 

    def open(self):
        self.submission_id = uuid.uuid1().hex
        loop = asyncio.get_event_loop()
        worker = await loop.run_in_executor(None, Worker.choose_worker)
        if not worker:
            raise HTTPError(404)
        self.client = WebSocketClient(worker.service_url, lambda: self.close(1000))
        self.client.on_open(self.submission_id, self.write_message)
        self.write_message(json.dumps({'type': 'result', 'data': {'result': -1}}))

 
    def on_message(self, message):
        data = json.loads(message)
        if data.get('type', None) == 'code':
            language = data['data'].pop('language')
            data['data']['language_config'] = languages[language]
            data['data']['submission_id'] = self.submission_id
            data['data']['max_cpu_time'] = 3000
            data['data']['max_real_time'] = 60000
            data['data']['max_memory'] = 1024 * 1024 * 512
            message = json.dumps(data)
        self.client.on_message(message)
        

    def on_close(self):
        self.client.on_close()
 