mod process;

use tokio::spawn;
use tokio::task::JoinHandle;
use crate::process::process::{InputPacket, run};

#[tokio::main]
async fn main() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let (tx2, mut rx2) = tokio::sync::mpsc::channel(100);

    let h1: JoinHandle<()> = spawn(async move {
        run(
            "C:\\Program Files\\Java\\jdk-17.0.2\\bin\\java.exe",
            vec!["-jar", "./paper.jar", "nogui"],
            "./program/",
            tx,
            rx2,
        ).await;
    });

    let h2: JoinHandle<()> = spawn(async move {
        loop {
            let mut s = String::new();
            std::io::stdin().read_line(&mut s).unwrap();
            let packet = InputPacket {
                input_type: crate::process::process::InputType::Command,
                message: s,
            };
            tx2.send(packet).await.unwrap();
        }
    });

    let h3: JoinHandle<()> = spawn(async move {
        while let Some(a) = rx.recv().await {
            println!("{}", serde_json::to_string_pretty(&a).unwrap());
        }
    });

    tokio::try_join!(h1, h2, h3).unwrap();
}

