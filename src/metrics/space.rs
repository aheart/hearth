use super::{MetricPlugin, Metrics};
use derive_more::Add;
use serde_derive::Serialize;
use std::str::FromStr;
use std::time::SystemTime;

#[derive(Default, PartialEq, Debug, Clone, Serialize, Add)]
pub struct SpaceMetrics {
    total: u64,
    used: u64,
}

impl SpaceMetrics {
    pub fn divide(self, divisor: u64) -> Self {
        Self {
            total: self.total / divisor,
            used: self.used / divisor,
        }
    }
}

pub struct SpaceMetricPlugin {
    command: String,
}

impl SpaceMetricPlugin {
    pub fn new(filesystem: &str) -> Self {
        // After researching how to fetch disk space metrics without relying on df
        // I've decided that it's not worth it for now.
        let command = format!("df /dev/{}", filesystem);
        Self { command }
    }
}

impl MetricPlugin for SpaceMetricPlugin {
    fn get_query(&self) -> &str {
        &self.command
    }

    fn process_data(&mut self, raw_data: &str, _timestamp: &SystemTime) -> Metrics {
        let metrics = raw_data
            .lines()
            .last()
            .and_then(|line| {
                let mut iter = line.split_whitespace();
                let total = iter.nth(1).and_then(|v| u64::from_str(v).ok()).unwrap_or(0);
                let free = iter.nth(1).and_then(|v| u64::from_str(v).ok()).unwrap_or(0);
                let used = total - free;
                Some(SpaceMetrics { total, used })
            })
            .unwrap_or_default();

        Metrics::Space(metrics)
    }

    fn empty_metrics(&self) -> Metrics {
        Metrics::Space(SpaceMetrics::default())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_process_data() {
        let raw_data = "Filesystem     1K-blocks      Used Available Use% Mounted on
/dev/sda1      475788360 389354068  62242600  87% /";
        let total = 475788360;
        let used = 475788360 - 62242600;
        assert_parse(raw_data, total, used);
        assert_parse("", 0, 0);
    }

    fn assert_parse(raw_data: &str, total: u64, used: u64) {
        let mut metric_plugin = SpaceMetricPlugin::new("sda1");
        let metrics = metric_plugin.process_data(raw_data, &std::time::UNIX_EPOCH);

        let expected_metrics = Metrics::Space(SpaceMetrics { total, used });

        assert_eq!(metrics, expected_metrics);
    }
}
