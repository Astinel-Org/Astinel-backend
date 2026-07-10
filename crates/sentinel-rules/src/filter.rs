use sentinel_core::{Rule, RuleRegistry};

use crate::config::RuleConfig;

pub(crate) struct RuleFilter<'a> {
    registry: &'a RuleRegistry,
    config: &'a RuleConfig,
}

impl<'a> RuleFilter<'a> {
    pub fn new(config: &'a RuleConfig, registry: &'a RuleRegistry) -> Self {
        Self { registry, config }
    }

    pub fn apply(&self) -> Vec<&'a dyn Rule> {
        let threshold = self.config.severity_threshold;
        let mut rules: Vec<&'a dyn Rule> = self
            .registry
            .iter()
            .filter(|r| {
                let id = r.id();
                let effective_severity = self
                    .config
                    .severity_overrides
                    .get(&id)
                    .copied()
                    .unwrap_or_else(|| r.severity());

                let is_enabled = {
                    if self.config.enabled.is_empty() {
                        true
                    } else {
                        self.config.enabled.contains(&id)
                    }
                };

                let is_not_disabled = if !self.config.enabled.is_empty() {
                    true // enabled list overrides disabled
                } else {
                    !self.config.disabled.contains(&id)
                };
                let meets_threshold = effective_severity <= threshold;

                is_enabled && is_not_disabled && meets_threshold
            })
            .collect();

        rules.sort_by_key(|r| r.id().clone());
        rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TestRuleBuilder;
    use sentinel_core::{RuleRegistry, Severity};

    fn registry_with(rules: Vec<Box<dyn Rule>>) -> RuleRegistry {
        let mut reg = RuleRegistry::new();
        for r in rules {
            reg.register(r).unwrap();
        }
        reg
    }

    #[test]
    fn all_rules_enabled_by_default() {
        let reg = registry_with(vec![
            Box::new(TestRuleBuilder::new("rule-a").severity(Severity::High).build()),
            Box::new(TestRuleBuilder::new("rule-b").severity(Severity::Low).build()),
        ]);
        let config = RuleConfig::default();
        let filter = RuleFilter::new(&config, &reg);
        let rules = filter.apply();
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn disabled_rule_excluded() {
        let mut config = RuleConfig::default();
        config.disabled.push(sentinel_core::RuleId::new("rule-b").unwrap());

        let reg = registry_with(vec![
            Box::new(TestRuleBuilder::new("rule-a").severity(Severity::High).build()),
            Box::new(TestRuleBuilder::new("rule-b").severity(Severity::Low).build()),
        ]);
        let filter = RuleFilter::new(&config, &reg);
        let rules = filter.apply();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id().as_str(), "rule-a");
    }

    #[test]
    fn severity_threshold_excludes_low() {
        let config = RuleConfig {
            severity_threshold: Severity::Medium,
            ..Default::default()
        };

        let reg = registry_with(vec![
            Box::new(TestRuleBuilder::new("high-rule").severity(Severity::High).build()),
            Box::new(TestRuleBuilder::new("low-rule").severity(Severity::Low).build()),
        ]);
        let filter = RuleFilter::new(&config, &reg);
        let rules = filter.apply();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id().as_str(), "high-rule");
    }

    #[test]
    fn deterministic_order_by_id() {
        let reg = registry_with(vec![
            Box::new(TestRuleBuilder::new("z-rule").severity(Severity::High).build()),
            Box::new(TestRuleBuilder::new("a-rule").severity(Severity::High).build()),
            Box::new(TestRuleBuilder::new("m-rule").severity(Severity::High).build()),
        ]);
        let config = RuleConfig::default();
        let filter = RuleFilter::new(&config, &reg);
        let rules = filter.apply();
        let ids: Vec<String> = rules.iter().map(|r| r.id().to_string()).collect();
        assert_eq!(ids, vec!["a-rule", "m-rule", "z-rule"]);
    }

    #[test]
    fn enabled_list_overrides_disabled() {
        let mut config = RuleConfig::default();
        config.enabled.push(sentinel_core::RuleId::new("rule-b").unwrap());
        config.disabled.push(sentinel_core::RuleId::new("rule-b").unwrap());

        let reg = registry_with(vec![
            Box::new(TestRuleBuilder::new("rule-a").severity(Severity::High).build()),
            Box::new(TestRuleBuilder::new("rule-b").severity(Severity::High).build()),
        ]);
        let filter = RuleFilter::new(&config, &reg);
        let rules = filter.apply();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id().as_str(), "rule-b");
    }

    #[test]
    fn empty_registry_returns_no_rules() {
        let reg = RuleRegistry::new();
        let config = RuleConfig::default();
        let filter = RuleFilter::new(&config, &reg);
        assert!(filter.apply().is_empty());
    }
}
