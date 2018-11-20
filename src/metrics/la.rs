use super::MetricPlugin;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::SystemTime;

pub struct LoadAverageMetricPlugin {}

impl LoadAverageMetricPlugin {
    pub fn new() -> Self {
        LoadAverageMetricPlugin {}
    }
}

impl MetricPlugin for LoadAverageMetricPlugin {

    fn get_query(&self) -> &'static str {
        "cat /proc/loadavg"
    }

    fn process_data(&mut self, raw_data: &str, _: &SystemTime) -> HashMap<String, String> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_process_data() {
        assert_parse("3.17 2.23 1.68 3/942 17454", "3.17");
        assert_parse("0.07 0.07 0.09 1/996 25491", "0.07");
        assert_parse("53.99 24.51 14.20 51/9958 41299", "53.99");
        assert_parse("", "0");
    }

    fn assert_parse(raw_data: &str, load_average: &str) {
        let mut metric_plugin = LoadAverageMetricPlugin::new();
        let now = SystemTime::now();
        let metrics = metric_plugin.process_data(raw_data, &now);

        let mut expected_metrics = HashMap::new();
        expected_metrics.insert("load_average".to_string(), load_average.to_string());

        assert_eq!(metrics, expected_metrics);
    }
}
