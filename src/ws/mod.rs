pub mod server;
pub mod session;

use actix_web::{ws, Error, HttpRequest, HttpResponse};
use std::time::Instant;

pub fn ws_route(req: &HttpRequest<session::WsSessionState>) -> Result<HttpResponse, Error> {
    ws::start(
        req,
        session::WsSession {
            id: 0,
            hb: Instant::now(),
            ip: req.peer_addr().map(|s| s.to_string()).unwrap_or("Unknown IP".to_string())
        },
    )
}
