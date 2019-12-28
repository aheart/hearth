use super::metric_buffer::{MetricBuffer, MetricBufferMap};
use crate::metrics::aggregator::{Node, NodeMetrics, NodeSpecs};
use crate::ws::server::MessageData::*;
use crate::ws::server::{ClientJoined, OutboundMessage, Receiver, View, WsServer};
use actix::prelude::*;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::time::Duration;

/// Metric Hub
///
/// It receives metrics from the MetricAggregator, stores them and provides to clients
pub struct MetricHub {
    ws_server: Option<Addr<WsServer>>,
    node_buffers: MetricBufferMap,
    cluster_buffer: MetricBuffer,
    node_specs: HashMap<String, NodeSpecs>,
    cluster_specs: NodeSpecs,
    latest_metrics: HashMap<String, NodeMetrics>,
}

impl Default for MetricHub {
    fn default() -> Self {
        Self {
            ws_server: None,
            node_buffers: MetricBufferMap::new(120),
            cluster_buffer: MetricBuffer::new(120),
            node_specs: HashMap::new(),
            cluster_specs: NodeSpecs::new(0, "Cluster".to_string(), 0, "".to_string()),
            latest_metrics: HashMap::new(),
        }
    }
}

impl MetricHub {
    fn send_to_server(&self, msg: OutboundMessage) {
        if let Some(ws_server) = &self.ws_server {
            ws_server.do_send(msg);
        }
    }

    fn aggregate_cluster_metrics(&self, ctx: &mut actix::Context<Self>) {
        let delay = Duration::new(1, 0);

        ctx.run_later(delay, move |hub, ctx| {
            let latest_metrics = hub.latest_metrics.borrow_mut();
            let buffers = hub.node_buffers.borrow_mut();
            let mut total_cpus: u16 = 0;
            for (hostname, specs) in hub.node_specs.iter() {
                total_cpus += specs.get_cpus();
                if let Some(metrics) = latest_metrics.remove(hostname) {
                    buffers.push(hostname, metrics);
                } else {
                    buffers.push(hostname, NodeMetrics::default());
                }
            }
            hub.cluster_specs.update_cpus(total_cpus);

            let latest_node_metrics: Vec<NodeMetrics> = hub
                .node_buffers
                .storage()
                .values()
                .map(|buffer| {
                    buffer
                        .storage(View::OverviewOneSecond)
                        .last()
                        .cloned()
                        .expect("Can't even aggregate these days")
                })
                .collect();

            let cluster = NodeMetrics::aggregate(latest_node_metrics);
            hub.cluster_buffer.push(cluster);

            hub.aggregate_cluster_metrics(ctx);
        });
    }

    fn send_metrics(&self, _ctx: &mut actix::Context<Self>, timeframe: View) {
        for (hostname, specs) in self.node_specs.iter() {
            if let Some(buffer) = self.node_buffers.storage().get(hostname) {
                if !buffer.storage(timeframe).is_empty() {
                    let metrics = buffer.storage(timeframe).last().unwrap().clone();
                    let node = Node::new(specs.clone(), metrics);
                    self.send_to_server(OutboundMessage {
                        receiver: Receiver::SubscribersOf(timeframe),
                        data: NodeMetrics(vec![node]),
                    });
                }
            }
        }

        if !self.cluster_buffer.storage(timeframe).is_empty() {
            if let Some(metrics) = self.cluster_buffer.storage(timeframe).last() {
                let specs = self.cluster_specs.clone();
                let node = Node::new(specs, metrics.clone());
                self.send_to_server(OutboundMessage {
                    receiver: Receiver::SubscribersOf(timeframe),
                    data: ClusterMetrics(vec![node]),
                });
            }
        }
    }

    fn send_1s_metrics(&self, ctx: &mut actix::Context<Self>) {
        let delay = Duration::new(1, 0);

        ctx.run_later(delay, move |hub, ctx| {
            hub.send_metrics(ctx, View::OverviewOneSecond);
            hub.send_1s_metrics(ctx);
        });
    }

    fn send_5s_metrics(&self, ctx: &mut actix::Context<Self>) {
        let delay = Duration::new(5, 0);

        ctx.run_later(delay, move |hub, ctx| {
            hub.send_metrics(ctx, View::OverviewFiveSeconds);
            hub.send_5s_metrics(ctx);
        });
    }

    fn send_15s_metrics(&self, ctx: &mut actix::Context<Self>) {
        let delay = Duration::new(15, 0);

        ctx.run_later(delay, move |hub, ctx| {
            hub.send_metrics(ctx, View::OverviewFifteenSeconds);
            hub.send_15s_metrics(ctx);
        });
    }

    fn send_node_history(
        &mut self,
        receiver_id: usize,
        subscribe_to: View,
        _: &mut actix::Context<Self>,
    ) {
        for (hostname, buffer) in self.node_buffers.storage() {
            if let Some(specs) = self.node_specs.get(hostname) {
                let nodes: Vec<Node> = buffer
                    .storage(subscribe_to.clone())
                    .iter()
                    .cloned()
                    .map(|metrics| Node::new(specs.clone(), metrics))
                    .collect();
                self.send_to_server(OutboundMessage {
                    receiver: Receiver::Only(receiver_id),
                    data: NodeMetrics(nodes),
                });
            }
        }
    }

    fn send_cluster_history(
        &mut self,
        receiver_id: usize,
        subscribe_to: View,
        _: &mut actix::Context<Self>,
    ) {
        let specs = self.cluster_specs.clone();
        let node_history: Vec<Node> = self
            .cluster_buffer
            .storage(subscribe_to.clone())
            .iter()
            .cloned()
            .map(|metrics| Node::new(specs.clone(), metrics))
            .collect();
        self.send_to_server(OutboundMessage {
            receiver: Receiver::Only(receiver_id),
            data: ClusterMetrics(node_history),
        });
    }
}

impl Actor for MetricHub {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.aggregate_cluster_metrics(ctx);
        self.send_1s_metrics(ctx);
        self.send_5s_metrics(ctx);
        self.send_15s_metrics(ctx);
    }
}

impl Handler<NodeMetrics> for MetricHub {
    type Result = ();

    fn handle(&mut self, metrics: NodeMetrics, _: &mut Context<Self>) {
        self.latest_metrics
            .insert(metrics.hostname().to_string(), metrics.clone());
    }
}

impl Handler<NodeSpecs> for MetricHub {
    type Result = ();

    fn handle(&mut self, specs: NodeSpecs, _: &mut Context<Self>) {
        self.node_specs
            .insert(specs.hostname().to_string(), specs.clone());
    }
}

impl Handler<ClientJoined> for MetricHub {
    type Result = ();

    fn handle(&mut self, msg: ClientJoined, ctx: &mut Context<Self>) {
        self.ws_server = Some(msg.ws_server);
        self.send_node_history(msg.session_id, msg.subscribe_to.clone(), ctx);
        self.send_cluster_history(msg.session_id, msg.subscribe_to.clone(), ctx);
    }
}
