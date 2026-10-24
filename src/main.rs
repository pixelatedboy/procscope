use procfs::CurrentSI;
use std::{thread, time::Duration};

mod cpu;
mod monitor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut previous = procfs::KernelStats::current()?.cpu_time;

    loop {
        thread::sleep(Duration::from_millis(500));

        let current = procfs::KernelStats::current()?.cpu_time;
        let usages = cpu::usage(&previous, &current)?;
        let memory = monitor::read_memory()?;

        print!("\x1B[2J\x1B[H");

        println!("\nCPU");
        for (index, usage) in usages.iter().enumerate() {
            println!("CPU {index}: {usage:5.1}%");
        }

        println!("\nMemory");
        println!(
            "RAM:       {:>7.1} / {:>7.1} MiB ({:5.1}%)",
                 memory.used as f64 / 1_048_576.0,
                 memory.total as f64 / 1_048_576.0,
                 percentage(memory.used, memory.total),
        );

        println!(
            "Available: {:>7.1} MiB",
            memory.available as f64 / 1_048_576.0,
        );

        println!(
            "Swap:      {:>7.1} / {:>7.1} MiB ({:5.1}%)",
                 memory.swap_used as f64 / 1_048_576.0,
                 memory.swap_total as f64 / 1_048_576.0,
                 percentage(memory.swap_used, memory.swap_total),
        );

        previous = current;
    }
}

fn percentage(used: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        used as f64 / total as f64 * 100.0
    }
}
