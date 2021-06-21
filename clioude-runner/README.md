# CliOuDE-Runner

## Process Flow

1. Connect Server through WebSocket
2. Send runner info includes
   * System info(CPU/Memory/OS Version/...)
   * Available language & executables(GCC/Python/...)
3. Prepare on Server responds with
   1. RunID(16 bytes) + Language
   2. Code content, write to directory `run/<RunID>/`
   3. If needed, compile the code in subprocess
4. Report compiling/running
5. Run as subprocess with interactive stdio
6. Send status info
   * Memory used increasely
   * Ok/CE/RE at done
   * Exit code at exit
## Protocol

Bytes messages based on WebSocket.

Exactly one flag at the end of each message.

### Flags

* `\xc0`: run id + language
* `\xc1`: code content
* `\xe0`: stdin append
* `\xe1`: stdout append
* `\xe2`: stderr append
* `\xe7`: status info
* `\xe8`: exit info

## Tips

* Preferred run in containers
* Subprocess killed on socket close