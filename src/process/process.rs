use std::cell::RefCell;
use std::process::Stdio;
use std::sync::Arc;
use std::thread::spawn;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

pub async fn run(program: &str, args: Vec<&str>, dir: &str, tx: tokio::sync::mpsc::Sender<OutputPacket>, mut rx: tokio::sync::mpsc::Receiver<InputPacket>) {
    let child = Arc::new(Mutex::new(RefCell::new(
        tokio::process::Command::new(program)
            .args(args)
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .expect("Cannot spawn child process"),
    )));


    let out = child.lock().await.get_mut().stdout.take().expect("Cannot take stdout");
    let err = child.lock().await.get_mut().stderr.take().expect("Cannot take stderr");
    let mut stdin = child.lock().await.get_mut().stdin.take().expect("Cannot take stdin");
    let mut ready = true;

    let tx1 = tx.clone();
    let child1 = child.clone();
    let h1 = spawn(move || async move {
        let mut out = tokio::io::BufReader::new(out);
        let mut s = String::new();

        loop {
            out.read_line(&mut s).await.unwrap();
            if !ready {
                return;
            }
            if let Ok(Some(_)) = child1.lock().await.get_mut().try_wait() {
                ready = false;
                tx1.send(OutputPacket {
                    output_type: OutputType::ActionResult,
                    message: "Stopped".to_string(),
                }).await.unwrap();
                return;
            }

            if !s.is_empty() {
                let ss = match s.strip_suffix('\n') {
                    None => { s }
                    Some(a) => { a.to_string() }
                };
                tx1.send(OutputPacket {
                    output_type: OutputType::Stdout,
                    message: ss.clone(),
                }).await.unwrap();
                // println!("'{}'", ss);
                s = "".to_string();
            }
        }
    });

    let tx2 = tx.clone();
    let h2 = spawn(move || async move {
        let mut out = tokio::io::BufReader::new(err);
        let mut s = String::new();

        loop {
            out.read_line(&mut s).await.unwrap();
            if !ready {
                return;
            }

            if !s.is_empty() {
                let ss = match s.strip_suffix('\n') {
                    None => { s }
                    Some(a) => { a.to_string() }
                };
                // println!("'{}'", ss);
                tx2.send(OutputPacket {
                    output_type: OutputType::Stderr,
                    message: ss.clone(),
                }).await.unwrap();
                s = "".to_string();
            }
        }
    });

    let h3 = spawn(move || async move {
        while let input = rx.recv().await {
            println!("{:?}", input);
            let input = input.unwrap();
            println!("{}", serde_json::to_string_pretty(&input).unwrap());
            if !ready {
                println!("exit");
                return;
            }
            match input.input_type {
                InputType::Command => {
                    stdin.write_all(format!("{}\n", input.message).as_bytes()).await.unwrap();
                }
                InputType::Action => {
                    match input.message.as_str() {
                        "stop" => {
                            stdin.write_all("stop\n".as_bytes()).await.unwrap();
                            tx.send(OutputPacket {
                                output_type: OutputType::ActionResult,
                                message: "Stopping".to_string(),
                            }).await.unwrap();

                            tokio::time::sleep(Duration::new(10, 0)).await;
                            ready = false;
                            child.lock().await.get_mut().kill().await.unwrap();

                            tx.send(OutputPacket {
                                output_type: OutputType::ActionResult,
                                message: "Stopped".to_string(),
                            }).await.unwrap();
                            println!("exit");
                            return;
                        }
                        _ => {
                            tx.send(OutputPacket {
                                output_type: OutputType::ActionResult,
                                message: "Unknown action".to_string(),
                            }).await.unwrap();
                        }
                    }
                }
            }
        }

        println!("exit");
    });

    h1.join().unwrap().await;
    ready = false;
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InputPacket {
    pub input_type: InputType,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum InputType {
    Command,
    Action,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OutputPacket {
    pub output_type: OutputType,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum OutputType {
    Stdout,
    Stderr,
    ActionResult,
}