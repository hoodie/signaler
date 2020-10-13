use config::ConfigError;

#[derive(Debug, serde::Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub stop_on_panic: bool,
    pub log_config: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::new())?;
        cfg.try_into()
    }
}
