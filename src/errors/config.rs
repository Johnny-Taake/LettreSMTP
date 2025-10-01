use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config build error: {0}")]
    Build(config::ConfigError),
    #[error("config deserialize error: {0}")]
    Deserialize(config::ConfigError),
    #[error("invalid configuration: {0}")]
    Invalid(String),
}
