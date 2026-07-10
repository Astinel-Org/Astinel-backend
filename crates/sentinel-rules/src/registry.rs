use sentinel_core::{Rule, RuleRegistry};

pub trait RuleRegistryExt {
    fn register_builtins(self) -> Self;
    fn with_rule(self, rule: Box<dyn Rule>) -> Self;
}

impl RuleRegistryExt for RuleRegistry {
    fn register_builtins(mut self) -> Self {
        let rules = crate::builtin::register_all();
        for rule in rules {
            let _ = self.register(rule);
        }
        self
    }

    fn with_rule(mut self, rule: Box<dyn Rule>) -> Self {
        let _ = self.register(rule);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TestRuleBuilder;

    #[test]
    fn register_builtins_returns_non_empty() {
        let registry = RuleRegistry::new().register_builtins();
        assert!(!registry.is_empty());
        assert!(registry.iter().count() >= 10);
    }

    #[test]
    fn with_rule_adds_one() {
        let registry = RuleRegistry::new()
            .register_builtins()
            .with_rule(Box::new(TestRuleBuilder::new("custom-rule").build()));
        assert!(registry.iter().any(|r| r.id().as_str() == "custom-rule"));
    }
}
