use indexmap::IndexMap;

use crate::core::error::CoreError;
use crate::core::rule::Rule;
use crate::core::rule_id::RuleId;

#[derive(Debug, Clone)]
pub struct RuleRegistry {
    rules: IndexMap<RuleId, Box<dyn Rule>>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        Self {
            rules: IndexMap::new(),
        }
    }

    pub fn register(&mut self, rule: Box<dyn Rule>) -> Result<(), CoreError> {
        let id = rule.id();
        if self.rules.contains_key(&id) {
            return Err(CoreError::DuplicateRule(id));
        }
        self.rules.insert(id, rule);
        Ok(())
    }

    pub fn get(&self, id: &RuleId) -> Option<&dyn Rule> {
        self.rules.get(id).map(|b| b.as_ref())
    }

    pub fn contains(&self, id: &RuleId) -> bool {
        self.rules.contains_key(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn Rule> {
        self.rules.values().map(|b| b.as_ref())
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ast::Ast;
    use crate::core::category::Category;
    use crate::core::finding::Finding;
    use crate::core::severity::Severity;

    macro_rules! make_rule {
        ($id:expr, $name:expr, $severity:ident) => {{
            #[derive(Debug, Clone)]
            struct R;
            impl Rule for R {
                fn id(&self) -> RuleId {
                    RuleId::new($id).unwrap()
                }
                fn name(&self) -> &'static str {
                    $name
                }
                fn description(&self) -> &'static str {
                    "desc"
                }
                fn severity(&self) -> Severity {
                    Severity::$severity
                }
                fn category(&self) -> Category {
                    Category::Security
                }
                fn check(&self, _: &dyn Ast) -> Vec<Finding> {
                    vec![]
                }
                fn clone_box(&self) -> Box<dyn Rule> {
                    Box::new(self.clone())
                }
            }
            Box::new(R) as Box<dyn Rule>
        }};
    }

    #[test]
    fn register_and_retrieve() {
        let mut reg = RuleRegistry::new();
        let rule = make_rule!("test-rule", "Test", High);
        let id = rule.id();
        reg.register(rule).unwrap();
        assert!(reg.contains(&id));
        assert!(reg.get(&id).is_some());
    }

    #[test]
    fn duplicate_registration_fails() {
        let mut reg = RuleRegistry::new();
        reg.register(make_rule!("dup", "Dup", High)).unwrap();
        let result = reg.register(make_rule!("dup", "Dup2", Critical));
        assert!(result.is_err());
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn get_none_for_unknown() {
        let reg = RuleRegistry::new();
        assert!(reg.get(&RuleId::new("unknown").unwrap()).is_none());
    }

    #[test]
    fn iter_all_rules() {
        let mut reg = RuleRegistry::new();
        reg.register(make_rule!("a", "A", Low)).unwrap();
        reg.register(make_rule!("b", "B", High)).unwrap();
        let ids: Vec<String> = reg.iter().map(|r| r.id().to_string()).collect();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn empty_registry() {
        let reg = RuleRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn len_after_registration() {
        let mut reg = RuleRegistry::new();
        assert_eq!(reg.len(), 0);
        reg.register(make_rule!("r1", "R1", High)).unwrap();
        assert_eq!(reg.len(), 1);
        reg.register(make_rule!("r2", "R2", Medium)).unwrap();
        assert_eq!(reg.len(), 2);
    }
}
