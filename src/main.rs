mod monitor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = monitor::read_memory()?;
    let cpu = monitor::read_cpu_stats()?;

    println!("Memory: {memory:#?}");
    println!("CPU stats: {:#?}", cpu.cpu_time);

    Ok(())
}
