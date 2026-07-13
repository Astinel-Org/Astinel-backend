use serde::Serialize;

use crate::scanner::report::formatter::*;

/// Versioned JSON output with stable field ordering.
pub struct JsonFormatter;

#[derive(Serialize)]
struct JsonOutput {
    schema: String,
    version: String,
    sentinel_version: String,
    summary: JsonSummary,
    score: JsonScore,
    findings: Vec<JsonFinding>,
    timings: JsonTimings,
}

#[derive(Serialize)]
struct JsonSummary {
    project_name: String,
    total_files: usize,
    total_rules: usize,
    total_findings: usize,
    suppressed_findings: usize,
    duration_ms: u64,
}

#[derive(Serialize)]
struct JsonScore {
    score: u8,
    critical: usize,
    high: usize,
    medium: usize,
    low: usize,
    info: usize,
}

#[derive(Serialize)]
struct JsonTimings {
    parse_ms: u64,
    analysis_ms: u64,
    total_ms: u64,
}

#[derive(Serialize)]
struct JsonFinding {
    rule_id: String,
    severity: String,
    category: String,
    message: String,
    recommendation: String,
    fix_example: Option<String>,
    location: JsonLocation,
}

#[derive(Serialize)]
struct JsonLocation {
    file: String,
    line: usize,
    column: usize,
}

impl OutputFormatter for JsonFormatter {
    fn format(&self, report: &Report) -> String {
        let output = JsonOutput {
            schema: "https://sentinel.dev/schemas/output/v1.json".to_string(),
            version: "1.0.0".to_string(),
            sentinel_version: env!("CARGO_PKG_VERSION").to_string(),
            summary: JsonSummary {
                project_name: report.summary.project_name.clone(),
                total_files: report.summary.total_files,
                total_rules: report.summary.total_rules,
                total_findings: report.summary.total_findings,
                suppressed_findings: report.summary.suppressed_findings,
                duration_ms: report.summary.duration.as_millis() as u64,
            },
            score: JsonScore {
                score: report.score.score,
                critical: report.score.critical,
                high: report.score.high,
                medium: report.score.medium,
                low: report.score.low,
                info: report.score.info,
            },
            findings: report
                .sorted_findings()
                .iter()
                .map(|f| JsonFinding {
                    rule_id: f.rule_id.to_string(),
                    severity: f.severity.as_str().to_string(),
                    category: f.category.as_str().to_string(),
                    message: f.message.clone(),
                    recommendation: f.recommendation.clone(),
                    fix_example: f.fix_example.clone(),
                    location: JsonLocation {
                        file: f.span.file.to_string_lossy().to_string(),
                        line: f.span.line,
                        column: f.span.column,
                    },
                })
                .collect(),
            timings: JsonTimings {
                parse_ms: report.summary.parse_duration.as_millis() as u64,
                analysis_ms: report.summary.rule_duration.as_millis() as u64,
                total_ms: report.summary.duration.as_millis() as u64,
            },
        };

        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string())
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
    fn json_output_valid() {
        let formatter = JsonFormatter;
        let r = report_data(
            vec![finding(Severity::High, "json-rule", "j.rs", 3, 7)],
            100,
        );
        let output = formatter.format(&r);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["version"], "1.0.0");
        assert_eq!(parsed["findings"][0]["rule_id"], "json-rule");
        assert_eq!(parsed["score"]["score"], 90);
        assert_eq!(parsed["summary"]["duration_ms"], 100);
        assert!(parsed.get("timings").is_some());
        assert_eq!(parsed["summary"]["project_name"], "test-project");
    }

    #[test]
    fn json_has_timings() {
        let formatter = JsonFormatter;
        let r = report_data(
            vec![finding(Severity::Info, "timing-rule", "t.rs", 1, 1)],
            200,
        );
        let output = formatter.format(&r);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let timings = &parsed["timings"];
        assert_eq!(timings["total_ms"], 200);
        assert_eq!(timings["parse_ms"], 10);
        assert_eq!(timings["analysis_ms"], 40);
    }

    #[test]
    fn json_stable_field_order() {
        let formatter = JsonFormatter;
        let r = report_data(
            vec![finding(Severity::Critical, "order-rule", "o.rs", 1, 1)],
            50,
        );
        let output = formatter.format(&r);
        // Schema field must come first
        assert!(output.contains("\"schema\""));
        let schema_pos = output.find("\"schema\"").unwrap();
        let version_pos = output.find("\"version\"").unwrap();
        assert!(schema_pos < version_pos);

        // summary before score
        let summary_pos = output.find("\"summary\"").unwrap();
        let score_pos = output.find("\"score\"").unwrap();
        assert!(summary_pos < score_pos);
    }

    #[test]
    fn json_schema_url() {
        let formatter = JsonFormatter;
        let r = report_data(vec![], 0);
        let output = formatter.format(&r);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            parsed["schema"],
            "https://sentinel.dev/schemas/output/v1.json"
        );
    }

    #[test]
    fn json_empty_findings_empty_array() {
        let formatter = JsonFormatter;
        let r = report_data(vec![], 0);
        let output = formatter.format(&r);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["findings"].as_array().unwrap().is_empty());
    }
}
