use super::session::SessionMessage;
use crate::metrics::aggreagtor::NodeMetrics;
use actix::prelude::*;
use log::info;
use rand::prelude::*;
use serde_json;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<SessionMessage>,
    pub ip: String,
}

#[derive(Message)]
pub struct Disconnect {
    pub sender_id: usize,
    pub ip: String,
}

#[derive(Message, Clone)]
pub struct Message {
    pub sender_id: usize,
    pub metrics: NodeMetrics,
}

pub struct WsServer {
    sessions: HashMap<usize, Recipient<SessionMessage>>,
    rng: RefCell<ThreadRng>,
    node_buffer: HashMap<String, Vec<NodeMetrics>>,
    cluster_buffer: Vec<NodeMetrics>,
}

impl Default for WsServer {
    fn default() -> WsServer {
        WsServer {
            sessions: HashMap::new(),
            rng: RefCell::new(rand::thread_rng()),
            node_buffer: HashMap::new(),
            cluster_buffer: Vec::new(),
        }
    }
}

impl WsServer {
    fn send_message(&mut self, message: &str, skip_id: usize) {
        for (id, addr) in &self.sessions {
            if *id != skip_id {
                let _ = addr.do_send(SessionMessage(message.to_owned()));
            };
        }
    }

    fn aggregate_history(&self, ctx: &mut actix::Context<Self>) {
        let delay = Duration::new(1, 0);

        ctx.run_later(delay, move |server, ctx| {
            let mut cluster = NodeMetrics::default();
            for history in server.node_buffer.values() {
                let metric = history
                    .last()
                    .cloned()
                    .expect("Can't even aggregate these days");
                cluster = cluster + metric;
            }
            if cluster.cpus > 0 {
                cluster.cpu = cluster.cpu.divide(server.node_buffer.len() as f32);

                cluster.hostname = "Cluster".to_string();
                let payload = serde_json::to_string(&cluster).expect("Unable to serialize metrics");
                server.cluster_buffer.push(cluster);
                if server.cluster_buffer.len() > 120 {
                    server.cluster_buffer.drain(0..1);
                };
                server.send_message(&payload, 0);
            }

            server.aggregate_history(ctx);
        });
    }

    fn send_node_history(&mut self, addr: &Recipient<SessionMessage>) {
        for server in self.node_buffer.values() {
            let payload = serde_json::to_string(&server).expect("Unable to serialize metrics");
            let _ = addr.do_send(SessionMessage(payload));
        }
    }

    fn send_cluster_history(&mut self, addr: &Recipient<SessionMessage>) {
        let payload =
            serde_json::to_string(&self.cluster_buffer).expect("Unable to serialize metrics");
        let _ = addr.do_send(SessionMessage(payload));
    }
}

impl Actor for WsServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.aggregate_history(ctx);
    }
}

impl Handler<Connect> for WsServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _ctx: &mut Context<Self>) -> Self::Result {
        let id = self.rng.borrow_mut().gen::<usize>();
        self.sessions.insert(id, msg.addr.clone());
        info!(
            "Client {} connected. Active sessions: {}",
            msg.ip,
            self.sessions.len()
        );

        self.send_node_history(&msg.addr);
        self.send_cluster_history(&msg.addr);

        id
    }
}

impl Handler<Disconnect> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions
            .remove(&msg.sender_id)
            .expect("There is a bug in handling of WS Disconnect messages");
        info!(
            "Client {} disconnected. Active sessions: {}",
            msg.ip,
            self.sessions.len()
        );
    }
}

impl Handler<Message> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Message, _: &mut Context<Self>) {
        let message = serde_json::to_string(&msg.metrics).unwrap();
        self.send_message(message.as_str(), msg.sender_id);

        let hostname = &msg.metrics.hostname;

        if let Some(server_history) = self.node_buffer.get_mut(hostname) {
            server_history.push(msg.metrics);
            if server_history.len() > 120 {
                server_history.drain(0..1);
            }
        } else {
            self.node_buffer
                .insert(hostname.to_string(), vec![msg.metrics]);
        }
    }
}
