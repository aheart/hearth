use std::fs::read_to_string;
use toml;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub hostname: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub address: String,
    pub port: u32,
    pub servers: Option<Vec<ServerConfig>>,
}

pub fn load_config() -> Config {
    let config_toml = read_to_string("config.toml").expect("Unable to read config.toml");
    toml::from_str(&config_toml).expect("config.toml contains invalid TOML")
}
