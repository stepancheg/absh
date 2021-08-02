use std::process::Child;
use std::process::Command;
use std::process::Stdio;

pub fn spawn_sh(script: &str) -> Child {
    Command::new("/bin/sh")
        .args(&["-ec", &script])
        .stdin(Stdio::null())
        .spawn()
        .expect("launch /bin/sh")
}
