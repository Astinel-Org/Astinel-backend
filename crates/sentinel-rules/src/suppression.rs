use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sentinel_core::{Finding, RuleId};

#[derive(Debug, Clone)]
pub struct SuppressionEngine {
    line_suppressions: HashMap<PathBuf, Vec<(usize, Vec<RuleId>)>>,
    file_suppressions: HashMap<PathBuf, Vec<RuleId>>,
}

impl SuppressionEngine {
    pub fn new() -> Self {
        Self {
            line_suppressions: HashMap::new(),
            file_suppressions: HashMap::new(),
        }
    }

    pub fn from_source_files(files: &[&Path]) -> Self {
        let mut engine = Self::new();
        for file in files {
            if let Ok(content) = std::fs::read_to_string(file) {
                engine.parse_suppressions(file, &content);
            }
        }
        engine
    }

    fn parse_suppressions(&mut self, path: &Path, content: &str) {
        let mut line_supps = Vec::new();
        let mut file_supps = Vec::new();

        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("// sentinel-ignore-file[") {
                if let Some(ids) = Self::extract_rule_ids(trimmed, "// sentinel-ignore-file[") {
                    file_supps.extend(ids);
                }
            } else if trimmed.starts_with("// sentinel-ignore-file") {
                file_supps.push(RuleId::new("all").unwrap());
            } else if trimmed.starts_with("// sentinel-ignore[") {
                if let Some(ids) = Self::extract_rule_ids(trimmed, "// sentinel-ignore[") {
                    line_supps.push((line_num, ids));
                }
            } else if trimmed.starts_with("// sentinel-ignore") {
                line_supps.push((line_num, vec![RuleId::new("all").unwrap()]));
            }
        }

        self.line_suppressions.insert(path.to_path_buf(), line_supps);
        self.file_suppressions.insert(path.to_path_buf(), file_supps);
    }

    fn extract_rule_ids(line: &str, prefix: &str) -> Option<Vec<RuleId>> {
        let rest = line.strip_prefix(prefix)?;
        let content = rest.strip_suffix(']')?;
        Some(
            content
                .split(',')
                .filter_map(|s| {
                    let trimmed = s.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        RuleId::new(trimmed).ok()
                    }
                })
                .collect(),
        )
    }

    pub fn is_suppressed(&self, finding: &Finding) -> bool {
        let path = &finding.span.file;

        // Check file-level suppression
        if let Some(suppressed_ids) = self.file_suppressions.get(path) {
            if suppressed_ids
                .iter()
                .any(|id| *id == finding.rule_id || id.as_str() == "all")
            {
                return true;
            }
        }

        // Check line-level suppression (previous line)
        if let Some(line_supps) = self.line_suppressions.get(path) {
            // Suppression comment on the line before the finding
            let suppress_line = if finding.span.line > 1 {
                finding.span.line - 1
            } else {
                return false;
            };

            for (sup_line, sup_ids) in line_supps {
                if *sup_line == suppress_line && sup_ids.iter().any(|id| *id == finding.rule_id || id.as_str() == "all")
                {
                    return true;
                }
            }
        }

        false
    }

    pub fn add_file_suppression(&mut self, path: PathBuf, rule_id: RuleId) {
        self.file_suppressions.entry(path).or_default().push(rule_id);
    }

    pub fn add_line_suppression(&mut self, path: PathBuf, line: usize, rule_id: RuleId) {
        self.line_suppressions
            .entry(path)
            .or_default()
            .push((line, vec![rule_id]));
    }
}

impl Default for SuppressionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sentinel_core::{Category, DiagnosticSpan, RuleId, Severity};

    fn finding(rule: &str, file: &str, line: usize, col: usize) -> Finding {
        Finding::new(
            RuleId::new(rule).unwrap(),
            Severity::High,
            Category::Security,
            DiagnosticSpan::new(file, line, col),
            "test",
            "fix",
        )
    }

    #[test]
    fn no_suppression_no_match() {
        let engine = SuppressionEngine::new();
        let f = finding("test-rule", "f.rs", 10, 1);
        assert!(!engine.is_suppressed(&f));
    }

    #[test]
    fn inline_suppression_specific_rule() {
        let mut engine = SuppressionEngine::new();
        engine.add_line_suppression(PathBuf::from("f.rs"), 9, RuleId::new("test-rule").unwrap());
        let f = finding("test-rule", "f.rs", 10, 1);
        assert!(engine.is_suppressed(&f));
    }

    #[test]
    fn inline_suppression_wrong_line() {
        let mut engine = SuppressionEngine::new();
        engine.add_line_suppression(PathBuf::from("f.rs"), 5, RuleId::new("test-rule").unwrap());
        let f = finding("test-rule", "f.rs", 10, 1);
        assert!(!engine.is_suppressed(&f));
    }

    #[test]
    fn inline_suppression_wrong_rule() {
        let mut engine = SuppressionEngine::new();
        engine.add_line_suppression(PathBuf::from("f.rs"), 9, RuleId::new("other-rule").unwrap());
        let f = finding("test-rule", "f.rs", 10, 1);
        assert!(!engine.is_suppressed(&f));
    }

    #[test]
    fn file_level_suppression() {
        let mut engine = SuppressionEngine::new();
        engine.add_file_suppression(PathBuf::from("f.rs"), RuleId::new("test-rule").unwrap());
        let f = finding("test-rule", "f.rs", 100, 1);
        assert!(engine.is_suppressed(&f));
    }

    #[test]
    fn parse_inline_comment() {
        let dir = std::env::temp_dir().join("sentinel_supp_test");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("test.rs");
        std::fs::write(&file, "// sentinel-ignore[test-rule]\nfn foo() {}\n").unwrap();

        let engine = SuppressionEngine::from_source_files(&[&file]);
        let f = finding("test-rule", file.to_str().unwrap(), 2, 1);
        assert!(engine.is_suppressed(&f));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_file_level_comment() {
        let dir = std::env::temp_dir().join("sentinel_supp_test2");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("test.rs");
        std::fs::write(&file, "// sentinel-ignore-file[test-rule]\nfn foo() {}\nfn bar() {}\n").unwrap();

        let engine = SuppressionEngine::from_source_files(&[&file]);
        let f1 = finding("test-rule", file.to_str().unwrap(), 2, 1);
        let f2 = finding("test-rule", file.to_str().unwrap(), 3, 1);
        assert!(engine.is_suppressed(&f1));
        assert!(engine.is_suppressed(&f2));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_ignore_all() {
        let dir = std::env::temp_dir().join("sentinel_supp_test3");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("test.rs");
        std::fs::write(&file, "// sentinel-ignore\nfn foo() {}\n").unwrap();

        let engine = SuppressionEngine::from_source_files(&[&file]);
        let f = finding("any-rule", file.to_str().unwrap(), 2, 1);
        assert!(engine.is_suppressed(&f));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
