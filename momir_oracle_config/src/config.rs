use config::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub printer: PrinterConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PrinterConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub usb_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: i32,
    pub scryfall_user_agent: String,
}

pub fn load_config() -> Result<AppConfig, config::ConfigError> {
    let config = Config::builder()
        // Defaults
        .set_default("server.host", "0.0.0.0")?
        .set_default("server.port", 8080)?
        .set_default("server.scryfall_user_agent", "momir_basic_rs/v0.1")?
        // TOML overrides defaults
        .add_source(config::File::with_name("momir_config").required(false))
        .build()?;
    config.try_deserialize()
}
