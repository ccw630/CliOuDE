use futures::{future, pin_mut, stream, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;
use sysinfo::{ProcessExt, ProcessorExt, Signal, System, SystemExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::{sleep, timeout, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::Error};
use url::Url;

#[tokio::main]
async fn main() {
    let connect_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires at least one argument"));
    let language_conf_path = env::args().nth(2).unwrap_or("languages.json".to_string());

    let url = Url::parse(&connect_addr).unwrap();
    // loop {
    let mut runner = Run::new(&language_conf_path);
    match runner.run(&url).await {
        Ok(_) => (),
        Err(err) => {
            runner.cleanup();
            println!("FATAL: Run {} ERROR {:?}", runner.id, err);
        }
    }

    // }
}

struct Run {
    id: String,
    pid: Option<u32>,
    // TODO Immutable
    _language_conf: HashMap<String, Language>,
}

impl Run {
    fn new(language_conf_path: &str) -> Run {
        Run {
            id: String::new(),
            pid: None,
            _language_conf: serde_json::from_str(
                &fs::read_to_string(language_conf_path).expect("Load language conf failed!"),
            )
            .expect("Extract language conf failed!"),
        }
    }

    async fn run(&mut self, url: &Url) -> Result<ExitStatus, Error> {
        let (ws_stream, _) = connect_async(url).await?;
        let (mut ws_writer, mut ws_reader) = ws_stream.split();
        let mut language: Option<String> = None;
        let mut process: Option<tokio::process::Child> = None;
        let mut cmd: Command;
        let mut sys = System::new();
        let idle_timeout: u64;
        let available_languages = stream::iter(&self._language_conf)
            .filter_map(|(k, v)| async move {
                if let Ok(check) = get_command(&v.check_command)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    let output = check.wait_with_output().await;
                    if let Ok(output) = output {
                        if output.status.success() {
                            return Some((
                                k.to_owned(),
                                format!(
                                    "{}{}",
                                    String::from_utf8_lossy(&output.stdout).into_owned(),
                                    String::from_utf8_lossy(&output.stderr).into_owned()
                                ),
                            ));
                        }
                    }
                }
                return None;
            })
            .collect::<HashMap<String, String>>()
            .await;

        sys.refresh_memory();
        let env_info = EnvInfo {
            cpu_freq: sys.get_global_processor_info().get_frequency(),
            total_memory: sys.get_total_memory(),
            available_languages,
        };
        ws_writer
            .send(Message::binary(serde_json::to_string(&env_info).unwrap()))
            .await?;
        loop {
            let message = match ws_reader.next().await {
                Some(message) => message,
                None => return Ok(ExitStatus::StartInterrupted),
            };
            let mut data = message?.into_data();
            match data.pop() {
                Some(0xc0u8) => {
                    self.id = String::from_utf8_lossy(&data[..36]).into_owned();
                    language = Some(String::from_utf8_lossy(&data[36..]).into_owned());
                }
                Some(0xc1u8) => {
                    let lang = match language {
                        Some(lang) => self._language_conf.get(&lang).expect("No such language"),
                        None => panic!("No language received"),
                    };
                    idle_timeout = lang.idle_timeout;
                    let path = format!("run/{}", self.id);
                    fs::create_dir_all(&path)?;
                    let mut file = File::create(format!("{}/{}", &path, lang.file))?;
                    file.write_all(&data)?;
                    if let Some(prepare_command_raw) = &lang.prepare_command {
                        let mut prepare_command = get_command(prepare_command_raw);
                        let p = prepare_command
                            .current_dir(&path)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()?;
                        self.pid = p.id();
                        process = Some(p);
                        ws_writer
                            .send(Message::binary(vec![RunStatus::Preparing as u8, 0xe7]))
                            .await?;
                    }
                    let mut run_command = get_command(&lang.run_command);
                    run_command.current_dir(&path);
                    cmd = run_command;
                    break;
                }
                None | Some(_) => (),
            }
        }
        if let Some(p) = process {
            let (_f1, _f2) = (
                ws_reader.next(),
                timeout(Duration::from_secs(idle_timeout), p.wait_with_output()),
            );
            pin_mut!(_f1, _f2);
            match future::select(_f1, _f2).await {
                future::Either::Left(_) => {
                    self.cleanup();
                    return Ok(ExitStatus::PrepareInterrupted);
                }
                future::Either::Right((output, _)) => match output {
                    Ok(output) => {
                        let output = output?;
                        if !output.status.success() {
                            ws_writer
                                .send(Message::binary([output.stdout, vec![0xe1]].concat()))
                                .await?;
                            ws_writer
                                .send(Message::binary([output.stderr, vec![0xe2]].concat()))
                                .await?;
                            ws_writer
                                .send(Message::binary(vec![RunStatus::PrepareError as u8, 0xe7]))
                                .await?;
                            ws_writer
                                .send(Message::binary(vec![
                                    match output.status.code() {
                                        Some(code) => code,
                                        None => 9,
                                    } as u8,
                                    0xe8,
                                ]))
                                .await?;
                            return Ok(ExitStatus::PrepareError);
                        }
                    }
                    Err(_) => {
                        ws_writer
                            .send(Message::binary(vec![RunStatus::TimedOut as u8, 0xe7]))
                            .await?;
                        self.cleanup();
                        return Ok(ExitStatus::PrepareInterrupted);
                    }
                },
            }
        }
        let mut process = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let now = SystemTime::now();
        let timeout = Arc::new(Mutex::new(get_current_timestamp() + idle_timeout));
        let timeout_for_loop = Arc::clone(&timeout);
        let timeout_for_wait = Arc::clone(&timeout);
        self.pid = process.id();
        let mut stdin = process
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
        let (ws_tx, mut ws_rx) = futures::channel::mpsc::unbounded::<Vec<u8>>();
        let stdout_tx = ws_tx.clone();
        let stderr_tx = ws_tx.clone();
        let loop_tx = ws_tx.clone();
        tokio::spawn(pipe(stdout, stdout_tx, 0xe1));
        tokio::spawn(pipe(stderr, stderr_tx, 0xe2));
        let pid = self.pid.unwrap() as i32;
        tokio::spawn(async move {
            let mut times = 0;
            while sys.refresh_process(pid) {
                times += 1;
                if let Some(process) = sys.get_process(pid) {
                    if get_current_timestamp() > *timeout_for_loop.lock().unwrap() {
                        process.kill(Signal::Kill);
                        break;
                    }
                    loop_tx
                        .unbounded_send(
                            [
                                now.elapsed()
                                    .unwrap()
                                    .as_secs_f64()
                                    .to_bits()
                                    .to_be_bytes()
                                    .to_vec(),
                                process.cpu_usage().to_bits().to_be_bytes().to_vec(),
                                process.memory().to_be_bytes().to_vec(),
                                vec![0xe6],
                            ]
                            .concat(),
                        )
                        .unwrap();
                    sleep(Duration::from_millis(if times > 100 { 200 } else { 20 })).await;
                } else {
                    break;
                }
            }
        });
        tokio::spawn(async move {
            let status = process
                .wait()
                .await
                .expect("child process encountered an error");
            ws_tx
                .unbounded_send(
                    [
                        now.elapsed()
                            .unwrap()
                            .as_secs_f64()
                            .to_bits()
                            .to_be_bytes()
                            .to_vec(),
                        0f32.to_bits().to_be_bytes().to_vec(),
                        0u64.to_be_bytes().to_vec(),
                        vec![0xe6],
                    ]
                    .concat(),
                )
                .unwrap();
            if get_current_timestamp() > *timeout_for_wait.lock().unwrap() {
                ws_tx
                    .unbounded_send(vec![RunStatus::TimedOut as u8, 0xe7])
                    .unwrap();
            } else {
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
            }
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
        tokio::spawn(async move {
            while let Some(data) = ws_rx.next().await {
                let last = data.last().unwrap().clone();
                match ws_writer.send(Message::binary(data)).await {
                    Ok(_) => (),
                    Err(err) => println!("FATAL: {:?} at send message", err),
                };
                if last == 0xe8u8 {
                    // ws_writer.send(Message::Close(Some(CloseFrame {
                    //     code: CloseCode::Normal,
                    //     reason: "".into()
                    // }))).await.unwrap();
                    ws_writer.close().await.unwrap();
                }
            }
        });
        while let Some(message) = ws_reader.next().await {
            match message {
                Ok(message) => {
                    *timeout.lock().unwrap() = get_current_timestamp() + idle_timeout;
                    let mut data = message.into_data();
                    match data.pop() {
                        Some(0xe0u8) => {
                            stdin.write_all(&data).await?;
                            stdin.flush().await?
                        }
                        None | Some(_) => (),
                    };
                }
                Err(err) => {
                    println!("FATAL: {:?} at read message", err);
                    break;
                }
            }
        }
        Ok(self.cleanup())
    }

    fn cleanup(&self) -> ExitStatus {
        // todo remove dir
        let mut s = System::new();
        if let Some(p) = self.pid {
            let pid = p as i32;
            s.refresh_process(pid);
            if let Some(process) = s.get_process(pid) {
                process.kill(Signal::Kill);
                return ExitStatus::Killed;
            }
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

fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

enum ExitStatus {
    StartInterrupted,
    PrepareError,
    PrepareInterrupted,
    Killed,
    Done,
}

enum RunStatus {
    Running = 0,
    Preparing,
    PrepareError,
    Ok,
    RuntimeError,
    TimedOut,
}

#[derive(Serialize, Deserialize, Debug)]
struct EnvInfo {
    cpu_freq: u64,
    total_memory: u64,
    available_languages: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Language {
    file: String,
    check_command: String,
    run_command: String,
    prepare_command: Option<String>,
    idle_timeout: u64,
}

fn get_command(raw: &String) -> Command {
    let mut args = raw.split_ascii_whitespace();
    let mut command = Command::new(args.next().unwrap());
    for arg in args {
        command.arg(arg);
    }
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn integration_test_ok() {
        let (listener, url) = before_test().await;
        let mut run = Run::new("etc/languages.json");
        let (_a, _b) = future::join(test_server_ok(listener), run.run(&url)).await;
        _a.unwrap();
        _b.unwrap();
    }

    #[tokio::test]
    async fn integration_test_kill() {
        let (listener, url) = before_test().await;
        let mut run = Run::new("etc/languages.json");
        let (_a, _b) = future::join(test_server_kill(listener), run.run(&url)).await;
        _a.unwrap();
        assert!(matches!(_b.unwrap(), ExitStatus::Killed));
    }

    #[tokio::test]
    async fn integration_test_kill_prepare() {
        let (listener, url) = before_test().await;
        let mut run = Run::new("etc/languages.json");
        let (_a, _b) = future::join(test_server_kill_prepare(listener), run.run(&url)).await;
        _a.unwrap();
        assert!(matches!(_b.unwrap(), ExitStatus::PrepareInterrupted));
    }

    #[tokio::test]
    async fn integration_test_fail_prepare() {
        let (listener, url) = before_test().await;
        let mut run = Run::new("etc/languages.json");
        let (_a, _b) = future::join(test_server_fail_prepare(listener), run.run(&url)).await;
        _a.unwrap();
        assert!(matches!(_b.unwrap(), ExitStatus::PrepareError));
    }

    #[tokio::test]
    async fn integration_test_disconnect_on_start() {
        let (listener, url) = before_test().await;
        let mut run = Run::new("etc/languages.json");
        let (_a, _b) = future::join(test_server_disconnect_on_start(listener), run.run(&url)).await;
        _a.unwrap();
        assert!(matches!(_b.unwrap(), ExitStatus::StartInterrupted));
    }

    #[tokio::test]
    async fn integration_test_timeout() {
        let (listener, url) = before_test().await;
        let mut run = Run::new("etc/languages.json");
        run._language_conf.get_mut("C++").unwrap().idle_timeout = 1;
        let (_a, _b) = future::join(test_server_timeout(listener), run.run(&url)).await;
        _a.unwrap();
        assert!(matches!(_b.unwrap(), ExitStatus::Done));
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
        read.next().await;
        write
            .send(Message::binary(
                b"00000000-0000-0000-0000-000000000001C++\xc0".as_ref(),
            ))
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
        let following = read.map(|x| x.unwrap()).collect::<Vec<_>>().await;
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
        read.next().await;
        write
            .send(Message::binary(
                b"00000000-0000-0000-0000-000000000002C++\xc0".as_ref(),
            ))
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

    async fn test_server_kill_prepare(listener: tokio::net::TcpListener) -> Result<(), Error> {
        let (stream, _) = listener.accept().await?;
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();
        read.next().await;
        write
            .send(Message::binary(
                b"00000000-0000-0000-0000-000000000003C++\xc0".as_ref(),
            ))
            .await?;
        write
            .send(Message::binary(
                b"#include<iostream>\nusing namespace std;int main(){}\n\xc1".as_ref(),
            ))
            .await?;
        write.close().await?;
        Ok(())
    }

    async fn test_server_fail_prepare(listener: tokio::net::TcpListener) -> Result<(), Error> {
        let (stream, _) = listener.accept().await?;
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();
        read.next().await;
        write
            .send(Message::binary(
                b"00000000-0000-0000-0000-000000000004C++\xc0".as_ref(),
            ))
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
        write.close().await?;
        Ok(())
    }

    async fn test_server_disconnect_on_start(
        listener: tokio::net::TcpListener,
    ) -> Result<(), Error> {
        let (stream, _) = listener.accept().await?;
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();
        read.next().await;
        write
            .send(Message::binary(
                b"00000000-0000-0000-0000-000000000005C++\xc0".as_ref(),
            ))
            .await?;
        sleep(Duration::from_millis(1000)).await;
        write.close().await?;
        Ok(())
    }

    async fn test_server_timeout(listener: tokio::net::TcpListener) -> Result<(), Error> {
        let (stream, _) = listener.accept().await?;
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();
        read.next().await;
        write
            .send(Message::binary(
                b"00000000-0000-0000-0000-000000000001C++\xc0".as_ref(),
            ))
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
        let following = read.map(|x| x.unwrap()).collect::<Vec<_>>().await;
        assert!(following.contains(&Message::binary(vec![0x05, 0xe7])));
        write.close().await?;
        Ok(())
    }
}
