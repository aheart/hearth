use super::MetricPlugin;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::SystemTime;

pub struct RamMetricPlugin {}

impl MetricPlugin for RamMetricPlugin {
    fn new() -> Self {
        Self {}
    }

    fn get_query(&self) -> &'static str {
        "cat /proc/meminfo"
    }

    fn process_data(&mut self, raw_data: &str, _: &SystemTime) -> HashMap<String, String> {
        let mut mem_total = 0;
        let mut mem_available = 0;

        raw_data.split(|c| c == '\n' || c == ':' || c == ' ')
            .filter(|c| *c != "" && *c != "kB")
            .collect::<Vec<&str>>()
            .chunks(2)
            .for_each(|metric|{
                match metric[0] {
                    "MemTotal" => mem_total = u64::from_str(metric[1]).unwrap_or(0) * 1024,
                    "MemAvailable" => mem_available = u64::from_str(metric[1]).unwrap_or(0) * 1024,
                    _ => (),
                };
            });

        let mut metrics = HashMap::new();
        let mem_used = mem_total - mem_available;
        metrics.insert("mem_total".into(), mem_total.to_string());
        metrics.insert("mem_used".into(), mem_used.to_string());
        metrics
    }

    fn empty_metrics(&self) -> HashMap<String, String> {
        let mut metrics = HashMap::new();
        metrics.insert("mem_total".into(), "0.0".into());
        metrics.insert("mem_used".into(), "0.0".into());
        metrics
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
        let mem_available: u64 = 11132000 * 1024;
        let mem_used: u64 = mem_total - mem_available;
        assert_parse(raw_data, &mem_total.to_string(), &mem_used.to_string());
        assert_parse("", "0", "0");
    }

    fn assert_parse(raw_data: &str, mem_total: &str, mem_used: &str) {
        let mut metric_plugin = RamMetricPlugin::new();
        let now = SystemTime::now();
        let metrics = metric_plugin.process_data(raw_data, &now);

        let mut expected_metrics = HashMap::new();
        expected_metrics.insert("mem_total".to_string(), mem_total.to_string());
        expected_metrics.insert("mem_used".to_string(), mem_used.to_string());

        assert_eq!(metrics, expected_metrics);
    }
}
