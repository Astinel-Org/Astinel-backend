use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiagnosticSpan {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
}

impl DiagnosticSpan {
    pub fn new(file: impl Into<PathBuf>, line: usize, column: usize) -> Self {
        Self {
            file: file.into(),
            line,
            column,
        }
    }
}

impl std::fmt::Display for DiagnosticSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file.display(), self.line, self.column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction() {
        let span = DiagnosticSpan::new("src/lib.rs", 10, 5);
        assert_eq!(span.line, 10);
        assert_eq!(span.column, 5);
    }

    #[test]
    fn display_format() {
        let span = DiagnosticSpan::new("src/lib.rs", 42, 1);
        assert_eq!(span.to_string(), "src/lib.rs:42:1");
    }

    #[test]
    fn serde_roundtrip() {
        let span = DiagnosticSpan::new("src/contract.rs", 100, 8);
        let json = serde_json::to_string(&span).unwrap();
        let back: DiagnosticSpan = serde_json::from_str(&json).unwrap();
        assert_eq!(back, span);
    }

    #[test]
    fn equality() {
        let a = DiagnosticSpan::new("a.rs", 1, 1);
        let b = DiagnosticSpan::new("a.rs", 1, 1);
        let c = DiagnosticSpan::new("a.rs", 1, 2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
