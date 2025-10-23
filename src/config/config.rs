use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{fmt, str::FromStr};

use crate::errors::config::ConfigError;
use crate::types::logger::LogLevel;
use crate::utils::mask_string::{mask_email, mask_secret};
use crate::utils::{
    allow_email_input_default, csv_to_vec, default_log_level,
    duplicate_emails_to_deafult_recipients_everytime_default, looks_like_email,
    rate_defaults::{
        rate_limit_max_default, rate_limit_timeframe_seconds_default, use_rate_limit_default,
    },
};

pub static CONFIG: Lazy<Settings> = Lazy::new(|| load().expect("Failed to load configuration"));

#[allow(dead_code)]
#[derive(Clone, Deserialize)]
pub struct Settings {
    pub port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,

    pub smtp_server: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,

    #[serde(default = "allow_email_input_default")]
    pub allow_email_input: bool,

    #[serde(default = "duplicate_emails_to_deafult_recipients_everytime_default")]
    pub duplicate_emails_to_deafult_recipients_everytime: bool,

    #[serde(default = "use_rate_limit_default")]
    pub use_rate_limit: bool,
    #[serde(default = "rate_limit_timeframe_seconds_default")]
    pub rate_limit_timeframe_seconds: u32,
    #[serde(default = "rate_limit_max_default")]
    pub rate_limit_max: u32,

    #[serde(deserialize_with = "csv_to_vec")]
    pub emails: Option<Vec<String>>,

    #[serde(deserialize_with = "csv_to_vec")]
    pub cors_origins: Option<Vec<String>>,
}

pub fn load() -> Result<Settings, ConfigError> {
    dotenvy::dotenv().ok();

    let cfg = config::Config::builder()
        .add_source(config::File::with_name("Settings").required(false))
        .add_source(
            config::Environment::with_prefix("APP")
                .separator("__")
                .try_parsing(true),
        )
        .build()
        .map_err(ConfigError::Build)?;

    let settings: Settings = cfg.try_deserialize().map_err(ConfigError::Deserialize)?;
    settings.validate()?;
    Ok(settings)
}

impl Settings {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !(1..=65535).contains(&self.port) {
            return Err(ConfigError::Invalid("port must be 1..=65535".into()));
        }
        if !(1..=65535).contains(&self.smtp_port) {
            return Err(ConfigError::Invalid("smtp_port must be 1..=65535".into()));
        }
        if self.smtp_server.trim().is_empty() {
            return Err(ConfigError::Invalid("smtp_server is empty".into()));
        }
        if self.smtp_user.trim().is_empty() {
            return Err(ConfigError::Invalid("smtp_user is empty".into()));
        }
        if self.smtp_password.is_empty() {
            return Err(ConfigError::Invalid("smtp_password is empty".into()));
        }
        if let Some(list) = &self.emails {
            for e in list {
                if !looks_like_email(e) {
                    return Err(ConfigError::Invalid(format!("invalid email: {e}")));
                }
            }
        }
        Ok(())
    }

    pub fn init_tracing(&self) {
        if std::env::var_os("RUST_LOG").is_none() {
            let filter = LogLevel::from_str(&self.log_level)
                .unwrap_or(LogLevel::Warn)
                .as_filter_str();
            unsafe {
                std::env::set_var("RUST_LOG", filter);
            }
        }
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init();
    }

    pub fn log_effective(&self) {
        if !(cfg!(debug_assertions) || self.log_level.eq_ignore_ascii_case("debug")) {
            return;
        }
        use tracing::{debug, info};

        info!(
            "Runtime mode: {} | log_level: {}",
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            self.log_level
        );
        debug!(config = ?Redacted(self));
    }
}

struct Redacted<'a>(&'a Settings);

impl<'a> fmt::Debug for Redacted<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0;
        f.debug_struct("Settings")
            .field("port", &s.port)
            .field("log_level", &s.log_level)
            .field("smtp_server", &s.smtp_server)
            .field("smtp_port", &s.smtp_port)
            .field("smtp_user", &mask_email(&s.smtp_user))
            .field("smtp_password", &mask_secret(&s.smtp_password, 2))
            .field("allow_email_input", &s.allow_email_input)
            .field("use_rate_limit", &s.use_rate_limit)
            .field("rate_limit_timeframe", &s.rate_limit_timeframe_seconds)
            .field("rate_limit_max", &s.rate_limit_max)
            .field(
                "emails",
                &s.emails
                    .as_ref()
                    .map(|v| v.iter().map(|e| mask_email(e)).collect::<Vec<_>>()),
            )
            .field(
                "duplicate_emails_to_deafult_recipients_everytime",
                &s.duplicate_emails_to_deafult_recipients_everytime,
            )
            .field("cors_origins", &s.cors_origins)
            .finish()
    }
}
