use super::MetricPlugin;
use std::collections::HashMap;
use std::str::FromStr;

pub struct LoadAverageMetricPlugin {}

impl MetricPlugin for LoadAverageMetricPlugin {
    fn new() -> Self {
        LoadAverageMetricPlugin {}
    }

    fn get_query(&self) -> &'static str {
        "cat /proc/loadavg"
    }

    fn process_data(&mut self, raw_data: &str) -> HashMap<String, String> {
        let (parts, _): (Vec<&str>, Vec<&str>) = raw_data.split(' ').partition(|s| !s.is_empty());
        let load_average_1m = parts.get(0).unwrap_or(&"0"); // and_then?
        let load_average_1m = f64::from_str(load_average_1m).unwrap_or(0.);

        let mut metrics = HashMap::new();
        metrics.insert("load_average".into(), load_average_1m.to_string());
        metrics
    }

    fn empty_metrics(&self) -> HashMap<String, String> {
        let mut metrics = HashMap::new();
        metrics.insert("load_average".into(), "0".into());
        metrics
    }
}
