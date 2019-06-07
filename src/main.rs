mod app;
mod config;
mod metrics;
mod ssh;
mod ws;

fn main() {
    let config = config::load_config().expect("Failed to load configuration from config.toml");
    app::run(config);
}
