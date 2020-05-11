from tornado.web import RequestHandler, HTTPError


class HeartbeatHandler(RequestHandler):
    def post(self):
        if self.request.headers.get('X-Worker-Server-Token', None) != 'b82fd881d1303ba9794e19b7f4a5e2b79231d065f744e72172ad9ee792909126':
            raise HTTPError(403)
        print(self.request.body)