mod cpu;
mod disk;
pub mod aggreagtor;
mod la;
mod ram;

use super::ssh::SshClient;
use std::collections::HashMap;
use std::time::SystemTime;

pub trait MetricPlugin: Send + 'static {
    fn new() -> Self
    where
        Self: Sized;

    fn get_query(&self) -> &'static str;

    fn process_data(&mut self, raw_data: &str, timestamp: &SystemTime) -> HashMap<String, String>;

    fn provide(&mut self, client: &mut SshClient, timestamp: &SystemTime) -> HashMap<String, String> {
        match client.run(self.get_query()) {
            Ok(raw_data) => self.process_data(&raw_data, timestamp),
            Err(_e) => HashMap::new(),
        }
    }

    fn empty_metrics(&self) -> HashMap<String, String>;
}

fn metric_plugin_factory() -> Vec<Box<dyn MetricPlugin>> {
    let metric_plugins: Vec<Box<dyn MetricPlugin>> = vec![
        Box::new(cpu::CpuMetricPlugin::new()),
        Box::new(ram::RamMetricPlugin::new()),
        Box::new(la::LoadAverageMetricPlugin::new()),
        Box::new(disk::DiskMetricPlugin::new()),
    ];

    metric_plugins
}
