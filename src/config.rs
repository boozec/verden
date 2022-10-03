pub use config::ConfigError;
use lazy_static::lazy_static;
use sentry::ClientInitGuard;
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

pub struct Sentry(pub ClientInitGuard);

impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::default())?;
        cfg.try_into()
    }
}

impl Sentry {
    pub fn new() -> Result<Self, std::env::VarError> {
        match std::env::var("SENTRY_DSN") {
            Ok(dsn) => {
                let guard = sentry::init((
                    dsn.clone(),
                    sentry::ClientOptions {
                        release: sentry::release_name!(),
                        ..Default::default()
                    },
                ));

                Ok(Self(guard))
            }
            Err(e) => Err(e),
        }
    }
}

lazy_static! {
    pub static ref CONFIG: Configuration = Configuration::new().expect("Config can be loaded");
    pub static ref SENTRY: Sentry = Sentry::new().expect("Sentry not configured.");
}
