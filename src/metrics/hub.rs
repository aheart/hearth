use super::metric_buffer::{MetricBuffer, MetricBufferMap};
use crate::metrics::aggregator::NodeMetrics;
use crate::ws::server::MessageData::*;
use crate::ws::server::{ClientJoined, OutboundMessage, Receiver, View, WsServer};
use actix::prelude::*;
use std::time::Duration;

/// Metric Hub
///
/// It receives metrics from the MetricAggregator, stores them and provides to clients
pub struct MetricHub {
    ws_server: Option<Addr<WsServer>>,
    node_buffers: MetricBufferMap,
    cluster_buffer: MetricBuffer,
}

impl Default for MetricHub {
    fn default() -> Self {
        Self {
            ws_server: None,
            node_buffers: MetricBufferMap::new(120),
            cluster_buffer: MetricBuffer::new(120),
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
            hub.cluster_buffer.push(cluster.clone());

            hub.aggregate_cluster_metrics(ctx);
        });
    }

    fn send_metrics(&self, ctx: &mut actix::Context<Self>, timeframe: View) {
        for metrics in self.node_buffers.storage().values() {
            if metrics.storage(timeframe).len() > 0 {
                self.send_to_server(OutboundMessage {
                    receiver: Receiver::SubscribersOf(timeframe),
                    data: NodeMetrics(vec![metrics.storage(timeframe).last().unwrap().clone()]),
                });
            }
        }
        if self.cluster_buffer.storage(timeframe).len() > 0 {
            self.send_to_server(OutboundMessage {
                receiver: Receiver::SubscribersOf(timeframe),
                data: ClusterMetrics(vec![self
                    .cluster_buffer
                    .storage(timeframe)
                    .last()
                    .unwrap()
                    .clone()]),
            });
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
        for buffer in self.node_buffers.storage().values() {
            self.send_to_server(OutboundMessage {
                receiver: Receiver::Only(receiver_id),
                data: NodeMetrics(buffer.storage(subscribe_to.clone()).clone()),
            });
        }
    }

    fn send_cluster_history(
        &mut self,
        receiver_id: usize,
        subscribe_to: View,
        _: &mut actix::Context<Self>,
    ) {
        self.send_to_server(OutboundMessage {
            receiver: Receiver::Only(receiver_id),
            data: ClusterMetrics(self.cluster_buffer.storage(subscribe_to).clone()),
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
        self.node_buffers.push(metrics.hostname(), metrics.clone());
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
