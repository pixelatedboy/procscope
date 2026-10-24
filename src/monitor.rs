use procfs::{Current, CurrentSI, KernelStats, Meminfo};
use std::{error::Error, fmt};

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

#[derive(Debug)]
pub enum MonitorError {
    ReadCpu(Box<dyn Error>),
    CalculateCpu(cpu::CpuUsageError),
    ReadMemory(Box<dyn Error>),
}

impl fmt::Display for MonitorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadCpu(err) => write!(f, "Failed to read CPU statistics: {err}"),
            Self::CalculateCpu(err) => write!(f, "Failed to calculate CPU usage: {err}"),
            Self::ReadMemory(err) => write!(f, "Failed to read memory statistics: {err}"),
        }
    }
}

impl Error for MonitorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadCpu(err) | Self::ReadMemory(err) => Some(err.as_ref()),
            Self::CalculateCpu(err) => Some(err),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemSnapshotPlaceholder;

pub fn read_memory() -> Result<MemoryStats, Box<dyn Error>> {
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

pub fn read_cpu_stats() -> Result<KernelStats, Box<dyn Error>> {
    Ok(KernelStats::current()?)
}

pub fn snapshot(
    previous_cpu: &[procfs::CpuTime],
) -> Result<(SystemSnapshot, Vec<procfs::CpuTime>), MonitorError> {
    let stats = read_cpu_stats().map_err(MonitorError::ReadCpu)?;
    let current_cpu = stats.cpu_time;

    let cpu_usage = cpu::usage(previous_cpu, &current_cpu)
    .map_err(MonitorError::CalculateCpu)?;

    let memory = read_memory().map_err(MonitorError::ReadMemory)?;

    Ok((
        SystemSnapshot {
            memory,
            cpu_usage,
        },
        current_cpu,
    ))
}
