use super::hub::MetricHub;
use crate::config::{AuthMethod, ServerConfig};
use crate::metrics::{
    cpu::CpuMetrics, disk::DiskMetrics, la::LaMetrics, network::NetMetrics, ram::RamMetrics,
    space::SpaceMetrics, MetricPlugin, Metrics,
};
use crate::ssh::SshClient;
use actix::prelude::*;
use log::{error, info};
use serde_derive::Serialize;
use std::ops::Add;
use std::time::{Duration, SystemTime};

#[derive(Default, Clone, Serialize, Message)]
#[rtype(result = "()")]
pub struct Node {
    #[serde(flatten)]
    specs: NodeSpecs,
    #[serde(flatten)]
    metrics: NodeMetrics,
}

impl Node {
    pub fn new(specs: NodeSpecs, metrics: NodeMetrics) -> Self {
        Node { specs, metrics }
    }
}

/// Node Specifications are data that changes rarely or doesn't change at all
#[derive(Default, Clone, Serialize, Message)]
#[rtype(result = "()")]
pub struct NodeSpecs {
    index: u8,
    hostname: String,
    cpus: u16,
    ip: String,
}

impl NodeSpecs {
    pub fn new(index: u8, hostname: String, cpus: u16, ip: String) -> Self {
        NodeSpecs {
            index,
            hostname,
            cpus,
            ip,
        }
    }

    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    pub fn get_cpus(&self) -> u16 {
        self.cpus
    }

    pub fn update_cpus(&mut self, cpus: u16) {
        self.cpus = cpus;
    }
}

/// Node Metrics are time-series data that changes often
/// (apart from the hostname that is used for identification at the moment)
#[derive(Default, Clone, Serialize, Message)]
#[rtype(result = "()")]
pub struct NodeMetrics {
    #[serde(skip)]
    hostname: String,
    online: bool,
    uptime_seconds: u64,

    cpu: CpuMetrics,
    disk: DiskMetrics,
    la: LaMetrics,
    net: NetMetrics,
    ram: RamMetrics,
    space: SpaceMetrics,
}

impl Add for NodeMetrics {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            hostname: "".to_string(),
            online: false,
            uptime_seconds: self.uptime_seconds + other.uptime_seconds,

            cpu: self.cpu + other.cpu,
            disk: self.disk + other.disk,
            la: self.la + other.la,
            net: self.net + other.net,
            ram: self.ram + other.ram,
            space: self.space + other.space,
        }
    }
}

impl NodeMetrics {
    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    pub fn aggregate(nodes: Vec<Self>) -> Self {
        let mut cluster = NodeMetrics::default();
        let node_count = nodes.len();

        for node in nodes {
            cluster = cluster + node;
        }

        if node_count > 0 {
            cluster.cpu = cluster.cpu.divide(node_count as f32);
        }

        cluster.hostname = "Cluster".to_string();
        cluster
    }

    pub fn aggregate_avg(measurements: Vec<Self>) -> Self {
        let mut average = NodeMetrics::default();
        let measurement_count = measurements.len();
        if let Some(last_measurement) = measurements.last().cloned() {
            average.hostname = last_measurement.hostname().to_string();
            average.uptime_seconds = last_measurement.uptime_seconds;
            average.online = last_measurement.online;
        }

        for measurement in measurements {
            average = average + measurement;
        }

        if measurement_count > 0 {
            average.cpu = average.cpu.divide(measurement_count as f32);
            average.disk = average.disk.divide(measurement_count as f64);
            average.la = average.la.divide(measurement_count as f64);
            average.net = average.net.divide(measurement_count as f64);
            average.ram = average.ram.divide(measurement_count as u64);
            average.space = average.space.divide(measurement_count as u64);
        }

        average
    }

    pub fn set(&mut self, metrics: Metrics) {
        use Metrics::*;
        match metrics {
            Cpu(m) => self.cpu = m,
            Disk(m) => self.disk = m,
            La(m) => self.la = m,
            Net(m) => self.net = m,
            Ram(m) => self.ram = m,
            Space(m) => self.space = m,
        }
    }
}

pub fn metric_aggregator_factory(
    hub: Addr<MetricHub>,
    server_config: &ServerConfig,
    auth_method: AuthMethod,
    index: u8,
) -> MetricAggregator {
    let ssh = SshClient::new(
        server_config.username.clone(),
        auth_method,
        server_config.hostname.clone(),
        22,
    );
    let plugins = super::metric_plugin_factory(
        &server_config.disk,
        &server_config.filesystem,
        &server_config.network_interface,
    );
    let aggregator = MetricProvider::new(ssh, plugins);

    MetricAggregator::new(hub, aggregator, index)
}

