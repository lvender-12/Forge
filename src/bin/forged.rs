use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    process::{Child, Command},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

use chrono::Local;
use forge::{
    common::protocol::{Request, Response},
    error::{ForgeError, Result},
    model::Process,
    storage,
};
use nix::{
    sys::signal::{SaFlags, SigAction, SigHandler, SigSet, Signal, kill, sigaction},
    sys::wait::{WaitPidFlag, WaitStatus, waitpid},
    unistd::Pid,
};

const SOCKET_PATH: &str = "/tmp/forge.sock";
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

fn process_is_alive(pid: Pid) -> bool {
    match waitpid(pid, Some(WaitPidFlag::WNOHANG)) {
        Ok(WaitStatus::StillAlive) => true,
        Ok(_) => false,
        Err(nix::errno::Errno::ECHILD) => kill(pid, None).is_ok(),
        Err(_) => kill(pid, None).is_ok(),
    }
}

extern "C" fn request_shutdown(_: i32) {
    SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
}

fn install_shutdown_handlers() -> Result<()> {
    let action = SigAction::new(
        SigHandler::Handler(request_shutdown),
        SaFlags::empty(),
        SigSet::empty(),
    );
    unsafe {
        sigaction(Signal::SIGINT, &action)?;
        sigaction(Signal::SIGTERM, &action)?;
    }
    Ok(())
}

fn spawn_command(command: &str) -> Result<Child> {
    if command.trim().is_empty() {
        return Err(ForgeError::EmptyCommand);
    }

    let mut child = Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("exec {command}"))
        .spawn()?;

    let deadline = Instant::now() + Duration::from_millis(300);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(ForgeError::CommandExited {
                    command: command.to_string(),
                    status: status.to_string(),
                });
            }
            Ok(None) if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(10));
            }
            Ok(None) => return Ok(child),
            Err(error) => {
                let _ = child.kill();
                return Err(error.into());
            }
        }
    }
}

fn mold(name: String, executable: String) -> Result<String> {
    let mut processes = storage::get_process()?;
    if processes
        .processed
        .iter()
        .any(|process| process.name == name)
    {
        return Err(ForgeError::DuplicateProcess(name));
    }

    let child = spawn_command(&executable)?;
    processes.processed.push(Process {
        name,
        pid: Some(child.id()),
        executable,
        status: "Running".to_string(),
        started_at: Some(Local::now().to_rfc3339()),
    });
    if let Err(error) = storage::save(&processes) {
        let _ = kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM);
        return Err(error);
    }

    Ok("Process molded and started".to_string())
}

fn ignite(name: String) -> Result<String> {
    let mut processes = storage::get_process()?;
    let process = processes
        .processed
        .iter_mut()
        .find(|process| process.name == name)
        .ok_or_else(|| ForgeError::ProcessNotFound(name.clone()))?;

    let child = spawn_command(&process.executable)?;
    let pid = child.id();

    process.pid = Some(pid);
    process.status = "Running".to_string();
    process.started_at = Some(Local::now().to_rfc3339());
    if let Err(error) = storage::save(&processes) {
        let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
        return Err(error);
    }

    Ok(format!("Process `{name}` started"))
}

fn quench(name: String) -> Result<String> {
    let mut processes = storage::get_process()?;
    let process = processes
        .processed
        .iter_mut()
        .find(|process| process.name == name)
        .ok_or_else(|| ForgeError::ProcessNotFound(name.clone()))?;

    let pid = process
        .pid
        .ok_or_else(|| ForgeError::ProcessNotRunning(name.clone()))?;

    kill(Pid::from_raw(pid as i32), Signal::SIGTERM)?;

    process.pid = None;
    process.status = "Stopped".to_string();
    storage::save(&processes)?;

    Ok(format!("Process `{name}` stopped"))
}

fn smelt(name: String) -> Result<String> {
    let mut processes = storage::get_process()?;
    let index = processes
        .processed
        .iter()
        .position(|process| process.name == name)
        .ok_or_else(|| ForgeError::ProcessNotFound(name.clone()))?;
    if let Some(pid) = processes.processed[index].pid {
        kill(Pid::from_raw(pid as i32), Signal::SIGTERM)?;
    }
    processes.processed.remove(index);
    storage::save(&processes)?;
    Ok(format!("Process `{name}` removed"))
}

