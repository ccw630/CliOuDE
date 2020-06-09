import asyncio
import logging

from jsonrpc.reader import JsonRpcStreamReader
from jsonrpc.writer import JsonRpcStreamWriter

log = logging.getLogger(__name__)

class LanguageServer:

    reader = None
    writer = None
    proc = None

    def __init__(self, proc, reader, writer):
        self.proc = proc
        self.reader = reader
        self.writer = writer

    @classmethod
    async def create(cls, *args):
        log.info(f"Spawning ls subprocess {' '.join(args)}")

        proc = await asyncio.create_subprocess_exec(
            *args,
            limit=1024 * 512,
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE
        )

        writer = JsonRpcStreamWriter(proc.stdin)
        reader = JsonRpcStreamReader(proc.stdout)
        return cls(proc, reader, writer)

    def close(self):
        log.info("Closing ls subprocess")
        self.reader.disconnected = True
        self.proc.kill()

    @classmethod
    async def pyls(cls):
        return await cls.create('pyls', '-v')

    @classmethod
    async def jsls(cls):
        return await cls.create('flow', 'lsp')

    @classmethod
    async def bashls(cls):
        return await cls.create('bash-language-server', 'start')

    @classmethod
    async def ccls(cls):
        #return await cls.create('clangd')
        return await cls.create('ccls', r'--init={"clang": {"extraArgs": ["-isystem", "/usr/local/Cellar/gcc/9.3.0/include/c++/9.3.0"], "resourceDir": "/usr/local/Cellar/gcc/9.3.0/include/c++/9.3.0"}}')

    @classmethod
    async def javals(cls):
        return await cls.create('/usr/local/opt/openjdk/bin/java', '-Declipse.application=org.eclipse.jdt.ls.core.id1', '-Dosgi.bundles.defaultStartLevel=4', '-Declipse.product=org.eclipse.jdt.ls.core.product', '-Dlog.protocol=true', '-Dlog.level=ALL', '-noverify', '-Xmx1G', '-jar', '/Users/ccw/Downloads/jdt-language-server-latest/plugins/org.eclipse.equinox.launcher_1.5.700.v20200207-2156.jar', '-configuration', '/Users/ccw/Downloads/jdt-language-server-latest/config_linux', '-data', '/tmp/ls', '--add-modules=ALL-SYSTEM', '--add-opens', 'java.base/java.util=ALL-UNNAMED', '--add-opens', 'java.base/java.lang=ALL-UNNAMED')
