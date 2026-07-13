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

use std::time::Instant;

use crate::core::Ast;
use crate::core::RuleConfig;
use crate::core::RuleRegistry;
use crate::scanner::rules::filter::RuleFilter;
use crate::scanner::rules::result::RuleResult;
use crate::scanner::rules::suppression::SuppressionEngine;

pub struct RuleEngine {
    registry: RuleRegistry,
    config: RuleConfig,
    suppression: Option<SuppressionEngine>,
}

impl RuleEngine {
    pub fn new(registry: RuleRegistry, config: RuleConfig) -> Self {
        Self {
            registry,
            config,
            suppression: None,
        }
    }

    pub fn new_with_suppression(
        registry: RuleRegistry,
        config: RuleConfig,
        suppression: SuppressionEngine,
    ) -> Self {
        Self {
            registry,
            config,
            suppression: Some(suppression),
        }
    }

    pub fn run(&self, project: &dyn Ast) -> RuleResult {
        let start = Instant::now();
        let filter = RuleFilter::new(&self.config, &self.registry);
        let rules = filter.apply();
        let total_rules = rules.len();
        let total_files = project.files().len();
        let mut all_findings: Vec<crate::core::Finding> = Vec::new();

        for rule in &rules {
            all_findings.extend(rule.check(project));
        }

        let mut suppressed_count = 0;
        let findings: Vec<_> = all_findings
            .into_iter()
            .filter(|f| {
                if let Some(ref suppression) = self.suppression {
                    if suppression.is_suppressed(f) {
                        suppressed_count += 1;
                        return false;
                    }
                }
                true
            })
            .collect();

        RuleResult::from_findings(
            findings,
            suppressed_count,
            total_files,
            total_rules,
            start.elapsed(),
        )
    }
}
