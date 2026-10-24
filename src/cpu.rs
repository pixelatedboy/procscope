use procfs::CpuTime;
use std::fmt;

#[derive(Debug)]
pub enum CpuUsageError {
    CpuCountMismatch {
        previous: usize,
        current: usize,
    },
    CounterOverflow {
        cpu: usize,
    },
    CounterDecreased {
        cpu: usize,
    },
    InvalidIdleTime {
        cpu: usize,
    },
}

impl fmt::Display for CpuUsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CpuCountMismatch { previous, current } => {
                write!(
                    f,
                    "CPU count changed between samples: {previous} -> {current}"
                )
            }
            Self::CounterOverflow { cpu } => {
                write!(f, "CPU {cpu}: counter total overflowed")
            }
            Self::CounterDecreased { cpu } => {
                write!(f, "CPU {cpu}: counters decreased between samples")
            }
            Self::InvalidIdleTime { cpu } => {
                write!(f, "CPU {cpu}: idle time exceeds total elapsed time")
            }
        }
    }
}

impl std::error::Error for CpuUsageError {}

fn total_time(cpu: &CpuTime) -> Option<u64> {
    [
        cpu.user,
        cpu.nice,
        cpu.system,
        cpu.idle,
        cpu.iowait.unwrap_or(0),
        cpu.irq.unwrap_or(0),
        cpu.softirq.unwrap_or(0),
        cpu.steal.unwrap_or(0),
    ]
    .into_iter()
    .try_fold(0_u64, u64::checked_add)
}

fn idle_time(cpu: &CpuTime) -> Option<u64> {
    cpu.idle.checked_add(cpu.iowait.unwrap_or(0))
}

pub fn usage(
    previous: &[CpuTime],
    current: &[CpuTime],
) -> Result<Vec<f64>, CpuUsageError> {
    if previous.len() != current.len() {
        return Err(CpuUsageError::CpuCountMismatch {
            previous: previous.len(),
                   current: current.len(),
        });
    }

    previous
    .iter()
    .zip(current.iter())
    .enumerate()
    .map(|(index, (prev, curr))| {
        let prev_total = total_time(prev)
        .ok_or(CpuUsageError::CounterOverflow { cpu: index })?;
        let curr_total = total_time(curr)
        .ok_or(CpuUsageError::CounterOverflow { cpu: index })?;

        let prev_idle = idle_time(prev)
        .ok_or(CpuUsageError::CounterOverflow { cpu: index })?;
        let curr_idle = idle_time(curr)
        .ok_or(CpuUsageError::CounterOverflow { cpu: index })?;

        let delta_total = curr_total
        .checked_sub(prev_total)
        .ok_or(CpuUsageError::CounterDecreased { cpu: index })?;

        let delta_idle = curr_idle
        .checked_sub(prev_idle)
        .ok_or(CpuUsageError::CounterDecreased { cpu: index })?;

        if delta_total == 0 {
            return Ok(0.0);
        }

        if delta_idle > delta_total {
            return Err(CpuUsageError::InvalidIdleTime { cpu: index });
        }

        let busy = delta_total - delta_idle;
        Ok(busy as f64 / delta_total as f64 * 100.0)
    })
    .collect()
}
