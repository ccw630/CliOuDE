import logging
import json

log = logging.getLogger(__name__)

class JsonRpcStreamWriter:

    def __init__(self, wfile, **json_dumps_args):
        self._wfile = wfile
        self._json_dumps_args = json_dumps_args

    async def write(self, message):
        try:
            body = json.dumps(message, **self._json_dumps_args)

            content_length = len(body) if isinstance(body, bytes) else len(body.encode('utf-8'))

            response = (
                "Content-Length: {}\r\n"
                "Content-Type: application/vscode-jsonrpc; charset=utf8\r\n\r\n"
                "{}".format(content_length, body)
            )

            self._wfile.write(response.encode('utf-8'))
            await self._wfile.drain()
        except Exception:
            log.exception("Failed to write message to output file %s", message)
