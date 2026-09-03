use config::Config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PrintMethod {
    Network,
    Usb,
}

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
pub struct AppConfig {
    pub printer: PrinterConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
pub struct PrinterConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub usb_path: Option<String>,
    pub print_method: Option<PrintMethod>,
}

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: i32,
    pub scryfall_user_agent: String,
    pub cache_dir: Option<String>,
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
