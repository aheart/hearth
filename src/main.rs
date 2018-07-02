extern crate futures;
extern crate rand;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate serde;
extern crate serde_json;
extern crate toml;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate actix;
extern crate actix_web;
extern crate ssh2;

mod config;
mod metrics;
mod ssh;
mod ws;

use actix::*;
use actix_web::server::HttpServer;
use actix_web::{fs, App};
use metrics::aggreagtor::*;
use ws::server::WsServer;
use ws::session::WsSessionState;

fn main() {
    env_logger::init();
    let config = config::load_config();

    let sys = actix::System::new("hearth");
    let server: Addr<Syn, _> = Arbiter::start(|_| WsServer::default());

    for (index, server_config) in config.servers.unwrap().iter().enumerate() {
        let metric_hub = metric_aggregator_factory(
            server.clone(),
            server_config.username.clone(),
            server_config.hostname.clone(),
            index,
        );
        let _: Addr<Syn, _> = Arbiter::start(|_| metric_hub);
    }

    let address = format!("{}:{}", config.address, config.port);

    HttpServer::new(move || {
        let state = WsSessionState {
            addr: server.clone(),
        };

        App::with_state(state)
            .resource("/ws/", |r| r.route().f(ws::ws_route))
            .handler("/", fs::StaticFiles::new("static/").index_file("index.html"))
    }).bind(address.clone())
        .expect("Can not start server on given IP/Port")
        .start();

    info!("Starting http server: {}", address);
    let _ = sys.run();
}
