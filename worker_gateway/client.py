from tornado.concurrent import Future
from tornado.websocket import WebSocketHandler, websocket_connect
from tornado.httpclient import HTTPRequest
from tornado.ioloop import IOLoop

import asyncio
import logging
import hashlib
import os

logger = logging.getLogger(__name__)

class WebSocketClient:

    def __init__(self, worker_base, close_callback):
        self.submission_id = None
        self.ws = None
        self.ws_future = Future()
        self.disconnected = False
        self.establish_attempts = 10
        self.worker_base_url = worker_base
        self.close_callback = close_callback


    def _connect(self, submission_id):
        self.ws = None
        self.submission_id = submission_id
        ws_url = f'{self.worker_base_url}/{submission_id}?token={hashlib.sha256(os.getenv('WORKER_TOKEN','').encode("utf-8")).hexdigest()}'
        logger.info('Connecting to {}'.format(ws_url))

        request = HTTPRequest(ws_url)
        self.ws_future = websocket_connect(request)
        self.ws_future.add_done_callback(self._connection_done)


    def _connection_done(self, future):
        if not self.disconnected and future.exception() is None:
            self.ws = future.result()
            self.attempt_id = 0
        else:
            logger.warning(f'Websocket connection of ID {self.submission_id} has been closed via client disconnect or due to error.')
            if self.attempt_id < self.establish_attempts:
                self.attempt_id += 1
            else:
                logger.error(f'Re-establish exhaust max attempts {self.establish_attempts}, give it up')
                self._disconnect()


    def _disconnect(self):
        self.disconnected = True
        if self.ws is not None:
            self.ws.close()
        elif not self.ws_future.done():
            self.ws_future.cancel()


    async def _read_messages(self, callback):
        while self.ws is not None:
            message = None
            if not self.disconnected:
                try:
                    message = await self.ws.read_message()
                except Exception as e:
                    log.exception(f'Exception reading message from websocket: {e}')
                if message is None:
                    if not self.disconnected:
                        self._disconnect()
                        self.close_callback()
                    break
                callback(message)
            else:
                break

        if not self.disconnected:
            logger.info(f'Attempting to re-establish the connection to Worker: {self.submission_id}')
            await asyncio.sleep(1)
            self._connect(self.submission_id)
            async def f(future):
                await self._read_messages(callback)
            loop = IOLoop.current()
            loop.add_future(self.ws_future, lambda future: self._read_messages(callback))


    def on_open(self, submission_id, callback):
        self.attempt_id = 0
        self._connect(submission_id)
        loop = IOLoop.current()
        loop.add_future(self.ws_future, lambda future: self._read_messages(callback))


    def on_message(self, message):
        if self.ws is None:
            self.ws_future.add_done_callback(lambda future: self._write_message(message))
        else:
            self._write_message(message)


    def _write_message(self, message):
        try:
            if not self.disconnected and self.ws is not None:
                self.ws.write_message(message)
        except Exception as e:
            logger.exception(f'Exception writing message to websocket: {e}')


    def on_close(self):
        self._disconnect()
