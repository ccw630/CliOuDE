import asyncio
import logging
import json

log = logging.getLogger(__name__)

class JsonRpcStreamReader:

    def __init__(self, rfile):
        self._rfile = rfile
        self.disconnected = False

    def listen(self, message_consumer):
        asyncio.create_task(self.read_forever(message_consumer))

    async def read_forever(self, message_consumer):
        while not self.disconnected:
            try:
                request_str = await self._read_message()
            except ValueError:
                if self._rfile.closed:
                    return
                else:
                    log.exception("Failed to read from rfile")

            if not request_str:
                break

            try:
                message_consumer(json.loads(request_str.decode('utf-8')))
            except (ValueError, json.decoder.JSONDecodeError):
                log.exception("Failed to parse JSON message %s", request_str)
                continue

    async def _read_message(self):
        line = await self._rfile.readline()

        if not line:
            return None

        content_length = self._content_length(line)

        if not content_length:
            return None

        while line and line.strip():
            line = await self._rfile.readline()

        if not line:
            return None

        res = b''
        while content_length > 0:
            sub = await self._rfile.read(content_length)
            res += sub
            content_length -= len(sub)
        return res

    @staticmethod
    def _content_length(line):
        if line.startswith(b'Content-Length: '):
            _, value = line.split(b'Content-Length: ')
            value = value.strip()
            try:
                return int(value)
            except ValueError:
                raise ValueError("Invalid Content-Length header: {}".format(value))

        return None