/// Metric Aggregator
///
/// Every second it fetches Metrics from a Metric Provider associated with one particular server
pub struct MetricAggregator {
    hub: Addr<MetricHub>,
    provider: MetricProvider,
    pub index: u8,
}

impl MetricAggregator {
    pub fn new(ws_server: Addr<MetricHub>, provider: MetricProvider, index: u8) -> Self {
        Self {
            hub: ws_server,
            provider,
            index,
        }
    }

    fn aggregate(&mut self, _ctx: &mut actix::Context<Self>) -> NodeMetrics {
        self.provider.get_metrics()
    }

    fn send_metrics(&mut self, metrics: NodeMetrics, _ctx: &mut actix::Context<Self>) {
        self.hub.do_send(metrics);
    }

    fn send_specs(&mut self, ctx: &mut actix::Context<Self>) {
        // TODO: implement proper initialization
        let ping = self.provider.ssh.run("|");
        let specs = NodeSpecs::new(
            self.index,
            self.provider.ssh.get_hostname().to_string(),
            self.provider.ssh.get_cpus() as u16,
            self.provider.ssh.get_ip().to_string(),
        );
        self.hub.do_send(specs);

        if ping.is_err() {
            let delay = Duration::new(1, 0);
            ctx.run_later(delay, move |aggregator, ctx| {
                aggregator.send_specs(ctx);
            });
        }
    }

    fn update_uptime(&self, ctx: &mut actix::Context<Self>) {
        let delay = Duration::new(60, 0);

        ctx.run_later(delay, move |aggregator, ctx| {
            aggregator.provider.ssh.update_uptime();
            aggregator.update_uptime(ctx);
        });
    }
}

impl Actor for MetricAggregator {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("[{}] Aggregator started", self.provider.ssh.get_hostname());
        self.send_specs(ctx);
        let interval = Duration::new(0, 1_000_000_000);

        ctx.run_interval(interval, move |aggregator, ctx| {
            let metrics = aggregator.aggregate(ctx);
            aggregator.send_metrics(metrics, ctx);
        });

        self.update_uptime(ctx);
    }
}

/// Metric Provider
///
/// Retrieves data from a server using the available Metric Plugins
pub struct MetricProvider {
    ssh: SshClient,
    metric_plugins: Vec<Box<dyn MetricPlugin>>,
}

impl MetricProvider {
    pub fn new(ssh: SshClient, metric_providers: Vec<Box<dyn MetricPlugin>>) -> Self {
        Self {
            ssh,
            metric_plugins: metric_providers,
        }
    }

    fn get_metrics(&mut self) -> NodeMetrics {
        let mut aggregate = self.batch_fetch();
        aggregate.hostname = self.ssh.get_hostname().to_string();
        aggregate.uptime_seconds = self.ssh.get_uptime();
        aggregate
    }

    fn batch_fetch(&mut self) -> NodeMetrics {
        let merged_command = self
            .metric_plugins
            .iter()
            .fold("".to_string(), |accum, provider| {
                if accum == "" {
                    return provider.get_query().to_string();
                }
                format!("{} && printf '######' && {}", accum, provider.get_query())
            });

        match self.ssh.run(&merged_command) {
            Ok(raw_data) => self.process_raw_data(&raw_data),
            Err(e) => {
                error!("[{}]: SSH FAILED: {:?}", self.ssh.get_hostname(), e);
                self.build_empty_metrics()
            }
        }
    }

    fn process_raw_data(&mut self, raw_data: &str) -> NodeMetrics {
        let (results, _): (Vec<&str>, Vec<&str>) =
            raw_data.split("######").partition(|s| !s.is_empty());
        let now = SystemTime::now();
        let mut aggregate = NodeMetrics::default();
        aggregate.online = true;

        self.metric_plugins
            .iter_mut()
            .zip(results.iter())
            .for_each(|(provider, &data)| {
                aggregate.set(provider.process_data(data, &now));
            });

        aggregate
    }

    fn build_empty_metrics(&mut self) -> NodeMetrics {
        let mut metrics = NodeMetrics::default();
        metrics.hostname = self.ssh.get_hostname().to_string();
        metrics
    }
}
