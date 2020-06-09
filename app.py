import asyncio
import logging
import json

from tornado import ioloop, web, websocket

from language_server import LanguageServer

log = logging.getLogger(__name__)

language_servers = {
    'python': LanguageServer.pyls,
    'javascript': LanguageServer.jsls,
    'cpp': LanguageServer.ccls,
    'c': LanguageServer.ccls,
    'shell': LanguageServer.bashls,
    'java': LanguageServer.javals
}


class LanguageServerWebSocketHandler(websocket.WebSocketHandler):

    ls = None

    async def open(self, language):
        self.ls = await language_servers[language]()
        self.ls.reader.listen(lambda msg: self.write_message(json.dumps(msg)))

    async def on_message(self, message):
        await self.ls.writer.write(json.loads(message))

    def check_origin(self, origin):
        return True

    def on_close(self):
        self.ls.close()


if __name__ == "__main__":
    app = web.Application([
        (r"/lsp/(\w+)", LanguageServerWebSocketHandler),
    ])
    app.listen(8999)
    ioloop.IOLoop.current().start()