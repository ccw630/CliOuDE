use std::env;

use url::Url;
use futures::{future, pin_mut, StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::Error};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::process::Command;
use std::process::Stdio;
use std::cell::RefCell;
use std::str;


#[tokio::main]
async fn main() {
    let connect_addr =
        env::args().nth(1).unwrap_or_else(|| panic!("this program requires at least one argument"));

    let url = Url::parse(&connect_addr).unwrap();

    loop {
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
        println!("WebSocket handshake has been successfully completed");
        let (ws_writer, ws_reader) = ws_stream.split();
        let mut cmd = Command::new("python3");
        cmd.arg("/Users/ccw/2.py");
        execute(cmd, ws_writer, ws_reader).await;
        break;
    }
    
}

async fn execute(mut cmd: tokio::process::Command, mut ws_writer: impl futures::sink::Sink<Message> + std::marker::Unpin, ws_reader: impl futures::Stream<Item = std::result::Result<Message, Error>>) {
    let mut process = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("failed to spawn command");

    let pid = process.id().unwrap();
    let stdin = process.stdin.take().expect("child did not have a handle to stdin");
    let stdout = process.stdout.take().expect("child did not have a handle to stdout");
    let stderr = process.stderr.take().expect("child did not have a handle to stderr");



    match ws_writer.send(Message::from(format!("PID: {:?}", pid))).await {
        Err(_) => panic!("failed to send message"),
        Ok(_) => ()
    };

    // Ensure the child process is spawned in the runtime so it can
    // make progress on its own while we await for any output.
    tokio::spawn(async move {
        let status = process.wait().await
            .expect("child process encountered an error");

        println!("child status was: {}", status);
    });

    // Handle stdout & stderr
    let (stdout_tx, ws_rx) = futures::channel::mpsc::unbounded::<Message>();
    let stderr_tx = stdout_tx.clone();
    tokio::spawn(pipe(stdout, stdout_tx, 0xe1));
    tokio::spawn(pipe(stderr, stderr_tx, 0xe2));
    let stdouterr_to_ws = ws_rx.map(Ok).forward(ws_writer);

    // Handle stdin
    let stdin_cell = RefCell::new(stdin);
    let ws_to_stdin = {
        ws_reader.for_each(|message| async {
            let mut data = message.unwrap().into_data();
            match data.pop() {
                Some(0xdeu8) => {
                    // kill running process & throw
                },
                None | Some(_) => (),
            };
            println!("WS Recieved {:?}", str::from_utf8(&data).unwrap());
            let mut stdin = stdin_cell.borrow_mut();
            stdin.write_all(&data).await.expect("write failed");
            stdin.flush().await.unwrap();
        })
    };

    pin_mut!(stdouterr_to_ws, ws_to_stdin);
    future::select(stdouterr_to_ws, ws_to_stdin).await;
}

async fn pipe(mut reader: impl tokio::io::AsyncRead + std::marker::Unpin, tx: futures::channel::mpsc::UnboundedSender<Message>, end: u8) {
    loop {
        let mut buf = vec![0; 1024];
        let n = match reader.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        println!("OUTPUT Recieved {:?}", str::from_utf8(&buf).unwrap());
        buf.push(end);
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}
