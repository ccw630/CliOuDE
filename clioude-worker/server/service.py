import json
import os

import requests
import socket

from exception import WorkerServiceError
from utils import server_info, logger, token


class WorkerService:
    def __init__(self):
        self.service_url = f"ws://{socket.gethostname()}:8080"
        self.backend_url = os.environ["BACKEND_URL"]

    def _request(self, data):
        try:
            resp = requests.post(self.backend_url, json=data,
                                 headers={"X-WORKER-SERVER-TOKEN": token,
                                          "Content-Type": "application/json"}, timeout=5).status_code
        except Exception as e:
            logger.exception(e)
            raise WorkerServiceError("Heartbeat request failed")
        try:
            if resp != 200:
                raise WorkerServiceError("Heartbeat response not OK")
        except Exception as e:
            logger.exception(f"Heartbeat failed, response is {resp}")
            raise WorkerServiceError("Invalid heartbeat response")

    def heartbeat(self):
        data = server_info()
        data["action"] = "heartbeat"
        data["service_url"] = self.service_url
        self._request(data)


if __name__ == "__main__":
    try:
        if not os.environ.get("DISABLE_HEARTBEAT"):
            service = WorkerService()
            service.heartbeat()
        exit(0)
    except Exception as e:
        logger.exception(e)
        exit(1)
