use actix;
use actix::*;
use actix_web::server::HttpServer;
use actix_web::{fs, App};
use metrics::aggreagtor::*;
use ws::ws_route;
use ws::server::WsServer;
use ws::session::WsSessionState;
use config::Config;
use env_logger;

pub fn run(config: Config) {
    let env = env_logger::Env::default()
        .filter_or(env_logger::DEFAULT_FILTER_ENV, "info");

    env_logger::Builder::from_env(env).init();

    let sys = actix::System::new("hearth");
    let ws_server = Arbiter::start(|_| WsServer::default());

    for (index, server_config) in config.servers.as_ref().unwrap().iter().enumerate() {
        let metric_hub = metric_aggregator_factory(
            ws_server.clone(),
            server_config.username.clone(),
            server_config.hostname.clone(),
            index,
        );
        let _ = Arbiter::start(|_| metric_hub);
    }

    HttpServer::new(move || {
        let state = WsSessionState {
            addr: ws_server.clone(),
        };

        App::with_state(state)
            .resource("/ws/", |r| r.route().f(ws_route))
            .handler(
                "/",
                fs::StaticFiles::new("static/")
                    .expect("Unable to initialize static resources")
                    .index_file("index.html")
            )
    }).bind(config.address())
        .expect("Can not start server on given IP/Port")
        .start();

    info!("Starting http server: {}", config.address());
    let _ = sys.run();
}