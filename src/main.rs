use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEventKind},
    execute, queue,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

macro_rules! line {
    ($out:expr) => {{
        write!($out, "\r\n")?;
    }};
    ($out:expr, $($arg:tt)*) => {{
        write!($out, $($arg)*)?;
        write!($out, "\r\n")?;
    }};
}

mod app;
mod cpu;
mod monitor;

use app::App;

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;

        if let Err(err) = execute!(io::stdout(), EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(err);
        }

        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
                         LeaveAlternateScreen,
                         Clear(ClearType::All),
                         MoveTo(0, 0)
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut previous_cpu = monitor::read_cpu_stats()?.cpu_time;
    let mut app = App::new();

    let _terminal = TerminalGuard::enter()?;

    run(&mut app, &mut previous_cpu)?;

    Ok(())
}

fn run(
    app: &mut App,
    previous_cpu: &mut Vec<procfs::CpuTime>,
) -> Result<(), Box<dyn std::error::Error>> {
    while app.is_running() {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    app.quit();
                    continue;
                }
            }
        }

        thread::sleep(Duration::from_millis(400));

        match monitor::snapshot(previous_cpu) {
            Ok((snapshot, current_cpu)) => {
                app.update(snapshot);
                *previous_cpu = current_cpu;
            }
            Err(err) => {
                app.set_error(err.to_string());

                if matches!(&err, monitor::MonitorError::CalculateCpu(_)) {
                    match monitor::read_cpu_stats() {
                        Ok(stats) => *previous_cpu = stats.cpu_time,
                        Err(recovery_err) => {
                            app.set_error(format!(
                                "{err}; CPU resynchronization failed: {recovery_err}"
                            ));
                        }
                    }
                }
            }
        }

        render(app)?;
    }

    Ok(())
}

fn render(app: &App) -> io::Result<()> {
    let mut stdout = io::stdout();

    queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;

    line!(stdout, "ProcScope");
    line!(stdout, "Press q to quit");
    line!(stdout);

    if let Some(err) = &app.last_error {
        line!(stdout, "Monitoring error: {err}");
        line!(stdout);
    }

    if let Some(snapshot) = &app.snapshot {
        for (index, usage) in snapshot.cpu_usage.iter().enumerate() {
            line!(stdout, "CPU {index}: {usage:5.1}%");
        }

        let memory = &snapshot.memory;

        line!(stdout);
        line!(stdout, "Memory");
        line!(
            stdout,
            "RAM: {:.1} / {:.1} MiB ({:.1}%)",
              memory.used as f64 / 1_048_576.0,
              memory.total as f64 / 1_048_576.0,
              percentage(memory.used, memory.total),
        );
        line!(
            stdout,
            "Available: {:.1} MiB",
            memory.available as f64 / 1_048_576.0,
        );
        line!(
            stdout,
            "Swap: {:.1} / {:.1} MiB ({:.1}%)",
              memory.swap_used as f64 / 1_048_576.0,
              memory.swap_total as f64 / 1_048_576.0,
              percentage(memory.swap_used, memory.swap_total),
        );
    } else {
        line!(stdout, "Waiting for the first valid system snapshot...");
    }

    stdout.flush()
}

fn percentage(used: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        used as f64 / total as f64 * 100.0
    }
}
