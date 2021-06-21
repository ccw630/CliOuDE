use std::env;

use futures::{future, pin_mut, SinkExt, StreamExt};
use std::cell::RefCell;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process::Stdio;
use sysinfo::{ProcessExt, RefreshKind, Signal, System, SystemExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::Error};
use url::Url;

#[tokio::main]
async fn main() {
    let connect_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires at least one argument"));

    let url = Url::parse(&connect_addr).unwrap();
    // loop {
    let mut runner = Run::new();
    match runner.run(&url).await {
        Ok(_) => (),
        Err(err) => println!("FATAL: Run {} ERROR {:?}", runner.id, err),
    }

    // }
}

struct Run {
    id: String,
    pid: Option<u32>,
}

impl Run {
    fn new() -> Run {
        Run {
            id: String::new(),
            pid: None,
        }
    }

    async fn run(&mut self, url: &Url) -> Result<ExitStatus, Error> {
        let (ws_stream, _) = connect_async(url).await?;
        let (mut ws_writer, mut ws_reader) = ws_stream.split();
        let mut language: Option<String> = None;
        let mut process: Option<tokio::process::Child> = None;
        let mut cmd: Command;
        loop {
            let message = ws_reader.next().await.unwrap();
            let mut data = message?.into_data();
            match data.pop() {
                Some(0xc0u8) => {
                    self.id = String::from_utf8_lossy(&data[..16]).into_owned();
                    language = Some(String::from_utf8_lossy(&data[16..]).into_owned());
                }
                Some(0xc1u8) => {
                    let single_run = get_raw_run(&language);
                    let path = format!("run/{}", self.id);
                    fs::create_dir_all(&path)?;
                    let mut file = File::create(format!("{}/{}", &path, single_run.file))?;
                    file.write_all(&data)?;
                    if let Some(mut compile_command) = single_run.compile_command {
                        let p = compile_command
                            .current_dir(&path)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .expect("Failed to create compile process");
                        self.pid = p.id();
                        process = Some(p);
                        ws_writer
                            .send(Message::binary(vec![RunStatus::Compiling as u8, 0xe7]))
                            .await
                            .unwrap();
                    }
                    let mut run_command = single_run.run_command;
                    run_command.current_dir(&path);
                    cmd = run_command;
                    break;
                }
                None | Some(_) => (),
            }
        }
        if let Some(p) = process {
            let (_f1, _f2) = (ws_reader.next(), p.wait_with_output());
            pin_mut!(_f1, _f2);
            match future::select(_f1, _f2).await {
                future::Either::Left(_) => {
                    self.cleanup();
                    return Ok(ExitStatus::CompileInterrupted);
                }
                future::Either::Right((output, _)) => {
                    let output = output?;
                    if !output.status.success() {
                        ws_writer
                            .send(Message::binary([output.stdout, vec![0xe1]].concat()))
                            .await?;
                        ws_writer
                            .send(Message::binary([output.stderr, vec![0xe2]].concat()))
                            .await?;
                        ws_writer
                            .send(Message::binary(vec![RunStatus::CompileError as u8, 0xe8]))
                            .await?;
                        return Ok(ExitStatus::CompileError);
                    }
                }
            }
        }
        let mut process = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        self.pid = process.id();
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
            .await?;
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
                    if status.success() {
                        RunStatus::Ok
                    } else {
                        RunStatus::RuntimeError
                    } as u8,
                    0xe7,
                ])
                .unwrap();
            sleep(Duration::from_millis(100)).await;
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

        Ok(self.cleanup())
    }

    fn cleanup(&self) -> ExitStatus {
        let mut s = System::new();
        s.refresh_specifics(RefreshKind::new().with_processes());
        if let Some(process) = s.get_process(self.pid.unwrap() as i32) {
            process.kill(Signal::Kill);
            return ExitStatus::Killed;
        }
        ExitStatus::Done
    }
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

enum ExitStatus {
    CompileError,
    CompileInterrupted,
    Killed,
    Done,
}

