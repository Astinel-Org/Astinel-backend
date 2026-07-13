use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::High => "high",
            Severity::Medium => "medium",
            Severity::Low => "low",
            Severity::Info => "info",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "critical" => Some(Self::Critical),
            "high" => Some(Self::High),
            "medium" => Some(Self::Medium),
            "low" => Some(Self::Low),
            "info" => Some(Self::Info),
            _ => None,
        }
    }

    pub fn weight(self) -> u8 {
        match self {
            Severity::Critical => 25,
            Severity::High => 10,
            Severity::Medium => 5,
            Severity::Low => 2,
            Severity::Info => 0,
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering() {
        assert!(Severity::Info > Severity::Low);
        assert!(Severity::Low > Severity::Medium);
        assert!(Severity::Medium > Severity::High);
        assert!(Severity::High > Severity::Critical);
    }

    #[test]
    fn severity_sorting_ascending() {
        let mut sevs = vec![
            Severity::Low,
            Severity::Critical,
            Severity::Info,
            Severity::High,
        ];
        sevs.sort();
        assert_eq!(
            sevs,
            vec![
                Severity::Critical,
                Severity::High,
                Severity::Low,
                Severity::Info
            ]
        );
    }

    #[test]
    fn severity_sort_descending_for_findings() {
        let mut sevs = vec![
            Severity::Low,
            Severity::Critical,
            Severity::Info,
            Severity::High,
        ];
        sevs.sort_by(|a, b| b.cmp(a));
        assert_eq!(
            sevs,
            vec![
                Severity::Info,
                Severity::Low,
                Severity::High,
                Severity::Critical
            ]
        );
    }

    #[test]
    fn severity_from_str() {
        assert_eq!(Severity::parse("critical"), Some(Severity::Critical));
        assert_eq!(Severity::parse("HIGH"), Some(Severity::High));
        assert_eq!(Severity::parse("Medium"), Some(Severity::Medium));
        assert_eq!(Severity::parse("unknown"), None);
    }

    #[test]
    fn severity_display() {
        assert_eq!(Severity::Critical.to_string(), "critical");
        assert_eq!(Severity::Info.to_string(), "info");
    }

    #[test]
    fn severity_weights() {
        assert_eq!(Severity::Critical.weight(), 25);
        assert_eq!(Severity::High.weight(), 10);
        assert_eq!(Severity::Medium.weight(), 5);
        assert_eq!(Severity::Low.weight(), 2);
        assert_eq!(Severity::Info.weight(), 0);
    }
}
