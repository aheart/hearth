mod cpu;
mod disk;
pub mod aggreagtor;
mod la;
mod ram;
mod network;

use std::collections::HashMap;
use std::time::SystemTime;

pub type Metrics = HashMap<String, String>;

/// Interface for Metric Plugins that possess the knowledge of retrieving raw metric data and
/// processing this raw data into structured Metric key value pairs.
pub trait MetricPlugin: Send + 'static {
    /// Returns a command that should be run in order to retrieve raw data
    fn get_query(&self) -> &str;

    /// Transforms raw data into a HashMap of metrics
    fn process_data(&mut self, raw_data: &str, timestamp: &SystemTime) -> Metrics;

    /// Returns a HashMap with keys and empty values
    fn empty_metrics(&self) -> Metrics;
}

/// Creates all possible metric plugins and returns them as a HashMap
fn metric_plugin_factory(disk: &str, network_interface: &str) -> Vec<Box<dyn MetricPlugin>> {
    let metric_plugins: Vec<Box<dyn MetricPlugin>> = vec![
        Box::new(cpu::CpuMetricPlugin::new()),
        Box::new(ram::RamMetricPlugin::new()),
        Box::new(la::LoadAverageMetricPlugin::new()),
        Box::new(disk::DiskMetricPlugin::new(disk)),
        Box::new(network::NetworkMetricPlugin::new(network_interface)),
    ];

    metric_plugins
}