fn dispatch(request: Request) -> Result<Response> {
    match request {
        Request::Mold { name, executable } => Ok(Response::Success {
            message: mold(name, executable)?,
        }),
        Request::Ignite { name } => Ok(Response::Success {
            message: ignite(name)?,
        }),
        Request::Quench { name } => Ok(Response::Success {
            message: quench(name)?,
        }),
        Request::Anvils => anvils(),
        Request::Smelt { name } => Ok(Response::Success {
            message: smelt(name)?,
        }),
    }
}

fn anvils() -> Result<Response> {
    let mut list = storage::get_process()?;
    let mut changed = false;
    for process in &mut list.processed {
        if let Some(pid) = process.pid {
            if !process_is_alive(Pid::from_raw(pid as i32)) {
                process.pid = None;
                process.status = "Exited".to_string();
                changed = true;
            }
        }
    }
    if changed {
        storage::save(&list)?;
    }
    Ok(Response::Anvils {
        processes: list.processed,
    })
}

fn handle_client(mut stream: UnixStream) -> Result<()> {
    let mut line = String::new();
    BufReader::new(&mut stream).read_line(&mut line)?;

    let response = match serde_json::from_str::<Request>(&line) {
        Ok(request) => dispatch(request).unwrap_or_else(|error| Response::Error {
            message: error.to_string(),
        }),
        Err(error) => Response::Error {
            message: format!("invalid request: {error}"),
        },
    };

    let mut payload = serde_json::to_vec(&response)?;
    payload.push(b'\n');
    stream.write_all(&payload)?;
    Ok(())
}

fn stop_managed_processes() -> Result<()> {
    let mut processes = storage::get_process()?;
    let pids: Vec<Pid> = processes
        .processed
        .iter()
        .filter_map(|process| process.pid)
        .map(|pid| Pid::from_raw(pid as i32))
        .collect();

    for pid in &pids {
        if let Err(error) = kill(*pid, Signal::SIGTERM) {
            if error != nix::errno::Errno::ESRCH {
                eprintln!("Could not gracefully stop process {}: {error}", pid);
            }
        }
    }

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut remaining = pids.clone();
    while !remaining.is_empty() && Instant::now() < deadline {
        remaining.retain(|pid| process_is_alive(*pid));
        if !remaining.is_empty() {
            thread::sleep(Duration::from_millis(100));
        }
    }

    for pid in &remaining {
        eprintln!(
            "Process {} did not exit after SIGTERM; sending SIGKILL",
            pid
        );
        if let Err(error) = kill(*pid, Signal::SIGKILL) {
            if error != nix::errno::Errno::ESRCH {
                eprintln!("Could not force stop process {}: {error}", pid);
            }
        }
    }

    let deadline = Instant::now() + Duration::from_secs(1);
    while !remaining.is_empty() && Instant::now() < deadline {
        remaining.retain(|pid| process_is_alive(*pid));
        if !remaining.is_empty() {
            thread::sleep(Duration::from_millis(50));
        }
    }

    let mut stopped_count = 0;
    let mut unresolved_count = 0;
    for process in &mut processes.processed {
        if let Some(pid) = process.pid {
            if remaining.contains(&Pid::from_raw(pid as i32)) {
                process.status = "Stop failed".to_string();
                unresolved_count += 1;
            } else {
                process.pid = None;
                process.status = "Stopped".to_string();
                stopped_count += 1;
            }
        }
    }
    storage::save(&processes)?;
    println!("Stopped {stopped_count} managed process(es) and saved process.json");
    if unresolved_count > 0 {
        eprintln!(
            "Could not confirm shutdown of {unresolved_count} process(es); their PIDs remain in process.json"
        );
    }
    Ok(())
}

pub fn main() -> Result<()> {
    if Path::new(SOCKET_PATH).exists() {
        return Err(ForgeError::Io(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "socket path already exists; check whether daemon is running",
        )));
    }

    install_shutdown_handlers()?;
    let listener = UnixListener::bind(SOCKET_PATH)?;
    listener.set_nonblocking(true)?;
    println!("Daemon listening on {SOCKET_PATH}");

    while !SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                if let Err(error) = handle_client(stream) {
                    eprintln!("Request connection failed: {error}");
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(error) => eprintln!("Connection failed: {error}"),
        }
    }

    let shutdown_result = stop_managed_processes();
    let socket_result = fs::remove_file(SOCKET_PATH);
    shutdown_result?;
    socket_result?;
    Ok(())
}
