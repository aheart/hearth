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
        for line in raw_data.split('\n') {
            let field = match line.split(':').next() {
                Some("MemTotal") => &mut mem_total,
                Some("MemAvailable") => &mut mem_available,
                _ => continue,
            };
            if let Some(val_str) = line.rsplit(' ').nth(1) {
                if let Ok(value) = u64::from_str(val_str) {
                    *field = value * 1024;
                }
            }
        }

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
