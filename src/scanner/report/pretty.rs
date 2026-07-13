use std::time::Duration;

use crate::scanner::report::formatter::*;

/// Human-readable, severity-grouped terminal output.
pub struct PrettyFormatter;

impl OutputFormatter for PrettyFormatter {
    fn format(&self, report: &Report) -> String {
        let mut out = String::new();
        let caps = report.options;

        if !report.summary.project_name.is_empty() {
            out.push_str(&style(
                &format!("Project: {}\n", report.summary.project_name),
                "bold",
                caps.color,
            ));
        }

        let sorted = report.sorted_findings();

        if sorted.is_empty() {
            out.push_str(&style(
                "No issues found. Your code looks clean!\n",
                "green",
                caps.color,
            ));
            write_score(&mut out, &report.score, &caps);
            write_summary(&mut out, &report.summary, &caps);
            return out;
        }

        out.push('\n');
        let mut current_severity = None;
        for finding in &sorted {
            if current_severity != Some(finding.severity) {
                current_severity = Some(finding.severity);
                let header_color = sev_color_name(finding.severity);
                out.push_str(&style(
                    &format!("── {} ──\n", finding.severity.as_str().to_uppercase()),
                    header_color,
                    caps.color,
                ));
            }

            let sev_colored = style(
                finding.severity.as_str(),
                sev_color_name(finding.severity),
                caps.color,
            );
            let fd = finding.span.file.display();
            let line_col = format!(":{}:{}", finding.span.line, finding.span.column);
            out.push_str(&format!(
                "  {}  {}  {}\n",
                sev_colored,
                style(finding.rule_id.as_ref(), "bold", caps.color),
                style(&format!("{}{}", fd, line_col), "underline", caps.color),
            ));
            out.push_str(&format!("       {}\n", finding.message));
            if !finding.recommendation.is_empty() {
                out.push_str(&format!(
                    "       {} {}\n",
                    style("→", "green", caps.color),
                    finding.recommendation
                ));
            }
            if let Some(ref fix) = finding.fix_example {
                out.push_str(&format!(
                    "       {} {}\n",
                    style("fix:", "cyan", caps.color),
                    fix
                ));
            }
            out.push('\n');
        }

        write_score(&mut out, &report.score, &caps);
        write_summary(&mut out, &report.summary, &caps);
        if report.summary.rule_duration > Duration::ZERO {
            out.push_str(&format!(
                "  Parse time:     {}ms\n",
                report.summary.parse_duration.as_millis()
            ));
            out.push_str(&format!(
                "  Analysis time:  {}ms\n",
                report.summary.rule_duration.as_millis()
            ));
        }
        out.push('\n');
        out
    }
}

fn write_score(out: &mut String, score: &crate::core::SecurityScore, caps: &ReportOptions) {
    let bar = severity_bar(score.score, caps.unicode);
    let sc = score_color_name(score.score);

    out.push_str(&format!(
        "{}  {}  {} /100\n",
        style("Security Score:", "bold", caps.color),
        style(&bar, sc, caps.color),
        style(&score.score.to_string(), sc, caps.color),
    ));
    out.push_str(&format!(
        "  {} critical, {} high, {} medium, {} low, {} info\n",
        style(&score.critical.to_string(), "red", caps.color),
        style(&score.high.to_string(), "yellow", caps.color),
        style(&score.medium.to_string(), "blue", caps.color),
        style(&score.low.to_string(), "cyan", caps.color),
        style(&score.info.to_string(), "white", caps.color),
    ));
    out.push('\n');
}

fn write_summary(out: &mut String, summary: &ReportSummary, _caps: &ReportOptions) {
    out.push_str("── Summary ──\n");
    out.push_str(&format!("  Files scanned:    {}\n", summary.total_files));
    out.push_str(&format!("  Rules executed:   {}\n", summary.total_rules));
    out.push_str(&format!("  Findings:         {}\n", summary.total_findings));
    if summary.suppressed_findings > 0 {
        out.push_str(&format!(
            "  Suppressed:       {}\n",
            summary.suppressed_findings
        ));
    }
    out.push_str(&format!(
        "  Duration:         {}ms\n",
        summary.duration.as_millis()
    ));
    out.push('\n');
}

// ---------------------------------------------------------------------------
// Compact (one-line-per-finding) formatter
// ---------------------------------------------------------------------------

pub struct CompactFormatter;

