pub mod server;
pub mod session;

use actix::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::time::Instant;

pub fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::WsServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        session::WsSession {
            id: 0,
            hb: Instant::now(),
            ip: req.peer_addr().map(|s| s.to_string()).unwrap_or_else(|| "Unknown IP".to_string()),
            addr: srv.get_ref().clone(),
        },
        &req,
        stream
    )
}
