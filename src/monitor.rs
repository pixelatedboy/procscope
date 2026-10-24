use procfs::{Current, CurrentSI, KernelStats, Meminfo};

use crate::cpu;

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub memory: MemoryStats,
    pub cpu_usage: Vec<f64>,
}

pub fn read_memory() -> Result<MemoryStats, Box<dyn std::error::Error>> {
    let mem = Meminfo::current()?;

    let total = mem.mem_total;
    let available = mem.mem_available.unwrap_or(mem.mem_free);

    Ok(MemoryStats {
        total,
       used: total.saturating_sub(available),
       available,
       swap_total: mem.swap_total,
       swap_used: mem.swap_total.saturating_sub(mem.swap_free),
    })
}

pub fn read_cpu_stats() -> Result<KernelStats, Box<dyn std::error::Error>> {
    Ok(KernelStats::current()?)
}

pub fn snapshot(
    previous_cpu: &[procfs::CpuTime],
) -> Result<(SystemSnapshot, Vec<procfs::CpuTime>), Box<dyn std::error::Error>> {
    let stats = read_cpu_stats()?;
    let current_cpu = stats.cpu_time;
    let cpu_usage = cpu::usage(previous_cpu, &current_cpu)?;
    let memory = read_memory()?;

    let snapshot = SystemSnapshot {
        memory,
        cpu_usage,
    };

    Ok((snapshot, current_cpu))
}