enum RunStatus {
    Running = 0,
    Compiling,
    CompileError,
    Ok,
    RuntimeError,
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
        let mut run = Run::new();
        let (_a, _b) = future::join(test_server_ok(listener), run.run(&url)).await;
        _a.unwrap();
        _b.unwrap();
    }

    #[tokio::test]
    async fn integration_test_kill() {
        let (listener, url) = before_test().await;
        let mut run = Run::new();
        let (_a, _b) = future::join(test_server_kill(listener), run.run(&url)).await;
        _a.unwrap();
        assert!(matches!(_b.unwrap(), ExitStatus::Killed));
    }

    #[tokio::test]
    async fn integration_test_kill_compile() {
        let (listener, url) = before_test().await;
        let mut run = Run::new();
        let (_a, _b) = future::join(test_server_kill_compile(listener), run.run(&url)).await;
        _a.unwrap();
        assert!(matches!(_b.unwrap(), ExitStatus::CompileInterrupted));
    }

    #[tokio::test]
    async fn integration_test_fail_compile() {
        let (listener, url) = before_test().await;
        let mut run = Run::new();
        let (_a, _b) = future::join(test_server_fail_compile(listener), run.run(&url)).await;
        _a.unwrap();
        assert!(matches!(_b.unwrap(), ExitStatus::CompileError));
    }

    async fn before_test() -> (tokio::net::TcpListener, url::Url) {
        for port in 1025u16..65535u16 {
            let addr = format!("ws://127.0.0.1:{}", port);
            match tokio::net::TcpListener::bind(&addr[5..]).await {
                Ok(l) => return (l, Url::parse(&addr).unwrap()),
                _ => (),
            }
        }
        panic!("No port available, failed to bind")
    }

    async fn test_server_ok(listener: tokio::net::TcpListener) -> Result<(), Error> {
        let (stream, _) = listener.accept().await?;
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();
        write
            .send(Message::binary(b"0123456789abcdefC++\xc0".as_ref()))
            .await?;
        write.send(Message::binary(b"#include<iostream>\nusing namespace std;int main(){int n;cin>>n;cout<<n+1<<endl;cerr<<n+2<<endl;}\n\xc1".as_ref())).await?;
        assert_eq!(
            Message::binary(vec![0x01, 0xe7]),
            read.next().await.unwrap().unwrap()
        );
        assert_eq!(
            Message::binary(vec![0x00, 0xe7]),
            read.next().await.unwrap().unwrap()
        );
        write.send(Message::binary(b"1\n\xe0".as_ref())).await?;
        let following = [
            read.next().await.unwrap().unwrap(),
            read.next().await.unwrap().unwrap(),
            read.next().await.unwrap().unwrap(),
            read.next().await.unwrap().unwrap(),
        ];
        assert!(following.contains(&Message::binary(b"2\n\xe1".as_ref())));
        assert!(following.contains(&Message::binary(b"3\n\xe2".as_ref())));
        assert!(following.contains(&Message::binary(vec![0x03, 0xe7])));
        assert!(following.contains(&Message::binary(vec![0x00, 0xe8])));
        write.close().await?;
        Ok(())
    }

    async fn test_server_kill(listener: tokio::net::TcpListener) -> Result<(), Error> {
        let (stream, _) = listener.accept().await?;
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();
        write
            .send(Message::binary(b"0123456789abcdeeC++\xc0".as_ref()))
            .await?;
        write
            .send(Message::binary(
                b"#include<iostream>\nusing namespace std;int main(){int n;cin>>n;}\n\xc1".as_ref(),
            ))
            .await?;
        assert_eq!(
            Message::binary(vec![0x01, 0xe7]),
            read.next().await.unwrap().unwrap()
        );
        assert_eq!(
            Message::binary(vec![0x00, 0xe7]),
            read.next().await.unwrap().unwrap()
        );
        write.close().await?;
        Ok(())
    }

    async fn test_server_kill_compile(listener: tokio::net::TcpListener) -> Result<(), Error> {
        let (stream, _) = listener.accept().await?;
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut write, _) = ws_stream.split();
        write
            .send(Message::binary(b"0123456789abcdedC++\xc0".as_ref()))
            .await?;
        write
            .send(Message::binary(
                b"#include<iostream>\nusing namespace std;int main(){}\n\xc1".as_ref(),
            ))
            .await?;
        write.close().await?;
        Ok(())
    }

    async fn test_server_fail_compile(listener: tokio::net::TcpListener) -> Result<(), Error> {
        let (stream, _) = listener.accept().await?;
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();
        write
            .send(Message::binary(b"0123456789abcdecC++\xc0".as_ref()))
            .await?;
        write
            .send(Message::binary(
                b"#include<iostream>using namespace std;int main(){}\n\xc1".as_ref(),
            ))
            .await?;
        sleep(Duration::from_millis(1000)).await;
        assert_eq!(
            Message::binary(vec![0x01, 0xe7]),
            read.next().await.unwrap().unwrap()
        );
        Ok(())
    }
}
