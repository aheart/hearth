use actix;
use actix::*;
use actix_web::{HttpServer, web};
use actix_web::{App};
use actix_files as fs;
use crate::metrics::aggreagtor::*;
use crate::ws::ws_route;
use crate::ws::server::WsServer;
use crate::config::Config;
use env_logger;
use log::info;

pub fn run(config: Config) {
    let env = env_logger::Env::default()
        .filter_or(env_logger::DEFAULT_FILTER_ENV, "info");

    env_logger::Builder::from_env(env).init();

    let sys = actix::System::new("hearth");
    let ws_server = WsServer::default().start();

    config.servers.as_ref()
        .expect("No servers are listed in the config. Can't continue.")
        .iter()
        .enumerate()
        .for_each(|(index, server_config)| {
            let metric_hub = metric_aggregator_factory(
                ws_server.clone(),
                server_config,
                index,
            );
            Actor::start_in_arbiter(&Arbiter::new(), |_| metric_hub);
        });

    HttpServer::new(move || {
        App::new()
            .data(ws_server.clone())
            .service(web::resource("/ws/").to(ws_route))
            .service(fs::Files::new("/", "./static/").index_file("index.html"))
    }).bind(config.address())
        .expect("Can not start server on given IP/Port")
        .start();

    info!("Starting http server: {}", config.address());
    sys.run().expect("There might be a bug in the Actor System");
}
