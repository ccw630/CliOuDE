import tornado.web
import tornado.websocket
import tornado.httpserver
import tornado.ioloop

from worker_gateway.server import WebSocketChannelHandler
from heartbeat.handler import HeartbeatHandler
from orm import Worker
 

class Application(tornado.web.Application):
    def __init__(self):
        handlers = [
            (r'/api/run', WebSocketChannelHandler),
            (r'/api/heartbeat', HeartbeatHandler)
        ]

        tornado.web.Application.__init__(self, handlers)


if __name__ == '__main__':
    Worker.cull_worker()
    app = Application()
    server = tornado.httpserver.HTTPServer(app)
    server.listen(8080)
    tornado.ioloop.IOLoop.instance().start()
