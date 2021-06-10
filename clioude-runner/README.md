# CliOuDE-Runner

## Process Flow

1. Connect Server through WebSocket
2. Loop send runner info includes
   * CPU Free
   * Memory Free
   * Available Language & Executables(GCC/Python/...)
3. Prepare on Server responds with
   1. Language
   2. Code Content
4. Run as subprocess with interactive stdio
5. Send subprocess info after exit
   * Time Spent
   * Memory Usage
   * Exit Code

## Protocol

Bytes messages based on WebSocket.

Exactly one flag at the end of each message.

### Flags

* `\xc0`: language
* `\xc1`: code content
* `\xde`: kill
* `\xe0`: stdin append
* `\xe1`: stdout append
* `\xe2`: stderr append
* `\xe8`: exit info