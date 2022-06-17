use crate::config::Config;
use crate::metrics::aggregator::*;
use crate::metrics::hub::MetricHub;
use crate::ws::server::WsServer;
use crate::ws::ws_route;
use actix::*;
use actix_files as fs;
use actix_web::App;
use actix_web::{web, HttpServer};

pub async fn run(config: Config) -> std::io::Result<()> {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::Builder::from_env(env).init();

    let hub = MetricHub::default().start();
    let ws_server = web::Data::new(WsServer::new(hub.clone()).start());

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
            Actor::start_in_arbiter(&Arbiter::new().handle(), |_| aggregator);
        });

    HttpServer::new(move || {
        App::new()
            .app_data(ws_server.clone())
            // .service(web::resource("/ws/").to(ws_route))
            .route("/ws/", web::get().to(ws_route))
            .service(fs::Files::new("/", "./static/").index_file("index.html"))
    })
    .bind(config.address())
    .expect("Can not start server on given IP/Port")
    .run()
    .await
}
