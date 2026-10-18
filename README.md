# Forge

**A lightweight Linux process manager for self-hosted applications — without containers.**

Forge starts, stops, and monitors applications directly on a Linux host through a CLI and a background daemon. It is designed for self-hosted APIs, bots, workers, and scripts that do not need container isolation.

Forge is not a Docker replacement. It is a simpler option for managing applications that do not require containers.

## Features

- Manage processes with the `forge` command-line client.
- Run the daemon in the foreground or as a systemd user service.
- Store process names, commands, working directories, PIDs, and status in `data/process.json`.
- Stop managed processes when the daemon receives `SIGINT` or `SIGTERM`.
- Reject commands that exit during startup instead of recording them as running.

## Requirements

- Linux
- Rust and Cargo
- systemd, if you want Forge to start automatically when you log in

## Build

From the project directory, build both binaries in release mode:

```bash
cargo build --release --bin forge --bin forged
```

The binaries are created here:

```text
target/release/forge
target/release/forged
```

## Run the daemon

Start the daemon from the project directory:

```bash
./target/release/forged
```

Keep it running in this terminal, or configure the systemd user service below. The daemon currently stores its process data relative to its working directory, so start it from the Forge project directory.

## Install the client for your user

Install the client under `~/.local/bin` so it can be called from any directory:

```bash
install -Dm755 target/release/forge "$HOME/.local/bin/forge"
```

Add that directory to your shell's `PATH` if needed.

For Fish:

```fish
fish_add_path ~/.local/bin
```

For Bash, add this line to `~/.bashrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

For Zsh, add the same line to `~/.zshrc`. Open a new terminal, then check the command is available:

```bash
forge --help
```

## Run Forge automatically with systemd

Create `~/.config/systemd/user/forge.service`. Replace the example project path with the actual path to your Forge checkout:

```ini
[Unit]
Description=Forge process manager
After=default.target

[Service]
Type=simple
WorkingDirectory=/home/your-user/path/to/forge
ExecStart=/home/your-user/path/to/forge/target/release/forged
Restart=on-failure
RestartSec=2

[Install]
WantedBy=default.target
```

Load and enable the service:

```bash
systemctl --user daemon-reload
systemctl --user enable --now forge.service
```

The user service starts when you log in. To keep it running from boot even before you log in, enable lingering for your account:

```bash
loginctl enable-linger "$USER"
```

Useful service commands:

```bash
systemctl --user status forge.service
systemctl --user restart forge.service
systemctl --user stop forge.service
journalctl --user -u forge.service -f
```

The unit uses absolute paths, so update it if you move the Forge project. Stop any manually started daemon before enabling the service; only one daemon can listen on Forge's socket at a time.

## Commands

Run `forge --help` to see the available commands.

### Register and start a process

```bash
forge mold "anime" exec "./anime"
```

Run this command from the application's working directory. Forge records that directory and uses it when starting the application. You can also provide a command with arguments:

```bash
forge mold "web" exec "python3 -m http.server 8000"
```

The application must exist and remain running through its startup check; failed commands are not added to the process list.

### Show managed processes

```bash
forge anvils
```

The table shows each process's status, PID, start time, working directory, and command.

### Stop and start a process

```bash
forge quench "anime"
forge ignite "anime"
```

`quench` sends `SIGTERM`, waits for the process to exit, and escalates to `SIGKILL` if needed. `ignite` runs the saved command again in its saved working directory.

### Stop and remove a process

```bash
forge smelt "anime"
```

`smelt` stops the process if it has a PID, then removes its entry from the process list.

Forge does not currently have a separate `restart` command. Use `quench` followed by `ignite`.

## Data and socket

- Process records are stored in `data/process.json`, relative to the daemon's working directory.
- The client and daemon communicate over `/tmp/forge.sock`.
- The daemon manages processes as the same user that runs the daemon. Do not run it with `sudo` unless you intend the managed applications to run as root.

## Current packaging status

Forge does not yet provide an Arch package or installable package for other distributions. The systemd user unit above is a manual setup for a source checkout. A distributable package should install the binaries and service unit in standard system locations and use user-specific state and runtime paths instead of the current checkout-relative data path and fixed socket path.
