use forge::{
    cli::{Command, parse},
    client::send,
    common::protocol::{Request, Response},
    model::Process,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = match parse() {
        Command::Mold {
            name,
            _exec: _,
            command,
        } => Request::Mold {
            name,
            executable: command,
            working_dir: std::env::current_dir()?.to_string_lossy().into_owned(),
        },
        Command::Ignite { name } => Request::Ignite { name },
        Command::Quench { name } => Request::Quench { name },
        Command::Anvils => Request::Anvils,
        Command::Smelt { name } => Request::Smelt { name },
    };

    let response = send(request)?;
    match response {
        Response::Success { message } => {
            println!("{message}");
            Ok(())
        }
        Response::Anvils { processes } => {
            print_anvils(&processes);
            Ok(())
        }
        Response::Error { message } => Err(message.into()),
    }
}

fn print_anvils(processes: &[Process]) {
    println!("\n  FORGE  /  ANVILS\n");
    if processes.is_empty() {
        println!("  No managed processes yet. Add one with: forge mold NAME exec COMMAND\n");
        return;
    }

    let headings = [
        "NAME",
        "STATUS",
        "PID",
        "STARTED",
        "WORKING DIRECTORY",
        "COMMAND",
    ];
    let mut rows: Vec<[String; 6]> = processes
        .iter()
        .map(|process| {
            [
                truncate(&process.name, 24),
                truncate(&process.status, 12),
                process
                    .pid
                    .map_or_else(|| "-".to_string(), |pid| pid.to_string()),
                process.started_at.as_deref().map_or_else(
                    || "-".to_string(),
                    |started| truncate(&started.replace('T', " "), 19),
                ),
                truncate(&process.working_dir, 48),
                truncate(&process.executable, 64),
            ]
        })
        .collect();

    let widths: [usize; 6] = std::array::from_fn(|column| {
        rows.iter()
            .map(|row| row[column].chars().count())
            .chain(std::iter::once(headings[column].chars().count()))
            .max()
            .unwrap_or(0)
    });
    let border = format!(
        "┌{}┬{}┬{}┬{}┬{}┬{}┐",
        "─".repeat(widths[0] + 2),
        "─".repeat(widths[1] + 2),
        "─".repeat(widths[2] + 2),
        "─".repeat(widths[3] + 2),
        "─".repeat(widths[4] + 2),
        "─".repeat(widths[5] + 2),
    );
    let divider = format!(
        "├{}┼{}┼{}┼{}┼{}┼{}┤",
        "─".repeat(widths[0] + 2),
        "─".repeat(widths[1] + 2),
        "─".repeat(widths[2] + 2),
        "─".repeat(widths[3] + 2),
        "─".repeat(widths[4] + 2),
        "─".repeat(widths[5] + 2),
    );
    let bottom = format!(
        "└{}┴{}┴{}┴{}┴{}┴{}┘",
        "─".repeat(widths[0] + 2),
        "─".repeat(widths[1] + 2),
        "─".repeat(widths[2] + 2),
        "─".repeat(widths[3] + 2),
        "─".repeat(widths[4] + 2),
        "─".repeat(widths[5] + 2),
    );

    println!("{border}");
    print_row(&headings.map(str::to_string), &widths);
    println!("{divider}");
    for row in rows.drain(..) {
        print_row(&row, &widths);
    }
    println!("{bottom}\n");
}

fn print_row(row: &[String; 6], widths: &[usize; 6]) {
    print!("│");
    for column in 0..6 {
        print!(" {:width$} │", row[column], width = widths[column]);
    }
    println!();
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let shortened: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        let mut shortened: String = shortened
            .chars()
            .take(max_chars.saturating_sub(1))
            .collect();
        shortened.push('…');
        shortened
    } else {
        shortened
    }
}
