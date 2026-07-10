pub mod builtin;
pub mod config;
pub mod diagnostic;
pub mod filter;
pub mod metadata;
pub mod registry;
pub mod result;
pub mod runner;
pub mod suppression;

#[cfg(test)]
pub(crate) mod testing;

use sentinel_core::RuleRegistry;

use crate::config::RuleConfig;
use crate::result::RuleResult;
use crate::runner::RuleRunner;
use crate::suppression::SuppressionEngine;

pub struct RuleEngine {
    runner: RuleRunner,
}

impl RuleEngine {
    pub fn new(registry: RuleRegistry, config: RuleConfig) -> Self {
        Self {
            runner: RuleRunner::new(registry, config),
        }
    }

    pub fn new_with_suppression(registry: RuleRegistry, config: RuleConfig, suppression: SuppressionEngine) -> Self {
        Self {
            runner: RuleRunner::new_with_suppression(registry, config, suppression),
        }
    }

    pub fn run(&self, project: &dyn sentinel_core::Ast) -> RuleResult {
        self.runner.run(project)
    }
}
