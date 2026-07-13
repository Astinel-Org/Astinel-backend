use std::collections::HashMap;

use crate::core::rule_id::RuleId;
use crate::core::severity::Severity;

#[derive(Debug, Clone)]
pub struct RuleConfig {
    pub severity_threshold: Severity,
    pub enabled: Vec<RuleId>,
    pub disabled: Vec<RuleId>,
    pub severity_overrides: HashMap<RuleId, Severity>,
    pub ignore_paths: Vec<String>,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            severity_threshold: Severity::Low,
            enabled: Vec::new(),
            disabled: Vec::new(),
            severity_overrides: HashMap::new(),
            ignore_paths: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_no_filters() {
        let config = RuleConfig::default();
        assert_eq!(config.severity_threshold, Severity::Low);
        assert!(config.enabled.is_empty());
        assert!(config.disabled.is_empty());
        assert!(config.severity_overrides.is_empty());
    }

    #[test]
    fn config_with_disabled_rules() {
        let mut config = RuleConfig::default();
        config.disabled.push(RuleId::new("some-rule").unwrap());
        assert_eq!(config.disabled.len(), 1);
    }

    #[test]
    fn config_with_severity_override() {
        let mut config = RuleConfig::default();
        config
            .severity_overrides
            .insert(RuleId::new("some-rule").unwrap(), Severity::Critical);
        assert_eq!(
            config
                .severity_overrides
                .get(&RuleId::new("some-rule").unwrap()),
            Some(&Severity::Critical),
        );
    }
}
