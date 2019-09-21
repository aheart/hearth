use super::session::SessionMessage;
use crate::metrics::aggregator::NodeMetrics;
use crate::metrics::hub::MetricHub;
use crate::ws::session::{Connect, Disconnect};
use actix::prelude::*;
use log::info;
use rand::prelude::*;
use serde_derive::Serialize;
use serde_json;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Message, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum MessageData {
    NodeMetrics(Vec<NodeMetrics>),
    ClusterMetrics(Vec<NodeMetrics>),
}

#[derive(Message, Clone, Serialize)]
pub enum Receiver {
    Everyone,
    Only(usize),
}

#[derive(Message, Clone, Serialize)]
pub struct OutboundMessage {
    #[serde(skip)]
    pub receiver: Receiver,
    #[serde(flatten)]
    pub data: MessageData,
}

#[derive(Message)]
pub struct ClientJoined {
    pub session_id: usize,
    pub ws_server: Addr<WsServer>,
}

/// WebSocket Server
///
/// Manages communication with Clients
pub struct WsServer {
    hub: Addr<MetricHub>,
    sessions: HashMap<usize, Recipient<SessionMessage>>,
    rng: RefCell<ThreadRng>,
}

impl WsServer {
    pub fn new(hub: Addr<MetricHub>) -> Self {
        Self {
            hub,
            sessions: HashMap::new(),
            rng: RefCell::new(rand::thread_rng()),
        }
    }

    fn broadcast(&mut self, message: &str) {
        for addr in self.sessions.values() {
            let _ = addr.do_send(SessionMessage(message.to_owned()));
        }
    }

    fn unicast(&mut self, message: &str, receiver: usize) {
        if let Some(addr) = self.sessions.get(&receiver) {
            let _ = addr.do_send(SessionMessage(message.to_owned()));
        }
    }
}

impl Actor for WsServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for WsServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, ctx: &mut Context<Self>) -> Self::Result {
        let id = self.rng.borrow_mut().gen::<usize>();
        self.sessions.insert(id, msg.addr.clone());
        info!(
            "Client {} connected. Active sessions: {}",
            msg.ip,
            self.sessions.len()
        );

        self.hub.do_send(ClientJoined {
            ws_server: ctx.address(),
            session_id: id,
        });

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

impl Handler<OutboundMessage> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: OutboundMessage, _: &mut Context<Self>) {
        let message = serde_json::to_string(&msg).unwrap();
        match msg.receiver {
            Receiver::Everyone => self.broadcast(message.as_str()),
            Receiver::Only(id) => self.unicast(message.as_str(), id),
        }
    }
}
