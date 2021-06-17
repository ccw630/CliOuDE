use std::env;

use futures::{future, pin_mut, SinkExt, StreamExt};
use std::cell::RefCell;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process::{exit, Stdio};
use sysinfo::{ProcessExt, RefreshKind, Signal, System, SystemExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

#[tokio::main]
async fn main() {
    let connect_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires at least one argument"));

    let url = Url::parse(&connect_addr).unwrap();
    // loop {
    run(&url).await;
    // }

}

async fn run(url: &Url) {

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");
    let (mut ws_writer, mut ws_reader) = ws_stream.split();

    let mut language: Option<String> = None;
    let mut run_id: String = String::new();
    let mut pid: Option<u32>;
    let mut cmd: Command;
    loop {
        let message = ws_reader.next().await.unwrap();
        let mut data = message.unwrap().into_data();
        println!("Received message {:?}", &data);
        match data.pop() {
            Some(0xc0u8) => {
                run_id = String::from_utf8_lossy(&data[..16]).into_owned();
                language = Some(String::from_utf8_lossy(&data[16..]).into_owned());
            }
            Some(0xc1u8) => {
                let single_run = get_raw_run(&language);
                let path = format!("run/{}", run_id);
                fs::create_dir_all(&path).expect("Failed to create directory");
                let mut file = File::create(format!("{}/{}", &path, single_run.file))
                    .expect("Failed to create file");
                file.write_all(&data).expect("Failed to write file");

                if let Some(mut compile_command) = single_run.compile_command {
                    let mut process = compile_command
                        .current_dir(&path)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .expect("Failed to create compile process");
                    pid = process.id();
                    ws_writer
                        .send(Message::binary(vec![RunStatus::Compiling as u8, 0xe7]))
                        .await
                        .unwrap();
                    let output = process
                        .wait_with_output()
                        .await
                        .expect("Compile process error");
                    if !output.status.success() {
                        ws_writer
                            .send(Message::binary([output.stdout, vec![0xe1]].concat()))
                            .await
                            .unwrap();
                        ws_writer
                            .send(Message::binary([output.stderr, vec![0xe2]].concat()))
                            .await
                            .unwrap();
                        ws_writer
                            .send(Message::binary(vec![RunStatus::CompileError as u8, 0xe8]))
                            .await
                            .unwrap();
                        exit(0);
                    }
                }

                let mut run_command = single_run.run_command;
                run_command.current_dir(&path);
                cmd = run_command;
                break;
            }
            Some(0xdeu8) => {
                // process.kill();
            }
            None | Some(_) => (),
        }
    }

    let mut process = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn command");

    pid = process.id();
    let stdin = process
        .stdin
        .take()
        .expect("child did not have a handle to stdin");
    let stdout = process
        .stdout
        .take()
        .expect("child did not have a handle to stdout");
    let stderr = process
        .stderr
        .take()
        .expect("child did not have a handle to stderr");
    ws_writer
        .send(Message::binary(vec![RunStatus::Running as u8, 0xe7]))
        .await
        .unwrap();

    let (ws_tx, ws_rx) = futures::channel::mpsc::unbounded::<Vec<u8>>();
    let stdout_tx = ws_tx.clone();
    let stderr_tx = ws_tx.clone();

    tokio::spawn(pipe(stdout, stdout_tx, 0xe1));
    tokio::spawn(pipe(stderr, stderr_tx, 0xe2));
    tokio::spawn(async move {
        let status = process
            .wait()
            .await
            .expect("child process encountered an error");

        ws_tx
            .unbounded_send(vec![
                match status.code() {
                    Some(code) => code,
                    None => 9,
                } as u8,
                0xe8,
            ])
            .unwrap();
    });

    let wsout_cell = RefCell::new(ws_writer);
    let out_to_ws = ws_rx.for_each(|data| async {
        let mut wsout = wsout_cell.borrow_mut();
        let last = data.last().unwrap().clone();
        wsout
            .send(Message::binary(data))
            .await
            .expect("send message failed");
        wsout.flush().await.unwrap();
        if last == 0xe8u8 {
            wsout.close().await.unwrap();
        }
    });

    let stdin_cell = RefCell::new(stdin);
    let ws_to_stdin = ws_reader.for_each(|message| async {
        let mut data = message.unwrap().into_data();
        match data.pop() {
            Some(0xdeu8) => {
                let mut s = System::new();
                s.refresh_specifics(RefreshKind::new().with_processes());
                if let Some(process) = s.get_process(pid.unwrap() as i32) {
                    process.kill(Signal::Kill);
                }
            }
            Some(0xe0u8) => {
                let mut stdin = stdin_cell.borrow_mut();
                stdin.write_all(&data).await.expect("write failed");
                stdin.flush().await.unwrap();
            }
            None | Some(_) => (),
        };
    });

    pin_mut!(out_to_ws, ws_to_stdin);
    future::select(out_to_ws, ws_to_stdin).await;
}

async fn pipe(
    mut reader: impl tokio::io::AsyncRead + std::marker::Unpin,
    tx: futures::channel::mpsc::UnboundedSender<Vec<u8>>,
    end: u8,
) {
    loop {
        let mut buf = vec![0; 1024];
        let n = match reader.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        buf.push(end);
        tx.unbounded_send(buf).unwrap();
    }
}

enum RunStatus {
    Running = 0,
    Compiling,
    CompileError,
    Ok,
    RuntimeError,
    TimeLimitExceeded,
    Killed,
}

struct SingleRun {
    file: String,
    run_command: Command,
    compile_command: Option<Command>,
}

fn get_raw_run(lang: &Option<String>) -> SingleRun {
    match lang.as_deref() {
        Some("C++") => {
            let file = String::from("main.cpp");
            let mut compile_command = Command::new("g++");
            compile_command.arg("-O2");
            compile_command.arg("-w");
            compile_command.arg("-fmax-errors=3");
            compile_command.arg("-std=c++17");
            compile_command.arg(&file);
            compile_command.arg("-lm");
            compile_command.arg("-o");
            compile_command.arg("./main");
            let run_command = Command::new("./main");
            SingleRun {
                file,
                run_command,
                compile_command: Some(compile_command),
            }
        }
        Some("Python") => {
            let file = String::from("main.py");
            let mut run_command = Command::new("python3");
            run_command.arg(&file);
            SingleRun {
                file,
                run_command,
                compile_command: None,
            }
        }
        Some(_) => panic!("No such language"),
        None => panic!("No language received"),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn integration_test_ok() {
        let (listener, url) = before_test().await;
        tokio::join!(test_server_ok(listener), run(&url));
    }

    #[tokio::test]
    async fn integration_test_kill() {
        let (listener, url) = before_test().await;
        tokio::join!(test_server_kill(listener), run(&url));
    }

    async fn before_test() -> (tokio::net::TcpListener, url::Url) {
        for port in 1025u16..65535u16 {
            let addr = format!("ws://127.0.0.1:{}", port);
            match tokio::net::TcpListener::bind(&addr[5..]).await {
                Ok(l) => return (l, Url::parse(&addr).unwrap()),
                _ => ()
            }
       }
       panic!("No port available, failed to bind")
    }

    async fn test_server_ok(listener: tokio::net::TcpListener) {
        let (stream, _) = listener.accept().await.expect("Failed to accept");
        //let addr = stream.peer_addr().expect("connected streams should have a peer address");
        let ws_stream = tokio_tungstenite::accept_async(stream).await.expect("Error during the websocket handshake occurred");
        let (mut write, mut read) = ws_stream.split();
        write.send(Message::binary(b"0123456789abcdefC++\xc0".as_ref())).await.expect("Send fail");
        write.send(Message::binary(b"#include<iostream>\nusing namespace std;int main(){int n;cin>>n;cout<<n+1<<endl;}\n\xc1".as_ref())).await.expect("Send fail");
        assert_eq!(Message::binary(vec![0x01, 0xe7]), read.next().await.unwrap().unwrap());
        assert_eq!(Message::binary(vec![0x00, 0xe7]), read.next().await.unwrap().unwrap());
        write.send(Message::binary(b"1\n\xe0".as_ref())).await.expect("Send fail");
        assert_eq!(Message::binary(b"2\n\xe1".as_ref()), read.next().await.unwrap().unwrap());
        write.close().await.expect("Close fail");
    }

    async fn test_server_kill(listener: tokio::net::TcpListener) {
        let (stream, _) = listener.accept().await.expect("Failed to accept");
        //let addr = stream.peer_addr().expect("connected streams should have a peer address");
        let ws_stream = tokio_tungstenite::accept_async(stream).await.expect("Error during the websocket handshake occurred");
        let (mut write, mut read) = ws_stream.split();
        write.send(Message::binary(b"0123456789abcdefC++\xc0".as_ref())).await.expect("Send fail");
        write.send(Message::binary(b"#include<iostream>\nusing namespace std;int main(){int n;cin>>n;cout<<n+1<<endl;}\n\xc1".as_ref())).await.expect("Send fail");
        assert_eq!(Message::binary(vec![0x01, 0xe7]), read.next().await.unwrap().unwrap());
        assert_eq!(Message::binary(vec![0x00, 0xe7]), read.next().await.unwrap().unwrap());
        write.send(Message::binary(vec![0xde])).await.expect("Send fail");
        assert_eq!(Message::binary(vec![0x09, 0xe8]), read.next().await.unwrap().unwrap());
        write.close().await.expect("Close fail");
    }

}
