fn main() {
    println!("Hello, world!");
}

// use std::process::{Command, Stdio};
//
// fn main() {
//     let mut child = Command::new("cmd")
//         .stdin(Stdio::piped())
//         .stdout(Stdio::piped())
//         .spawn()
//         .expect("err");
//     let mut child_stdin = child.stdin.take().expect("stdin err");
//     //不会咯（？）
// }