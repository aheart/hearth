use super::{MetricPlugin, Metrics};
use derive_more::Add;
use serde_derive::Serialize;
use std::str::FromStr;
use std::time::SystemTime;

#[derive(Default, PartialEq, Debug, Clone, Serialize, Add)]
pub struct RamMetrics {
    mem_total: u64,
    mem_used: u64,
    mem_buffers: u64,
    mem_cached: u64,
}

impl RamMetrics {
    pub fn divide(self, divisor: u64) -> Self {
        Self {
            mem_total: self.mem_total / divisor,
            mem_used: self.mem_used / divisor,
            mem_buffers: self.mem_buffers / divisor,
            mem_cached: self.mem_cached / divisor,
        }
    }
}

pub struct RamMetricPlugin {}

impl RamMetricPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl MetricPlugin for RamMetricPlugin {
    fn get_query(&self) -> &'static str {
        "cat /proc/meminfo"
    }

    fn process_data(&mut self, raw_data: &str, _: &SystemTime) -> Metrics {
        let mut mem_total = 0;
        let mut mem_free = 0;
        let mut mem_buffers = 0;
        let mut mem_cached = 0;
        let mut mem_shmem = 0;
        let mut mem_sreclaimable = 0;

        raw_data
            .split(|c| c == '\n' || c == ':' || c == ' ')
            .filter(|c| *c != "" && *c != "kB")
            .collect::<Vec<&str>>()
            .chunks(2)
            .for_each(|metric| {
                match metric[0] {
                    "MemTotal" => mem_total = u64::from_str(metric[1]).unwrap_or(0) * 1024,
                    "MemFree" => mem_free = u64::from_str(metric[1]).unwrap_or(0) * 1024,
                    "Buffers" => mem_buffers = u64::from_str(metric[1]).unwrap_or(0) * 1024,
                    "Cached" => mem_cached = u64::from_str(metric[1]).unwrap_or(0) * 1024,
                    "Shmem" => mem_shmem = u64::from_str(metric[1]).unwrap_or(0) * 1024,
                    "SReclaimable" => {
                        mem_sreclaimable = u64::from_str(metric[1]).unwrap_or(0) * 1024
                    }
                    _ => (),
                };
            });
        let mem_cached = mem_cached + mem_sreclaimable - mem_shmem;
        let mem_used = mem_total - mem_free - mem_cached - mem_buffers;

        Metrics::Ram(RamMetrics {
            mem_total,
            mem_used,
            mem_buffers,
            mem_cached,
        })
    }

    fn empty_metrics(&self) -> Metrics {
        Metrics::Ram(RamMetrics::default())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_process_data() {
        let raw_data = r#"
MemTotal:       16256332 kB
MemFree:         6890464 kB
MemAvailable:   11132000 kB
Buffers:          536332 kB
Cached:          3729760 kB
SwapCached:            0 kB
Active:          5845680 kB
Inactive:        2676952 kB
Active(anon):    4258740 kB
Inactive(anon):    71668 kB
Active(file):    1586940 kB
Inactive(file):  2605284 kB
Unevictable:         668 kB
Mlocked:             668 kB
SwapTotal:      16598524 kB
SwapFree:       16598524 kB
Dirty:              2900 kB
Writeback:             0 kB
AnonPages:       4241168 kB
Mapped:           802548 kB
Shmem:             73864 kB
Slab:             560172 kB
SReclaimable:     391760 kB
SUnreclaim:       168412 kB
KernelStack:       17232 kB
PageTables:        67836 kB
NFS_Unstable:          0 kB
Bounce:                0 kB
WritebackTmp:          0 kB
CommitLimit:    24726688 kB
Committed_AS:   12589700 kB
VmallocTotal:   34359738367 kB
VmallocUsed:           0 kB
VmallocChunk:          0 kB
HardwareCorrupted:     0 kB
AnonHugePages:   2174976 kB
CmaTotal:              0 kB
CmaFree:               0 kB
HugePages_Total:       0
HugePages_Free:        0
HugePages_Rsvd:        0
HugePages_Surp:        0
Hugepagesize:       2048 kB
DirectMap4k:      401680 kB
DirectMap2M:     9908224 kB
DirectMap1G:     7340032 kB
        "#;
        let mem_total: u64 = 16256332 * 1024;
        let mem_free: u64 = 6890464 * 1024;
        let mem_buffers: u64 = 536332 * 1024;
        let mem_cached: u64 = (3729760 + 391760 - 73864) * 1024;
        let mem_used: u64 = mem_total - mem_free - mem_buffers - mem_cached;
        assert_parse(raw_data, mem_total, mem_used, mem_buffers, mem_cached);
        assert_parse("", 0, 0, 0, 0);
    }

    fn assert_parse(
        raw_data: &str,
        mem_total: u64,
        mem_used: u64,
        mem_buffers: u64,
        mem_cached: u64,
    ) {
        let mut metric_plugin = RamMetricPlugin::new();
        let now = SystemTime::now();
        let metrics = metric_plugin.process_data(raw_data, &now);

        let expected_metrics = Metrics::Ram(RamMetrics {
            mem_total,
            mem_used,
            mem_buffers,
            mem_cached,
        });

        assert_eq!(metrics, expected_metrics);
    }
}
