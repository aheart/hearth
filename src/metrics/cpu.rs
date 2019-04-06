use super::{MetricPlugin, Metrics};
use std::str::FromStr;
use std::time::SystemTime;
use serde_derive::Serialize;

#[derive(Default, PartialEq, Debug, Clone, Serialize)]
pub struct CpuMetrics {
    cpu_usage: f64,
    iowait: f64,
}

pub struct CpuMetricPlugin {
    cpu: Cpu,
}

impl CpuMetricPlugin {
    pub fn new() -> Self {
        let processor = Cpu::default();
        Self { cpu: processor }
    }
}

impl MetricPlugin for CpuMetricPlugin {

    fn get_query(&self) -> &'static str {
        "grep 'cpu '  /proc/stat"
    }

    fn process_data(&mut self, raw_data: &str, _: &SystemTime) -> Metrics {
        let cpu_times = CpuTimes::from_string(&raw_data);
        self.cpu.push(cpu_times);

        Metrics::Cpu(CpuMetrics {
            cpu_usage: self.cpu.work_percent(),
            iowait: self.cpu.iowait_percent(),
        })
    }

    fn empty_metrics(&self) -> Metrics {
        Metrics::Cpu(CpuMetrics::default())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CpuTimes {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
    guest: u64,
    guest_nice: u64,
}

impl CpuTimes {
    pub fn new(
        user: u64,
        nice: u64,
        system: u64,
        idle: u64,
        iowait: u64,
        irq: u64,
        softirq: u64,
        steal: u64,
        guest: u64,
        guest_nice: u64,
    ) -> CpuTimes {
        CpuTimes {
            user,
            nice,
            system,
            idle,
            iowait,
            irq,
            softirq,
            steal,
            guest,
            guest_nice,
        }
    }

    pub fn from_string(raw_data: &str) -> CpuTimes {
        let (cpu_stats, _): (Vec<&str>, Vec<&str>) =
            raw_data.split_whitespace().partition(|s| !s.is_empty());

        macro_rules! parse_number {
            ($source:expr, $n:expr) => {
                $source
                    .get($n)
                    .and_then(|v| u64::from_str(v).ok())
                    .unwrap_or(0)
            };
        }

        Self::new(
            parse_number!(cpu_stats, 1),
            parse_number!(cpu_stats, 2),
            parse_number!(cpu_stats, 3),
            parse_number!(cpu_stats, 4),
            parse_number!(cpu_stats, 5),
            parse_number!(cpu_stats, 6),
            parse_number!(cpu_stats, 7),
            parse_number!(cpu_stats, 8),
            parse_number!(cpu_stats, 9),
            parse_number!(cpu_stats, 10),
        )
    }

    pub fn work(&self) -> u64 {
        self.user + self.nice + self.system + self.irq + self.softirq + self.steal
    }

    pub fn iowait(&self) -> u64 {
        self.iowait
    }

    pub fn total(&self) -> u64 {
        // guest is included in user, guest_nice is included in nice
        // which is why we do not add them to total
        self.work() + self.idle + self.iowait
    }

    pub fn diff(&self, other: &Self) -> Self {
        macro_rules! diff {
            ($this:expr, $that:expr) => {
                if $this > $that {
                    $this - $that
                } else {
                    $that - $this
                }
            };
        }
        Self::new(
            diff!(self.user, other.user),
            diff!(self.nice, other.nice),
            diff!(self.system, other.system),
            diff!(self.idle, other.idle),
            diff!(self.iowait, other.iowait),
            diff!(self.irq, other.irq),
            diff!(self.softirq, other.softirq),
            diff!(self.steal, other.steal),
            diff!(self.guest, other.guest),
            diff!(self.guest_nice, other.guest_nice),
        )
    }
}

#[derive(Default)]
pub struct Cpu {
    last_cpu_times: CpuTimes,
    work_percent: f64,
    iowait_percent: f64,
}

impl Cpu {
    pub fn work_percent(&self) -> f64 {
        self.work_percent
    }

    pub fn iowait_percent(&self) -> f64 {
        self.iowait_percent
    }

    pub fn push(&mut self, cpu_times: CpuTimes) {
        let diff = cpu_times.diff(&self.last_cpu_times);

        let total = diff.total() as f64;
        let work = diff.work() as f64;
        let iowait = diff.iowait() as f64;

        if total == 0.0 {
            self.work_percent = 0.0;
            self.iowait_percent = 0.0;
        } else {
            self.work_percent = work / total * 100.0;
            self.iowait_percent = iowait / total * 100.0;
        }

        self.last_cpu_times = cpu_times;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_process_data() {
        let raw_data_1 = "cpu  350732 1048 57727 6753933 12435 0 859 0 0 0";
        let raw_data_2 = "cpu  360767 1051 58366 6829700 12458 0 861 0 0 0";
        assert_parse(raw_data_1, raw_data_2, 12.350090783980386, 0.02659912801119476);
        assert_parse("", "", 0.0, 0.0);
    }

    fn assert_parse(raw_data_1: &str, raw_data_2: &str, cpu_usage: f64, iowait: f64) {
        let mut metric_plugin = CpuMetricPlugin::new();
        let now = SystemTime::now();
        metric_plugin.process_data(raw_data_1, &now);
        let metrics = metric_plugin.process_data(raw_data_2, &now);

        let expected_metrics = Metrics::Cpu(CpuMetrics {
            cpu_usage,
            iowait,
        });

        assert_eq!(metrics, expected_metrics);
    }
}

