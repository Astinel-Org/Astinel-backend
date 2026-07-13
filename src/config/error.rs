use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    ConfigurationNotFound(PathBuf),

    #[error("Failed to read configuration file {path}: {source}")]
    IoError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Invalid TOML syntax in {path} at line {line} column {col}: {message}")]
    InvalidTomlSyntax {
        path: PathBuf,
        line: usize,
        col: usize,
        message: String,
    },

    #[error("Invalid TOML in {path}: {message}")]
    InvalidToml { path: PathBuf, message: String },

    #[error("Unknown configuration section '{section}' in {path} at line {line}")]
    UnknownSection {
        path: PathBuf,
        section: String,
        line: usize,
    },

    #[error("Unknown key '{key}' in section [{section}] at line {line} in {path}")]
    UnknownKey {
        path: PathBuf,
        section: String,
        key: String,
        line: usize,
    },

    #[error(
        "Invalid value for '{key}' in section [{section}] at line {line} in {path}: {message}"
    )]
    InvalidValue {
        path: PathBuf,
        section: String,
        key: String,
        line: usize,
        message: String,
    },

    #[error("Duplicate rule '{rule}' in rules section: occurs at enable and disable lists")]
    DuplicateRule { rule: String, path: PathBuf },

    #[error("Invalid rule ID '{rule}' in {path}: rule IDs must be non-empty, lowercase alphanumeric with hyphens")]
    InvalidRuleId {
        rule: String,
        path: PathBuf,
        line: usize,
    },

    #[error("Invalid glob pattern '{pattern}' in {path}: {message}")]
    InvalidGlob {
        pattern: String,
        path: PathBuf,
        line: usize,
        message: String,
    },

    #[error("Invalid severity value '{value}' in {path}: must be one of info, low, medium, high, critical")]
    InvalidSeverity {
        value: String,
        path: PathBuf,
        line: usize,
    },

    #[error("Invalid output format '{value}' in {path}: must be one of pretty, compact, json")]
    InvalidOutputFormat {
        value: String,
        path: PathBuf,
        line: usize,
    },

    #[error("Invalid color value '{value}' in {path}: must be one of auto, always, never")]
    InvalidColor {
        value: String,
        path: PathBuf,
        line: usize,
    },

    #[error("Negative value for 'threads' ({value}) in {path}")]
    NegativeThreads {
        value: i64,
        path: PathBuf,
        line: usize,
    },

    #[error("Conflicting configuration: {message}")]
    Conflicting { message: String },

    #[error("Configuration section '{section}' not found in {path}")]
    SectionNotFound { section: String, path: PathBuf },

    #[error("Cannot read current directory: {0}")]
    CurrentDir(std::io::Error),
}
