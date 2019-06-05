use actix::prelude::*;
use rand::prelude::*;
use serde_json;
use std::cell::RefCell;
use std::collections::HashMap;
use log::info;
use crate::metrics::aggreagtor::MetricAggregate;
use super::session::SessionMessage;

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<SessionMessage>,
    pub ip: String,
}

#[derive(Message)]
pub struct Disconnect {
    pub id: usize,
    pub ip: String,
}

#[derive(Message, Clone)]
pub struct Message {
    pub id: usize,
    pub metrics: MetricAggregate,
}

pub struct WsServer {
    sessions: HashMap<usize, Recipient<SessionMessage>>,
    rng: RefCell<ThreadRng>,
    metric_buffer: HashMap<String, Vec<MetricAggregate>>,
}

impl Default for WsServer {
    fn default() -> WsServer {
        WsServer {
            sessions: HashMap::new(),
            rng: RefCell::new(rand::thread_rng()),
            metric_buffer: HashMap::new(),
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
}

impl Actor for WsServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for WsServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _ctx: &mut Context<Self>) -> Self::Result {
        let id = self.rng.borrow_mut().gen::<usize>();
        self.sessions.insert(id, msg.addr.clone());
        info!("Client {} connected. Active sessions: {}", msg.ip, self.sessions.len());

        for server in self.metric_buffer.values() {
            let payload = serde_json::to_string(&server).expect("Unable to serialize metrics");
            let _ = msg.addr.do_send(SessionMessage(payload));
        }

        id
    }
}

impl Handler<Disconnect> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions
            .remove(&msg.id)
            .expect("There is a bug in handling of WS Disconnect messages");
        info!("Client {} disconnected. Active sessions: {}", msg.ip, self.sessions.len());
    }
}

impl Handler<Message> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Message, _: &mut Context<Self>) {
        let message = serde_json::to_string(&msg.metrics).unwrap();
        self.send_message(message.as_str(), msg.id);

        let hostname = &msg.metrics.server;

        if let Some(server_history) = self.metric_buffer.get_mut(hostname) {
            server_history.push(msg.metrics);
            if server_history.len() > 120 {
                server_history.drain(0..1);
            }
        } else {
            self.metric_buffer.insert(hostname.to_string(), vec![msg.metrics]);
        }
    }
}
