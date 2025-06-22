use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvConfigError {
    #[error("Missing required environment variable: {var_name}")]
    MissingVar { var_name: String },

    #[error("Invalid value for {var_name}: {value}")]
    InvalidValue { var_name: String, value: String },

    #[error("Configuration file error: {0}")]
    Figment(#[from] figment::Error),

    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),

    #[error("Parse error for {var_name}: {source}")]
    ParseError {
        var_name: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl From<std::num::ParseIntError> for EnvConfigError {
    fn from(err: std::num::ParseIntError) -> Self {
        EnvConfigError::ParseError {
            var_name: "unknown".to_string(),
            source: Box::new(err),
        }
    }
}
