use procfs::CurrentSI;
use std::{thread, time::Duration};

mod cpu;
mod monitor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut previous_cpu = monitor::read_cpu_stats()?.cpu_time;

    loop {
        thread::sleep(Duration::from_millis(500));

        match monitor::snapshot(&previous_cpu) {
            Ok((snapshot, current_cpu)) => {
                print!("\x1B[2J\x1B[H");

                for (index, usage) in snapshot.cpu_usage.iter().enumerate() {
                    println!("CPU {index}: {usage:5.1}%");
                }

                let memory = &snapshot.memory;

                println!("\nMemory");
                println!(
                    "RAM: {:.1} / {:.1} MiB ({:.1}%)",
                         memory.used as f64 / 1_048_576.0,
                         memory.total as f64 / 1_048_576.0,
                         percentage(memory.used, memory.total),
                );

                println!(
                    "Available: {:.1} MiB",
                    memory.available as f64 / 1_048_576.0,
                );

                println!(
                    "Swap: {:.1} / {:.1} MiB ({:.1}%)",
                         memory.swap_used as f64 / 1_048_576.0,
                         memory.swap_total as f64 / 1_048_576.0,
                         percentage(memory.swap_used, memory.swap_total),
                );

                previous_cpu = current_cpu;
            }
            Err(err) => {
                eprintln!("ProcScope monitoring error: {err}");

                if matches!(err, monitor::MonitorError::CalculateCpu(_)) {
                    match monitor::read_cpu_stats() {
                        Ok(stats) => previous_cpu = stats.cpu_time,
                        Err(recovery_err) => {
                            eprintln!("Failed to resynchronize CPU counters: {recovery_err}");
                        }
                    }
                }
            }
        }
    }
}

fn percentage(used: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        used as f64 / total as f64 * 100.0
    }
}
