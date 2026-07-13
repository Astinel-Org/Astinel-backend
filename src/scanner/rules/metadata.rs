use crate::core::{Category, RuleId, Severity};

#[derive(Debug, Clone)]
pub struct RuleMetadata {
    pub id: RuleId,
    pub name: String,
    pub short_description: String,
    pub description: String,
    pub severity: Severity,
    pub category: Category,
    pub documentation_url: String,
    pub cwe_id: Option<String>,
    pub since_version: String,
}

pub trait RuleMetaProvider {
    fn metadata(&self) -> RuleMetadata;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_constructs() {
        let meta = RuleMetadata {
            id: RuleId::new("test-rule").unwrap(),
            name: "Test Rule".to_string(),
            short_description: "A test".to_string(),
            description: "A longer test description".to_string(),
            severity: Severity::High,
            category: Category::Security,
            documentation_url: "https://docs.sentinel.dev/rules/test".to_string(),
            cwe_id: Some("CWE-123".to_string()),
            since_version: "0.1.0".to_string(),
        };

        assert_eq!(meta.id.as_str(), "test-rule");
        assert_eq!(meta.severity, Severity::High);
    }
}
