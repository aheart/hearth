use crate::config::Config;
use crate::metrics::aggregator::*;
use crate::metrics::hub::MetricHub;
use crate::ws::server::WsServer;
use crate::ws::ws_route;
use actix;
use actix::*;
use actix_files as fs;
use actix_web::App;
use actix_web::{web, HttpServer};
use env_logger;

pub fn run(config: Config) {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::Builder::from_env(env).init();
    let sys = actix::System::new("hearth");

    let hub = MetricHub::default().start();
    let ws_server = WsServer::new(hub.clone()).start();

    config
        .servers
        .as_ref()
        .expect("No servers are listed in the config. Can't continue.")
        .iter()
        .enumerate()
        .for_each(|(index, server_config)| {
            let aggregator = metric_aggregator_factory(
                hub.clone(),
                server_config,
                config.authentication.clone(),
                index as u8 + 1,
            );
            Actor::start_in_arbiter(&Arbiter::new(), |_| aggregator);
        });

    HttpServer::new(move || {
        App::new()
            .data(ws_server.clone())
            .service(web::resource("/ws/").to(ws_route))
            .service(fs::Files::new("/", "./static/").index_file("index.html"))
    })
    .bind(config.address())
    .expect("Can not start server on given IP/Port")
    .run();

    sys.run().expect("There might be a bug in the Actor System");
}
