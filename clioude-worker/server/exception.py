class WorkerException(Exception):
    def __init__(self, message):
        super().__init__()
        self.message = message


class CompileError(WorkerException):
    pass


class WorkerServiceError(WorkerException):
    pass


class KernelClientException(WorkerException):
    pass
