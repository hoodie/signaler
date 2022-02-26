#[derive(Debug, serde::Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}
#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub log_config: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(config::Config::builder()
            .add_source(config::Environment::default())
            .build()?
            .try_deserialize()?)
    }
}
