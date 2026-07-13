mod config_impl;
mod error;

pub use config_impl::{
    build_run_config, discover_config, is_valid_rule_id, load_config, CategoryFilter, CliOverrides,
    ConfigSource, OutputColor, OutputFormat, RunConfig,
};
pub use error::ConfigError;
