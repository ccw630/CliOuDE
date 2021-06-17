# CliOuDE-Runner

## Process Flow

1. Connect Server through WebSocket
2. Loop send runner info includes
   * CPU Free
   * Memory Free
   * Available Language & Executables(GCC/Python/...)
3. Prepare on Server responds with
   1. RunID(16 bytes) + Language
   2. Code Content
4. Report running/compiling
5. Run as subprocess with interactive stdio
6. Send status info
   * Ok/CE/RE/TLE
7. Send subprocess info after exit

   * Exit Code(1 bytes as u8)
   * Time Spent(8 bytes as u64)
   * Memory Usage(in KB, 8 bytes as u64)

## Protocol

Bytes messages based on WebSocket.

Exactly one flag at the end of each message.

### Flags

* `\xc0`: run id + language
* `\xc1`: code content
* `\xde`: kill
* `\xe0`: stdin append
* `\xe1`: stdout append
* `\xe2`: stderr append
* `\xe7`: status info
* `\xe8`: exit info