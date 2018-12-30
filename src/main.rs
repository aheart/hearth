mod app;
mod config;
mod metrics;
mod ssh;
mod ws;

use std::alloc::System;

#[global_allocator]
static GLOBAL: System = System;

fn main() {
    let config = config::load_config().expect("Failed to load configuration from config.toml");
    app::run(config);
}