impl OutputFormatter for CompactFormatter {
    fn format(&self, report: &Report) -> String {
        let mut out = String::new();
        let caps = report.options;

        for finding in &report.sorted_findings() {
            let sev = style(
                finding.severity.as_str(),
                sev_color_name(finding.severity),
                caps.color,
            );
            let fd = finding.span.file.display();
            out.push_str(&format!(
                "{}\t{}\t{}:{}:{}\t{}\n",
                sev, finding.rule_id, fd, finding.span.line, finding.span.column, finding.message,
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Category, DiagnosticSpan, Finding, RuleId, Severity};

    fn finding(severity: Severity, rule: &str, file: &str, line: usize, col: usize) -> Finding {
        Finding::new(
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
            duration: Duration::from_millis(duration_ms),
            parse_duration: Duration::from_millis(10),
            rule_duration: Duration::from_millis(40),
        }
    }

    fn report(findings: Vec<Finding>, duration_ms: u64) -> Report {
        Report {
            score: crate::core::SecurityScore::from_findings(&findings),
            summary: summary(duration_ms),
            options: caps(),
            findings,
        }
    }

    #[test]
    fn pretty_empty_output() {
        let formatter = PrettyFormatter;
        let r = report(vec![], 50);
        let output = formatter.format(&r);
        assert!(output.contains("No issues found"));
        assert!(output.contains("100"));
        assert!(output.contains("test-project"));
    }

    #[test]
    fn pretty_with_findings() {
        let formatter = PrettyFormatter;
        let r = report(
            vec![finding(Severity::Critical, "test-rule", "f.rs", 10, 5)],
            50,
        );
        let output = formatter.format(&r);
        assert!(output.contains("CRITICAL"));
        assert!(output.contains("test-rule"));
        assert!(output.contains("f.rs"));
    }

    #[test]
    fn compact_output() {
        let formatter = CompactFormatter;
        let r = report(vec![finding(Severity::High, "some-rule", "a.rs", 5, 1)], 0);
        let output = formatter.format(&r);
        assert!(output.contains("high"));
        assert!(output.contains("some-rule"));
        assert!(output.contains("a.rs:5:1"));
    }

    #[test]
    fn pretty_groups_by_severity_descending() {
        let findings = vec![
            finding(Severity::Critical, "rule-a", "a.rs", 1, 1),
            finding(Severity::High, "rule-b", "b.rs", 2, 1),
            finding(Severity::Info, "rule-c", "c.rs", 3, 1),
        ];
        let formatter = PrettyFormatter;
        let r = report(findings, 50);
        let output = formatter.format(&r);
        // CRITICAL first, then HIGH, then INFO
        let crit_pos = output.find("CRITICAL").unwrap();
        let high_pos = output.find("HIGH").unwrap();
        let info_pos = output.find("INFO").unwrap();
        assert!(crit_pos < high_pos, "CRITICAL should come before HIGH");
        assert!(high_pos < info_pos, "HIGH should come before INFO");
    }

    #[test]
    fn compact_no_findings_empty() {
        let formatter = CompactFormatter;
        let r = report(vec![], 0);
        let output = formatter.format(&r);
        assert_eq!(output, "");
    }

    #[test]
    fn pretty_score_bar_renders() {
        let formatter = PrettyFormatter;
        let r = report(vec![], 50);
        let output = formatter.format(&r);
        assert!(output.contains("Security Score"));
        assert!(output.contains("/100"));
    }

    #[test]
    fn pretty_shows_recommendation() {
        let mut f = finding(Severity::High, "test-rule", "f.rs", 1, 1);
        f.recommendation = "use checked arithmetic".to_string();
        let formatter = PrettyFormatter;
        let r = report(vec![f], 50);
        let output = formatter.format(&r);
        assert!(output.contains("use checked arithmetic"));
    }

    #[test]
    fn pretty_shows_fix_example() {
        let mut f = finding(Severity::Medium, "fix-rule", "f.rs", 3, 7);
        f.fix_example = Some("use wrapping_add".to_string());
        let formatter = PrettyFormatter;
        let r = report(vec![f], 50);
        let output = formatter.format(&r);
        assert!(output.contains("use wrapping_add"));
    }

    #[test]
    fn cross_platform_newlines() {
        let formatter = PrettyFormatter;
        let r = report(vec![finding(Severity::Low, "nl-rule", "f.rs", 1, 1)], 10);
        let output = formatter.format(&r);
        // All newlines should be '\n' (Unix style)
        assert!(!output.contains("\r\n"));
        assert!(output.ends_with('\n'));
    }
}
