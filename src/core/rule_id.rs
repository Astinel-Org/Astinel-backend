use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RuleId(String);

impl RuleId {
    pub fn new(id: impl Into<String>) -> Result<Self, CoreError> {
        let id = id.into();
        if id.is_empty() {
            return Err(CoreError::InvalidRuleId(
                "rule id cannot be empty".to_string(),
            ));
        }
        if !id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
        {
            return Err(CoreError::InvalidRuleId(format!(
                "'{id}' is not a valid kebab-case rule id"
            )));
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for RuleId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<str> for RuleId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

use crate::core::error::CoreError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_kebab_case() {
        let id = RuleId::new("missing-require-auth").unwrap();
        assert_eq!(id.as_str(), "missing-require-auth");
    }

    #[test]
    fn valid_with_digits() {
        let id = RuleId::new("rule-001").unwrap();
        assert_eq!(id.as_str(), "rule-001");
    }

    #[test]
    fn empty_id() {
        assert!(RuleId::new("").is_err());
    }

    #[test]
    fn uppercase_rejected() {
        assert!(RuleId::new("MISSING-AUTH").is_err());
    }

    #[test]
    fn spaces_rejected() {
        assert!(RuleId::new("missing auth").is_err());
    }

    #[test]
    fn underscores_rejected() {
        assert!(RuleId::new("missing_auth").is_err());
    }

    #[test]
    fn equality() {
        let a = RuleId::new("test-rule").unwrap();
        let b = RuleId::new("test-rule").unwrap();
        let c = RuleId::new("other-rule").unwrap();
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn hash_map_key() {
        use std::collections::HashMap;
        let mut map: HashMap<RuleId, i32> = HashMap::new();
        map.insert(RuleId::new("test").unwrap(), 42);
        assert_eq!(map.get(&RuleId::new("test").unwrap()), Some(&42));
    }

    #[test]
    fn display() {
        let id = RuleId::new("my-rule").unwrap();
        assert_eq!(id.to_string(), "my-rule");
    }
}
