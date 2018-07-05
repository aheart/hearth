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

mod app;
mod config;
mod metrics;
mod ssh;
mod ws;

fn main() {
    let config = config::load_config().expect("Failed to load configuration from config.toml");
    app::run(config);
}
