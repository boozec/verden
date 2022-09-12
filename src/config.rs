pub use config::ConfigError;
use lazy_static::lazy_static;
use serde::Deserialize;

// pub static PAGE_LIMIT: i64 = std::env::var("PAGE_LIMIT")
//     .unwrap_or_else(|_| "20".to_string())
//     .parse::<i64>()
//     .unwrap();
// pub const MAX_UPLOAD_FILE_SIZE: u64 = 1024 * 1024; // 1 MB
// pub const SAVE_FILE_BASE_PATH: &str = &std::env::var("SAVE_FILE_BASE_PATH").unwrap();
// pub const UPLOADS_ENDPOINT: &str = &std::env::var("UPLOADS_ENDPOINT").unwrap();

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
