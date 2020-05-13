import tornado.web
import tornado.websocket
import tornado.httpserver
import tornado.ioloop

import asyncio
import socket
import sys
import os

from worker_gateway.server import WebSocketChannelHandler
from heartbeat.handler import HeartbeatHandler
 

class Application(tornado.web.Application):
    def __init__(self):
        handlers = [
            (r'/run', WebSocketChannelHandler),
            (r'/heartbeat', HeartbeatHandler)
        ]

        tornado.web.Application.__init__(self, handlers)


if __name__ == '__main__':
    app = Application()
    server = tornado.httpserver.HTTPServer(app)
    server.listen(8080)
    tornado.ioloop.IOLoop.instance().start()
