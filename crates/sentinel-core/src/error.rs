use crate::rule_id::RuleId;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoreError {
    #[error("rule with id `{0}` is already registered")]
    DuplicateRule(RuleId),

    #[error("invalid rule id: {0}")]
    InvalidRuleId(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_rule_display() {
        let id = RuleId::new("test-rule").unwrap();
        let err = CoreError::DuplicateRule(id);
        assert_eq!(err.to_string(), "rule with id `test-rule` is already registered");
    }

    #[test]
    fn invalid_rule_id_display() {
        let err = CoreError::InvalidRuleId("bad id".to_string());
        assert_eq!(err.to_string(), "invalid rule id: bad id");
    }

    #[test]
    fn clone_and_eq() {
        let err = CoreError::InvalidRuleId("x".to_string());
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }
}
