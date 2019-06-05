use super::server::{Connect, Disconnect, WsServer};
use actix::prelude::*;
use actix_web_actors::ws;
use std::time::Instant;
use log::warn;

#[derive(Message)]
pub struct SessionMessage(pub String);

pub struct WsSession {
    pub id: usize,
    pub hb: Instant,
    pub ip: String,
    pub addr: Addr<WsServer>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self
            .addr
            .send(Connect {
                addr: addr.recipient(),
                ip: self.ip.clone()
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
        self.addr.do_send(Disconnect { id: self.id, ip: self.ip.clone() });
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
            ws::Message::Text(_) => warn!("Unexpected text message"),
            ws::Message::Binary(_) => warn!("Unexpected binary"),
            ws::Message::Close(_) => {
                ctx.stop();
            },
            ws::Message::Nop => (),
        }
    }
}
