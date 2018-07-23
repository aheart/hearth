use super::MetricPlugin;
use actix::prelude::*;
use ssh::SshClient;
use std::collections::HashMap;
use std::time::Duration;
use ws::server::{Message, WsServer};
use std::time::SystemTime;

pub fn metric_aggregator_factory(
    ws_server: Addr<WsServer>,
    username: String,
    hostname: String,
    index: usize,
) -> MetricAggregator {
    let ssh = SshClient::new(username, hostname, 22);
    let aggregator = MetricProvider::new(ssh, super::metric_plugin_factory());

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
            metrics.insert("index".into(), aggregator.index.to_string());
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

    fn get_metrics(&mut self) -> HashMap<String, String> {
        let server = self.ssh.get_hostname().to_string();
        let cpus = self.ssh.get_cpus().to_string();
        let uptime_seconds = self.ssh.get_uptime().to_string();
        let mut accum = HashMap::new();
        accum.insert("server".into(), server.to_string());
        accum.insert("cpus".into(), cpus.to_string());
        accum.insert("uptime_seconds".into(), uptime_seconds.to_string());
        accum.extend(self.batch_fetch());
        accum
    }

    fn batch_fetch(&mut self) -> HashMap<String, String> {
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

    fn process_raw_data(&mut self, raw_data: &str) -> HashMap<String, String> {
        let (results, _): (Vec<&str>, Vec<&str>) =
            raw_data.split("######").partition(|s| !s.is_empty());
        let now = SystemTime::now();
        let mut metrics = HashMap::new();
        self.metric_providers
            .iter_mut()
            .zip(results.iter())
            .for_each(|(provider, &data)| {
                metrics.extend(provider.process_data(data, &now));
            });
        metrics
    }

    fn build_empty_metrics(&mut self) -> HashMap<String, String> {
        let mut metrics = HashMap::new();
        self.metric_providers.iter_mut().for_each(|provider| {
            metrics.extend(provider.empty_metrics());
        });
        metrics
    }
}
