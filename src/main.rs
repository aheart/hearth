mod app;
mod config;
mod metrics;
mod ssh;
mod ws;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let config = config::load_config().expect("Failed to load configuration from config.toml");
    app::run(config).await
}
