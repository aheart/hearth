mod cpu;
mod disk;
pub mod aggreagtor;
mod la;
mod ram;

use std::collections::HashMap;
use std::time::SystemTime;

/// Interface for Metric Plugins that possess the knowledge of retrieving raw metric data and
/// processing this raw data into structured Metric key value pairs.
pub trait MetricPlugin: Send + 'static {

    /// Metric Plugin Constructor
    fn new() -> Self
        where
            Self: Sized;

    /// Returns a command that should be run in order to retrieve raw data
    fn get_query(&self) -> &'static str;

    /// Transforms raw data into a HashMap of metrics
    fn process_data(&mut self, raw_data: &str, timestamp: &SystemTime) -> HashMap<String, String>;

    /// Returns a HashMap with keys and empty values
    fn empty_metrics(&self) -> HashMap<String, String>;
}

/// Creates all possible metric plugins and returns them as a HashMap
fn metric_plugin_factory() -> Vec<Box<dyn MetricPlugin>> {
    let metric_plugins: Vec<Box<dyn MetricPlugin>> = vec![
        Box::new(cpu::CpuMetricPlugin::new()),
        Box::new(ram::RamMetricPlugin::new()),
        Box::new(la::LoadAverageMetricPlugin::new()),
        Box::new(disk::DiskMetricPlugin::new()),
    ];

    metric_plugins
}
