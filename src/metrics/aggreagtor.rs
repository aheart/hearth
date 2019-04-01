use actix::prelude::*;
use crate::ssh::SshClient;
use crate::ws::server::{Message, WsServer};
use crate::config::ServerConfig;
use crate::metrics::{
    MetricPlugin,
    Metrics,
    cpu::CpuMetrics,
    disk::DiskMetrics,
    la::LaMetrics,
    network::NetMetrics,
    ram::RamMetrics
};
use log::{info, error};
use std::time::{Duration, SystemTime};
use serde_derive::Serialize;



#[derive(Default, Clone, Serialize)]
pub struct MetricAggregate {
    index: String,
    pub server: String,
    cpus: u8,
    uptime_seconds: u64,

    cpu: CpuMetrics,
    disk: DiskMetrics,
    la: LaMetrics,
    net: NetMetrics,
    ram: RamMetrics,
}

impl MetricAggregate {
    pub fn add(&mut self, metrics: Metrics) {
        use Metrics::*;
        match metrics {
            Cpu(m) => self.cpu = m,
            Disk(m) => self.disk = m,
            La(m) => self.la = m,
            Net(m) => self.net = m,
            Ram(m) => self.ram = m,
        }
    }
}

pub fn metric_aggregator_factory(
    ws_server: Addr<WsServer>,
    server_config: &ServerConfig,
    index: usize,
) -> MetricAggregator {
    let ssh = SshClient::new(
        server_config.username.clone(),
        server_config.hostname.clone(),
        22
    );
    let plugins = super::metric_plugin_factory(
        &server_config.disk,
        &server_config.network_interface
    );
    let aggregator = MetricProvider::new(ssh, plugins);

    MetricAggregator::new(ws_server, aggregator, index)
}

pub struct MetricAggregator {
    ws_server: Addr<WsServer>,
    provider: MetricProvider,
    index: usize,
}

impl MetricAggregator {
    pub fn new(ws_server: Addr<WsServer>, provider: MetricProvider, index: usize) -> MetricAggregator {
        MetricAggregator {
            ws_server,
            provider,
            index,
        }
    }

    fn send_metrics(&self, ctx: &mut actix::Context<Self>) {
        let delay = Duration::new(0, 1_000_000_000);

        ctx.run_later(delay, move |aggregator, ctx| {
            let mut metrics = aggregator.provider.get_metrics();
            metrics.index = aggregator.index.to_string();
            let ws_message = Message { id: 0, metrics };
            aggregator.ws_server.do_send(ws_message);
            aggregator.send_metrics(ctx);
        });
    }

    fn update_uptime(&self, ctx: &mut actix::Context<Self>) {
        let delay = Duration::new(60, 0);

        ctx.run_later(delay, move |aggreagator, ctx| {
            aggreagator.provider.ssh.update_uptime();
            aggreagator.update_uptime(ctx);
        });
    }
}

impl Actor for MetricAggregator {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("[{}] Aggregator started", self.provider.ssh.get_hostname());
        self.send_metrics(ctx);
        self.update_uptime(ctx);
    }
}

pub struct MetricProvider {
    ssh: SshClient,
    metric_providers: Vec<Box<dyn MetricPlugin>>,
}

impl MetricProvider {
    pub fn new(ssh: SshClient, metric_providers: Vec<Box<dyn MetricPlugin>>) -> MetricProvider {
        MetricProvider {
            ssh,
            metric_providers,
        }
    }

    fn get_metrics(&mut self) -> MetricAggregate {
        let mut aggregate = self.batch_fetch();
        aggregate.server = self.ssh.get_hostname().to_string();
        aggregate.cpus = self.ssh.get_cpus();
        aggregate.uptime_seconds = self.ssh.get_uptime();
        aggregate
    }

    fn batch_fetch(&mut self) -> MetricAggregate {
        let merged_command = self.metric_providers.iter().fold(
            "".to_string(),
            |accum, provider| {
                if accum == "" {
                    return provider.get_query().to_string();
                }
                format!("{} && printf '######' && {}", accum, provider.get_query())
            },
        );

        match self.ssh.run(&merged_command) {
            Ok(raw_data) => self.process_raw_data(&raw_data),
            Err(e) => {
                error!("{}: SSH FAILED: {:?}", self.ssh.get_hostname(), e);
                self.build_empty_metrics()
            }
        }
    }

    fn process_raw_data(&mut self, raw_data: &str) -> MetricAggregate {
        let (results, _): (Vec<&str>, Vec<&str>) =
            raw_data.split("######").partition(|s| !s.is_empty());
        let now = SystemTime::now();
        let mut aggregate = MetricAggregate::default();

        self.metric_providers
            .iter_mut()
            .zip(results.iter())
            .for_each(|(provider, &data)| {
                aggregate.add(provider.process_data(data, &now));
            });

        aggregate
    }

    fn build_empty_metrics(&mut self) -> MetricAggregate {
        MetricAggregate::default()
    }
}
