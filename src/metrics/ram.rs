use super::MetricPlugin;
use std::collections::HashMap;
use std::str::FromStr;

pub struct RamMetricPlugin {}

impl MetricPlugin for RamMetricPlugin {
    fn new() -> Self {
        Self {}
    }

    fn get_query(&self) -> &'static str {
        "cat /proc/meminfo"
    }

    fn process_data(&mut self, raw_data: &str) -> HashMap<String, String> {
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
