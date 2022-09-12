pub use config::ConfigError;
use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Configuration {
    pub page_limit: i64,
    pub save_file_base_path: String,
    pub uploads_endpoint: String,
    pub rust_log: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub allowed_host: String,
}

impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::default())?;
        cfg.try_into()
    }
}

lazy_static! {
    pub static ref CONFIG: Configuration = Configuration::new().expect("Config can be loaded");
}
