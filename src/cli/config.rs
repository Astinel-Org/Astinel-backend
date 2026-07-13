use std::path::Path;

use crate::config::{CliOverrides, RunConfig};

/// Raw CLI arguments, parsed by clap.
#[derive(Debug, Clone)]
pub struct CliArgs {
    pub path: String,
    pub json: bool,
    pub compact: bool,
    pub quiet: bool,
    pub verbose: bool,
    pub color: String,
    pub fail_on: Option<String>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub rule: Option<String>,
    pub exclude: Option<String>,
    pub timings: bool,
    pub score: bool,
    pub threads: Option<usize>,
    pub config_path: Option<String>,
}

impl CliArgs {
    pub fn merge_into_run_config(&self) -> Result<RunConfig, crate::cli::errors::CliError> {
        let cli = CliOverrides {
            path: Some(self.path.clone()),
            json: Some(self.json),
            compact: Some(self.compact),
            quiet: Some(self.quiet),
            verbose: Some(self.verbose),
            color: Some(self.color.clone()),
            fail_on: self.fail_on.clone(),
            severity: self.severity.clone(),
            category: self.category.clone(),
            rule: self.rule.as_ref().map(|r| {
                r.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            }),
            exclude: self.exclude.as_ref().map(|e| {
                e.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            }),
            timings: Some(self.timings),
            score: Some(self.score),
            threads: self.threads,
            config_path: self.config_path.clone(),
        };

        crate::config::load_config(Path::new(&self.path), &cli)
            .map_err(|e| crate::cli::errors::CliError::Config(e.to_string()))
    }
}
