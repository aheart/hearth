use actix::prelude::*;
use actix::*;
use rand::{self, Rng, ThreadRng};
use serde_json;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<super::session::SessionMessage>,
}

#[derive(Message)]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message, Clone)]
pub struct Message {
    pub id: usize,
    pub metrics: HashMap<String, String>,
}

pub struct WsServer {
    sessions: HashMap<usize, Recipient<super::session::SessionMessage>>,
    clients: HashSet<usize>,
    rng: RefCell<ThreadRng>,
    metric_buffer: Vec<HashMap<String, String>>,
}

impl Default for WsServer {
    fn default() -> WsServer {
        WsServer {
            sessions: HashMap::new(),
            clients: HashSet::new(),
            rng: RefCell::new(rand::thread_rng()),
            metric_buffer: vec![],
        }
    }
}

impl WsServer {
    fn send_message(&mut self, message: &str, skip_id: usize) {
        for id in &self.clients {
            if *id != skip_id {
                if let Some(addr) = self.sessions.get(id) {
                    let _ = addr.do_send(super::session::SessionMessage(message.to_owned()));
                };
            };
        }
    }
}

impl Actor for WsServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for WsServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, ctx: &mut Context<Self>) -> Self::Result {
        info!("Someone joined");

        let id = self.rng.borrow_mut().gen::<usize>();
        self.sessions.insert(id, msg.addr.clone());

        self.clients.insert(id);

        let addr = msg.addr;
        let msg = serde_json::to_string(&self.metric_buffer).unwrap();
        addr.send(super::session::SessionMessage(msg.clone()))
            .into_actor(self)
            .then(|_res, _act, _ctx| fut::ok(()))
            .wait(ctx);
        id
    }
}

impl Handler<Disconnect> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        info!("Someone disconnected");

        if self.sessions.remove(&msg.id).is_some() {
            self.clients.remove(&msg.id);
        }
    }
}

impl Handler<Message> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Message, _: &mut Context<Self>) {
        self.metric_buffer.push(msg.metrics.clone());
        if self.metric_buffer.len() > 1320 {
            self.metric_buffer.drain(0..1);
        }
        let message = serde_json::to_string(&msg.metrics).unwrap();
        self.send_message(message.as_str(), msg.id);
    }
}
