use super::server::WsServer;
use crate::ws::server::{InboundMessage, View};
use actix::prelude::*;
use actix_web_actors::ws;
use log::{error, warn};
use serde_derive::Deserialize;
use serde_json;
use std::time::Instant;

#[derive(Deserialize)]
struct Command {
    subscribe_to: View,
}

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

#[derive(Message)]
pub struct SessionMessage(pub String);

pub struct WsSession {
    pub id: usize,
    pub hb: Instant,
    pub ip: String,
    pub ws_server: Addr<WsServer>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.ws_server
            .send(Connect {
                addr: addr.recipient(),
                ip: self.ip.clone(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    _ => ctx.stop(),
                }
                fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.ws_server.do_send(Disconnect {
            sender_id: self.id,
            ip: self.ip.clone(),
        });
        Running::Stop
    }
}

impl Handler<SessionMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: SessionMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WsSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Pong(_) => self.hb = Instant::now(),
            ws::Message::Text(text) => {
                match serde_json::from_str::<Command>(&text) {
                    Ok(msg) => self.ws_server.do_send(InboundMessage {
                        session_id: self.id,
                        subscribe_to: msg.subscribe_to,
                    }),
                    Err(e) => error!("{:?}", e),
                };
            }
            ws::Message::Binary(_) => warn!("Unexpected binary"),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
