use nix::{
    sys::signal::{self, kill},
    unistd::Pid,
};
use std::{io, process::Command};

fn main() -> io::Result<()> {
    println!("Forge starting child process");

    let mut child = Command::new("sleep").arg("5").spawn()?;

    let pid = Pid::from_raw(child.id() as i32);

    println!("child pid {}", child.id());
    println!("Forge waiting child process");

    kill(pid, signal::SIGTERM)?;

    let status = child.wait()?;

    println!("Child exited {status}");

    Ok(())
}
