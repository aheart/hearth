use serde_derive::Deserialize;
use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;
use toml;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub ip: String,
    pub port: u32,
    #[serde(default)]
    pub authentication: AuthMethod,
    pub servers: Option<Vec<ServerConfig>>,
}

impl Config {
    pub fn address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "method")]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    SshAgent,
    PubKey(PubKeyConfig),
}

impl Default for AuthMethod {
    fn default() -> Self {
        AuthMethod::SshAgent
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct PubKeyConfig {
    pub public_key: Option<String>,
    pub private_key: String,
    pub passphrase: Option<String>,
}

impl PubKeyConfig {
    pub fn public_key_path(&self) -> Option<&Path> {
        self.public_key.as_ref().map(Path::new)
    }

    pub fn private_key_path(&self) -> &Path {
        Path::new(&self.private_key)
    }

    pub fn passphrase(&self) -> Option<&str> {
        self.passphrase.as_ref().map(|p| p.as_ref())
    }
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub hostname: String,
    pub username: String,
    pub disk: String,
    pub filesystem: String,
    pub network_interface: String,
}

pub fn load_config() -> Result<Config, Box<dyn Error>> {
    let config_toml = read_to_string("config.toml")?;
    Ok(toml::from_str(&config_toml)?)
}
