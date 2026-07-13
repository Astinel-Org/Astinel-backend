use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Category {
    Security,
    Performance,
    Gas,
    BestPractice,
    Upgrade,
}

impl Category {
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Security => "security",
            Category::Performance => "performance",
            Category::Gas => "gas",
            Category::BestPractice => "best-practice",
            Category::Upgrade => "upgrade",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "security" => Some(Self::Security),
            "performance" => Some(Self::Performance),
            "gas" => Some(Self::Gas),
            "best-practice" | "bestpractice" => Some(Self::BestPractice),
            "upgrade" => Some(Self::Upgrade),
            _ => None,
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_display() {
        assert_eq!(Category::Security.to_string(), "security");
        assert_eq!(Category::BestPractice.to_string(), "best-practice");
    }

    #[test]
    fn category_from_str() {
        assert_eq!(Category::parse("security"), Some(Category::Security));
        assert_eq!(
            Category::parse("best-practice"),
            Some(Category::BestPractice)
        );
        assert_eq!(Category::parse("unknown"), None);
    }

    #[test]
    fn category_serde_roundtrip() {
        let json = serde_json::to_string(&Category::Security).unwrap();
        let back: Category = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Category::Security);
    }
}
