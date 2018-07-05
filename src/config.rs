use std::error::Error;
use std::fs::read_to_string;
use toml;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub hostname: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub ip: String,
    pub port: u32,
    pub servers: Option<Vec<ServerConfig>>,
}

impl Config {
    pub fn address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

pub fn load_config() -> Result<Config, Box<Error>> {
    let config_toml = read_to_string("config.toml")?;
    Ok(toml::from_str(&config_toml)?)
}
