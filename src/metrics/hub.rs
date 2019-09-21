use super::metric_buffer::{MetricBuffer, MetricBufferMap};
use crate::metrics::aggregator::NodeMetrics;
use crate::ws::server::MessageData::*;
use crate::ws::server::{ClientJoined, OutboundMessage, Receiver, WsServer};
use actix::prelude::*;
use std::time::Duration;

/// Metric Hub
///
/// Will be responsible for Metric buffering
/// Will aggregate data from all aggregators
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

    fn aggregate_history(&self, ctx: &mut actix::Context<Self>) {
        let delay = Duration::new(1, 0);

        ctx.run_later(delay, move |hub, ctx| {
            let latest_node_metrics: Vec<NodeMetrics> = hub
                .node_buffers
                .storage()
                .values()
                .map(|buffer| {
                    buffer
                        .storage()
                        .last()
                        .cloned()
                        .expect("Can't even aggregate these days")
                })
                .collect();

            let cluster = NodeMetrics::aggregate(latest_node_metrics);
            hub.cluster_buffer.push(cluster.clone());
            hub.send_to_server(OutboundMessage {
                receiver: Receiver::Everyone,
                data: ClusterMetrics(vec![cluster]),
            });

            hub.aggregate_history(ctx);
        });
    }

    fn send_node_history(&mut self, receiver_id: usize, _: &mut actix::Context<Self>) {
        for buffer in self.node_buffers.storage().values() {
            self.send_to_server(OutboundMessage {
                receiver: Receiver::Only(receiver_id),
                data: NodeMetrics(buffer.storage().clone()),
            });
        }
    }

    fn send_cluster_history(&mut self, receiver_id: usize, _: &mut actix::Context<Self>) {
        self.send_to_server(OutboundMessage {
            receiver: Receiver::Only(receiver_id),
            data: ClusterMetrics(self.cluster_buffer.storage().clone()),
        });
    }
}

impl Actor for MetricHub {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.aggregate_history(ctx);
    }
}

impl Handler<NodeMetrics> for MetricHub {
    type Result = ();

    fn handle(&mut self, metrics: NodeMetrics, _: &mut Context<Self>) {
        self.node_buffers.push(metrics.hostname(), metrics.clone());

        let ws_message = OutboundMessage {
            receiver: Receiver::Everyone,
            data: NodeMetrics(vec![metrics]),
        };
        self.send_to_server(ws_message)
    }
}

impl Handler<ClientJoined> for MetricHub {
    type Result = ();

    fn handle(&mut self, msg: ClientJoined, ctx: &mut Context<Self>) {
        self.ws_server = Some(msg.ws_server);
        self.send_node_history(msg.session_id, ctx);
        self.send_cluster_history(msg.session_id, ctx);
    }
}
