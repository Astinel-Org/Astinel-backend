use std::collections::BTreeMap;

use serde::Serialize;

use crate::scanner::report::formatter::*;

/// SARIF 2.1.0 formatter compatible with GitHub Code Scanning.
pub struct SarifFormatter;

// ---------------------------------------------------------------------------
// SARIF data model
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct SarifLog {
    #[serde(rename = "$schema")]
    schema: String,
    version: String,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
struct SarifRun {
    tool: SarifTool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    artifacts: Vec<SarifArtifact>,
    results: Vec<SarifResult>,
    column_kind: String,
    properties: SarifRunProperties,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
struct SarifDriver {
    name: String,
    full_name: String,
    version: String,
    semantic_version: String,
    information_uri: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    rules: Vec<SarifRule>,
}

#[derive(Serialize)]
struct SarifRule {
    id: String,
    name: Option<String>,
    short_description: Option<SarifMessage>,
    full_description: Option<SarifMessage>,
    default_configuration: Option<SarifRuleConfiguration>,
    properties: Option<SarifRuleProperties>,
}

#[derive(Serialize)]
struct SarifRuleConfiguration {
    level: String,
}

#[derive(Serialize)]
struct SarifRuleProperties {
    category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    precision: Option<String>,
}

#[derive(Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Serialize)]
struct SarifArtifact {
    location: SarifArtifactLocation,
    length: i64,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
struct SarifResult {
    rule_id: String,
    rule_index: usize,
    level: String,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
    properties: SarifResultProperties,
}

#[derive(Serialize)]
struct SarifLocation {
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
struct SarifPhysicalLocation {
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Serialize)]
struct SarifRegion {
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
}

#[derive(Serialize)]
struct SarifResultProperties {
    severity: String,
    recommendation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fix_example: Option<String>,
}

#[derive(Serialize)]
struct SarifRunProperties {
    score: u8,
    total_files: usize,
    total_rules: usize,
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

impl OutputFormatter for SarifFormatter {
    fn format(&self, report: &Report) -> String {
        let sorted = report.sorted_findings();

        // Build unique file set
        let mut file_map: BTreeMap<String, usize> = BTreeMap::new();
        for f in &sorted {
            let path = f.span.file.to_string_lossy().to_string();
            let entry = file_map.entry(path).or_insert(0);
            *entry += 1;
        }

        // Build artifacts
        let artifacts: Vec<SarifArtifact> = file_map
            .keys()
            .map(|uri| SarifArtifact {
                location: SarifArtifactLocation { uri: uri.clone() },
                length: -1,
            })
            .collect();

        // Build unique rule set and index
        let mut rule_map: BTreeMap<String, usize> = BTreeMap::new();
        let mut rule_details: Vec<SarifRule> = Vec::new();
        for f in &sorted {
            let rid = f.rule_id.to_string();
            if !rule_map.contains_key(&rid) {
                let idx = rule_details.len();
                rule_map.insert(rid.clone(), idx);
                let level = sarif_level(f.severity);
                rule_details.push(SarifRule {
                    id: rid.clone(),
                    name: Some(rid.clone()),
                    short_description: Some(SarifMessage {
                        text: f.message.clone(),
                    }),
                    full_description: Some(SarifMessage {
                        text: f.recommendation.clone(),
                    }),
                    default_configuration: Some(SarifRuleConfiguration {
                        level: level.clone(),
                    }),
                    properties: Some(SarifRuleProperties {
                        category: f.category.as_str().to_string(),
                        precision: Some("very-high".to_string()),
                    }),
                });
            }
        }

        // Build results
        let results: Vec<SarifResult> = sorted
            .iter()
            .map(|f| {
                let rid = f.rule_id.to_string();
                let rule_index = rule_map[&rid];
                let level = sarif_level(f.severity);
                let uri = f.span.file.to_string_lossy().to_string();
                SarifResult {
                    rule_id: rid,
                    rule_index,
                    level,
                    message: SarifMessage {
                        text: f.message.clone(),
                    },
                    locations: vec![SarifLocation {
                        physical_location: SarifPhysicalLocation {
                            artifact_location: SarifArtifactLocation { uri },
                            region: SarifRegion {
                                start_line: f.span.line,
                                start_column: f.span.column,
                                end_line: f.span.line,
                                end_column: f.span.column + 1,
                            },
                        },
                    }],
                    properties: SarifResultProperties {
                        severity: f.severity.as_str().to_string(),
                        recommendation: f.recommendation.clone(),
                        fix_example: f.fix_example.clone(),
                    },
                }
            })
            .collect();

        let log = SarifLog {
            schema: "https://json.schemastore.org/sarif-2.1.0.json".to_string(),
            version: "2.1.0".to_string(),
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifDriver {
                        name: "Sentinel".to_string(),
                        full_name: "Sentinel Static Analysis".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                        semantic_version: env!("CARGO_PKG_VERSION").to_string(),
                        information_uri: "https://sentinel.dev".to_string(),
                        rules: rule_details,
                    },
                },
                artifacts,
                results,
                column_kind: "utf16CodeUnits".to_string(),
                properties: SarifRunProperties {
                    score: report.score.score,
                    total_files: report.summary.total_files,
                    total_rules: report.summary.total_rules,
                },
            }],
        };

        serde_json::to_string_pretty(&log).unwrap_or_else(|_| "{}".to_string())
    }
}

fn sarif_level(severity: crate::core::Severity) -> String {
    match severity {
        crate::core::Severity::Critical => "error".to_string(),
        crate::core::Severity::High => "error".to_string(),
        crate::core::Severity::Medium => "warning".to_string(),
        crate::core::Severity::Low => "note".to_string(),
        crate::core::Severity::Info => "note".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Category, DiagnosticSpan, RuleId, Severity};

    fn finding(
        severity: Severity,
        rule: &str,
        file: &str,
        line: usize,
        col: usize,
    ) -> crate::core::Finding {
        crate::core::Finding::new(
            RuleId::new(rule).unwrap(),
            severity,
            Category::Security,
            DiagnosticSpan::new(file, line, col),
            "test message",
            "test recommendation",
        )
    }

    fn caps() -> ReportOptions {
        ReportOptions {
            color: false,
            unicode: false,
            width: 80,
        }
    }

    fn summary(duration_ms: u64) -> ReportSummary {
        ReportSummary {
            project_name: "test-project".to_string(),
            total_files: 5,
            total_rules: 10,
            total_findings: 2,
            suppressed_findings: 0,
            duration: std::time::Duration::from_millis(duration_ms),
            parse_duration: std::time::Duration::from_millis(10),
            rule_duration: std::time::Duration::from_millis(40),
        }
    }

    fn report_data(findings: Vec<crate::core::Finding>, duration_ms: u64) -> Report {
        Report {
            score: crate::core::SecurityScore::from_findings(&findings),
            summary: summary(duration_ms),
            options: caps(),
            findings,
        }
    }

    #[test]
    fn sarif_valid_structure() {
        let formatter = SarifFormatter;
        let r = report_data(
            vec![finding(
                Severity::Critical,
                "critical-rule",
                "src/main.rs",
                42,
                5,
            )],
            100,
        );
        let output = formatter.format(&r);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(
            parsed["$schema"],
            "https://json.schemastore.org/sarif-2.1.0.json"
        );
        assert_eq!(parsed["version"], "2.1.0");
        assert!(parsed["runs"].is_array());
        assert_eq!(parsed["runs"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn sarif_has_tool_driver() {
        let formatter = SarifFormatter;
        let r = report_data(vec![], 0);
        let output = formatter.format(&r);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let driver = &parsed["runs"][0]["tool"]["driver"];
        assert_eq!(driver["name"], "Sentinel");
        assert_eq!(driver["full_name"], "Sentinel Static Analysis");
        assert!(driver["semantic_version"].as_str().unwrap().len() >= 5);
    }

    #[test]
    fn sarif_result_has_location() {
        let formatter = SarifFormatter;
        let r = report_data(
            vec![finding(Severity::High, "loc-rule", "src/lib.rs", 10, 3)],
            50,
        );
        let output = formatter.format(&r);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let result = &parsed["runs"][0]["results"][0];
        assert_eq!(result["rule_id"], "loc-rule");
        assert_eq!(result["level"], "error");

        let loc = &result["locations"][0]["physical_location"];
        assert_eq!(loc["artifact_location"]["uri"], "src/lib.rs");
        assert_eq!(loc["region"]["start_line"], 10);
        assert_eq!(loc["region"]["start_column"], 3);
    }

    #[test]
    fn sarif_severity_maps_to_level() {
        let formatter = SarifFormatter;

        // Critical -> error
        let r = report_data(vec![finding(Severity::Critical, "c", "f.rs", 1, 1)], 0);
        let p: serde_json::Value = serde_json::from_str(&formatter.format(&r)).unwrap();
        assert_eq!(p["runs"][0]["results"][0]["level"], "error");

        // Medium -> warning
        let r = report_data(vec![finding(Severity::Medium, "m", "f.rs", 1, 1)], 0);
        let p: serde_json::Value = serde_json::from_str(&formatter.format(&r)).unwrap();
        assert_eq!(p["runs"][0]["results"][0]["level"], "warning");

        // Low -> note
        let r = report_data(vec![finding(Severity::Low, "l", "f.rs", 1, 1)], 0);
        let p: serde_json::Value = serde_json::from_str(&formatter.format(&r)).unwrap();
        assert_eq!(p["runs"][0]["results"][0]["level"], "note");
    }

    #[test]
    fn sarif_has_rules_array() {
        let formatter = SarifFormatter;
        let findings = vec![
            finding(Severity::High, "rule-one", "a.rs", 1, 1),
            finding(Severity::High, "rule-two", "b.rs", 2, 2),
        ];
        let r = report_data(findings, 50);
        let output = formatter.format(&r);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let rules = &parsed["runs"][0]["tool"]["driver"]["rules"];
        let rule_ids: Vec<&str> = rules
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        assert!(rule_ids.contains(&"rule-one"));
        assert!(rule_ids.contains(&"rule-two"));
    }

    #[test]
    fn sarif_empty_findings_no_results() {
        let formatter = SarifFormatter;
        let r = report_data(vec![], 0);
        let output = formatter.format(&r);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["runs"][0]["results"].as_array().unwrap().is_empty());
    }

    #[test]
    fn sarif_has_properties() {
        let formatter = SarifFormatter;
        let r = report_data(
            vec![finding(Severity::Critical, "prop-rule", "f.rs", 1, 1)],
            50,
        );
        let output = formatter.format(&r);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let props = &parsed["runs"][0]["properties"];
        assert_eq!(props["score"], 75);
        assert_eq!(props["total_files"], 5);
        assert_eq!(props["total_rules"], 10);
    }

    #[test]
    fn sarif_rule_index_matches_rule_order() {
        let formatter = SarifFormatter;
        let findings = vec![
            finding(Severity::High, "rule-a", "a.rs", 1, 1),
            finding(Severity::Medium, "rule-b", "b.rs", 1, 1),
            finding(Severity::High, "rule-a", "a.rs", 2, 1),
        ];
        let r = report_data(findings, 0);
        let output = formatter.format(&r);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let results = parsed["runs"][0]["results"].as_array().unwrap();
        // After sorting by severity descending then file/line/column ascending:
        //   0: rule-a (High, a.rs:1:1) -> rule_index 0 (rule-a first alphabetically in BTreeMap)
        //   1: rule-a (High, a.rs:2:1) -> rule_index 0
        //   2: rule-b (Medium, b.rs:1:1) -> rule_index 1
        assert_eq!(results[0]["rule_id"], "rule-a");
        assert_eq!(results[0]["rule_index"], 0);
        assert_eq!(results[1]["rule_id"], "rule-a");
        assert_eq!(results[1]["rule_index"], 0);
        assert_eq!(results[2]["rule_id"], "rule-b");
        assert_eq!(results[2]["rule_index"], 1);
    }

    #[test]
    fn sarif_artifacts_includes_files() {
        let formatter = SarifFormatter;
        let findings = vec![
            finding(Severity::High, "r", "src/main.rs", 1, 1),
            finding(Severity::High, "r", "src/lib.rs", 1, 1),
        ];
        let r = report_data(findings, 0);
        let output = formatter.format(&r);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let artifacts = parsed["runs"][0]["artifacts"].as_array().unwrap();
        let uris: Vec<&str> = artifacts
            .iter()
            .map(|a| a["location"]["uri"].as_str().unwrap())
            .collect();
        assert!(uris.contains(&"src/main.rs"));
        assert!(uris.contains(&"src/lib.rs"));
    }
}
