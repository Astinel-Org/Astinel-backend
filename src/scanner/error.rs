use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScanError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Parser error: {0}")]
    Parser(String),

    #[error("Rule execution error: {0}")]
    Rule(String),

    #[error("AI analysis error: {0}")]
    AI(String),

    #[error("Report generation error: {0}")]
    Report(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid project: {0}")]
    InvalidProject(String),

    #[error("Scan cancelled")]
    Cancelled,
}

impl From<crate::scanner::parser::ParserError> for ScanError {
    fn from(e: crate::scanner::parser::ParserError) -> Self {
        ScanError::Parser(e.to_string())
    }
}

impl From<crate::config::ConfigError> for ScanError {
    fn from(e: crate::config::ConfigError) -> Self {
        ScanError::Configuration(e.to_string())
    }
}
