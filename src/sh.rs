use std::process::Child;
use std::process::Command;
use std::process::Stdio;

#[cfg(not(windows))]
pub fn spawn_sh(script: &str) -> anyhow::Result<Child> {
    Ok(Command::new("/bin/sh")
        .args(&["-c", &script])
        .stdin(Stdio::null())
        .spawn()?)
}

#[cfg(windows)]
pub fn spawn_sh(script: &str) -> anyhow::Result<Child> {
    Ok(Command::new("powershell.exe")
        .args(&["-Command", &script])
        .stdin(Stdio::null())
        .spawn()?)
}
