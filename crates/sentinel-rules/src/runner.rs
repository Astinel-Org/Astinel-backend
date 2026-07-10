use std::time::Instant;

use sentinel_core::{Ast, RuleRegistry};

use crate::config::RuleConfig;
use crate::filter::RuleFilter;
use crate::result::RuleResult;
use crate::suppression::SuppressionEngine;

pub struct RuleRunner {
    registry: RuleRegistry,
    config: RuleConfig,
    suppression: SuppressionEngine,
}

impl RuleRunner {
    pub fn new(registry: RuleRegistry, config: RuleConfig) -> Self {
        Self {
            registry,
            config,
            suppression: SuppressionEngine::new(),
        }
    }

    pub fn new_with_suppression(registry: RuleRegistry, config: RuleConfig, suppression: SuppressionEngine) -> Self {
        Self {
            registry,
            config,
            suppression,
        }
    }

    pub fn with_suppression_engine(mut self, suppression: SuppressionEngine) -> Self {
        self.suppression = suppression;
        self
    }

    pub fn run(&self, project: &dyn Ast) -> RuleResult {
        let start = Instant::now();
        let filter = RuleFilter::new(&self.config, &self.registry);
        let rules = filter.apply();
        let total_rules = rules.len();
        let total_files = project.files().len();
        let mut all_findings: Vec<Finding> = Vec::new();

        for rule in &rules {
            all_findings.extend(rule.check(project));
        }

        let mut suppressed_count = 0;
        let mut findings: Vec<_> = all_findings
            .into_iter()
            .filter(|f| {
                if self.suppression.is_suppressed(f) {
                    suppressed_count += 1;
                    false
                } else {
                    true
                }
            })
            .collect();

        findings.sort();

        RuleResult::from_findings(findings, suppressed_count, total_files, total_rules, start.elapsed())
    }
}

use sentinel_core::Finding;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{TestProject, TestRuleBuilder};
    use sentinel_core::{RuleId, RuleRegistry, Severity};

    #[test]
    fn runner_empty_project() {
        let mut registry = RuleRegistry::new();
        registry
            .register(Box::new(
                TestRuleBuilder::new("empty-rule").severity(Severity::Medium).build(),
            ))
            .unwrap();

        let config = RuleConfig::default();
        let runner = RuleRunner::new(registry, config);
        let result = runner.run(&TestProject::empty());
        assert_eq!(result.summary.total_rules_run, 1);
        assert_eq!(result.summary.total_findings, 0);
    }

    #[test]
    fn runner_detects_findings() {
        let mut registry = RuleRegistry::new();
        registry
            .register(Box::new(
                TestRuleBuilder::new("finding-rule")
                    .severity(Severity::High)
                    .with_finding(true)
                    .build(),
            ))
            .unwrap();

        let config = RuleConfig::default();
        let runner = RuleRunner::new(registry, config);
        let result = runner.run(&TestProject::empty());
        assert_eq!(result.summary.total_findings, 1);
    }

    #[test]
    fn runner_suppresses_findings() {
        let mut registry = RuleRegistry::new();
        registry
            .register(Box::new(
                TestRuleBuilder::new("finding-rule")
                    .severity(Severity::High)
                    .with_finding(true)
                    .build(),
            ))
            .unwrap();

        let mut suppression = SuppressionEngine::new();
        suppression.add_file_suppression(
            std::path::PathBuf::from("test.rs"),
            RuleId::new("finding-rule").unwrap(),
        );

        let config = RuleConfig::default();
        let runner = RuleRunner::new_with_suppression(registry, config, suppression);
        let result = runner.run(&TestProject::with_file("test.rs"));
        assert_eq!(result.summary.total_findings, 0);
        assert_eq!(result.summary.suppressed_findings, 1);
    }

    #[test]
    fn runner_filters_by_severity() {
        let mut registry = RuleRegistry::new();
        registry
            .register(Box::new(
                TestRuleBuilder::new("low-rule")
                    .severity(Severity::Low)
                    .with_finding(true)
                    .build(),
            ))
            .unwrap();
        registry
            .register(Box::new(
                TestRuleBuilder::new("high-rule")
                    .severity(Severity::High)
                    .with_finding(true)
                    .build(),
            ))
            .unwrap();

        let config = RuleConfig {
            severity_threshold: Severity::High,
            ..Default::default()
        };
        let runner = RuleRunner::new(registry, config);
        let result = runner.run(&TestProject::empty());
        assert_eq!(result.summary.total_rules_run, 1);
        assert_eq!(result.summary.total_findings, 1);
        assert_eq!(result.findings[0].rule_id.as_str(), "high-rule");
    }
}
