use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

use sentinel_core::{Finding, SecurityScore};
use serde::Serialize;

use crate::terminal::TerminalCapabilities;

/// Formatter trait for scan output.
/// Trait for formatting scan output (pretty, compact, or JSON).
pub trait OutputFormatter {
    fn write(
        &self,
        writer: &mut dyn Write,
        findings: &[Finding],
        score: &SecurityScore,
        summary: &ScanOutputSummary,
        caps: &TerminalCapabilities,
    ) -> io::Result<()>;
}

/// Summary of a scan run.
#[derive(Debug, Clone)]
/// Summary statistics for a scan run.
pub struct ScanOutputSummary {
    pub project_name: String,
    pub total_files: usize,
    pub total_rules: usize,
    pub total_findings: usize,
    pub suppressed_findings: usize,
    pub duration: Duration,
    pub parse_duration: Duration,
    pub rule_duration: Duration,
}

/// Pretty human-readable formatter.
/// Human-readable pretty-printed formatter.
pub struct PrettyFormatter;

fn file_display(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

impl OutputFormatter for PrettyFormatter {
    fn write(
        &self,
        writer: &mut dyn Write,
        findings: &[Finding],
        score: &SecurityScore,
        summary: &ScanOutputSummary,
        caps: &TerminalCapabilities,
    ) -> io::Result<()> {
        if !summary.project_name.is_empty() {
            writeln!(
                writer,
                "{}",
                style(&format!("Project: {}", summary.project_name), "bold", caps)
            )?;
        }

        if findings.is_empty() {
            writeln!(
                writer,
                "{}",
                style("No issues found. Your code looks clean!", "green", caps)
            )?;
            write_score(writer, score, caps)?;
            write_summary(writer, summary, caps)?;
            return Ok(());
        }

        writeln!(writer)?;
        let mut current_severity = None;
        for finding in findings {
            let sev_str = finding.severity.as_str();
            if current_severity != Some(finding.severity) {
                current_severity = Some(finding.severity);
                let header_color = match finding.severity {
                    sentinel_core::Severity::Critical => "red",
                    sentinel_core::Severity::High => "yellow",
                    sentinel_core::Severity::Medium => "blue",
                    sentinel_core::Severity::Low => "cyan",
                    sentinel_core::Severity::Info => "white",
                };
                writeln!(
                    writer,
                    "{}",
                    style(&format!("── {} ──", sev_str.to_uppercase()), header_color, caps)
                )?;
            }

            let sev_colored = style(sev_str, sev_color(finding.severity), caps);
            let fd = file_display(&finding.span.file);
            let line_col = format!(":{}:{}", finding.span.line, finding.span.column);
            writeln!(
                writer,
                "  {}  {}  {}",
                sev_colored,
                style(finding.rule_id.as_ref(), "bold", caps),
                style(&format!("{}{}", fd, line_col), "underline", caps),
            )?;
            writeln!(writer, "       {}", finding.message)?;
            if !finding.recommendation.is_empty() {
                writeln!(
                    writer,
                    "       {} {}",
                    style("→", "green", caps),
                    finding.recommendation
                )?;
            }
            if let Some(ref fix) = finding.fix_example {
                writeln!(writer, "       {} {}", style("fix:", "cyan", caps), fix)?;
            }
            writeln!(writer)?;
        }

        write_score(writer, score, caps)?;
        write_summary(writer, summary, caps)?;
        if summary.rule_duration > Duration::ZERO {
            writeln!(writer, "  Parse time:     {}ms", summary.parse_duration.as_millis())?;
            writeln!(writer, "  Analysis time:  {}ms", summary.rule_duration.as_millis())?;
        }
        writeln!(writer)?;
        Ok(())
    }
}

fn write_score(writer: &mut dyn Write, score: &SecurityScore, caps: &TerminalCapabilities) -> io::Result<()> {
    let bar_width = 20usize;
    let filled = (score.score as usize * bar_width) / 100;
    let empty = bar_width.saturating_sub(filled);

    let bar = if caps.unicode {
        format!("{}{}", "█".repeat(filled), "░".repeat(empty))
    } else {
        format!("{}{}", "#".repeat(filled), "-".repeat(empty))
    };

    let score_color = if score.score >= 80 {
        "green"
    } else if score.score >= 50 {
        "yellow"
    } else {
        "red"
    };

    writeln!(
        writer,
        "{}  {}  {} /100",
        style("Security Score:", "bold", caps),
        style(&bar, score_color, caps),
        style(&score.score.to_string(), score_color, caps),
    )?;
    writeln!(
        writer,
        "  {} critical, {} high, {} medium, {} low, {} info",
        style(&score.critical.to_string(), "red", caps),
        style(&score.high.to_string(), "yellow", caps),
        style(&score.medium.to_string(), "blue", caps),
        style(&score.low.to_string(), "cyan", caps),
        style(&score.info.to_string(), "white", caps),
    )?;
    writeln!(writer)?;
    Ok(())
}

fn write_summary(writer: &mut dyn Write, summary: &ScanOutputSummary, _caps: &TerminalCapabilities) -> io::Result<()> {
    writeln!(writer, "── Summary ──")?;
    writeln!(writer, "  Files scanned:    {}", summary.total_files)?;
    writeln!(writer, "  Rules executed:   {}", summary.total_rules)?;
    writeln!(writer, "  Findings:         {}", summary.total_findings)?;
    if summary.suppressed_findings > 0 {
        writeln!(writer, "  Suppressed:       {}", summary.suppressed_findings)?;
    }
    writeln!(writer, "  Duration:         {}ms", summary.duration.as_millis())?;
    writeln!(writer)?;
    Ok(())
}

/// Compact tab-delimited formatter.
/// Compact tab-delimited formatter (one line per finding).
pub struct CompactFormatter;

impl OutputFormatter for CompactFormatter {
    fn write(
        &self,
        writer: &mut dyn Write,
        findings: &[Finding],
        _score: &SecurityScore,
        _summary: &ScanOutputSummary,
        caps: &TerminalCapabilities,
    ) -> io::Result<()> {
        for finding in findings {
            let sev = style(finding.severity.as_str(), sev_color(finding.severity), caps);
            let fd = file_display(&finding.span.file);
            writeln!(
                writer,
                "{}\t{}\t{}:{}:{}\t{}",
                sev, finding.rule_id, fd, finding.span.line, finding.span.column, finding.message,
            )?;
        }
        Ok(())
    }
}

/// JSON formatter with versioned schema.
/// Versioned JSON formatter.
pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn write(
        &self,
        writer: &mut dyn Write,
        findings: &[Finding],
        score: &SecurityScore,
        summary: &ScanOutputSummary,
        _caps: &TerminalCapabilities,
    ) -> io::Result<()> {
        let output = JsonOutput {
            schema: "https://sentinel.dev/schemas/output/v1.json".to_string(),
            version: "1.0.0".to_string(),
            sentinel_version: env!("CARGO_PKG_VERSION").to_string(),
            summary: JsonSummary {
                project_name: summary.project_name.clone(),
                total_files: summary.total_files,
                total_rules: summary.total_rules,
                total_findings: summary.total_findings,
                suppressed_findings: summary.suppressed_findings,
                duration_ms: summary.duration.as_millis() as u64,
            },
            score: JsonScore {
                score: score.score,
                critical: score.critical,
                high: score.high,
                medium: score.medium,
                low: score.low,
                info: score.info,
            },
            findings: findings
                .iter()
                .map(|f| JsonFinding {
                    rule_id: f.rule_id.to_string(),
                    severity: f.severity.as_str().to_string(),
                    category: f.category.as_str().to_string(),
                    message: f.message.clone(),
                    recommendation: f.recommendation.clone(),
                    fix_example: f.fix_example.clone(),
                    location: JsonLocation {
                        file: file_display(&f.span.file),
                        line: f.span.line,
                        column: f.span.column,
                    },
                })
                .collect(),
            timings: JsonTimings {
                parse_ms: summary.parse_duration.as_millis() as u64,
                analysis_ms: summary.rule_duration.as_millis() as u64,
                total_ms: summary.duration.as_millis() as u64,
            },
        };

        let json = serde_json::to_string_pretty(&output).map_err(io::Error::other)?;
        writeln!(writer, "{}", json)?;
        Ok(())
    }
}

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

fn sev_color(severity: sentinel_core::Severity) -> &'static str {
    match severity {
        sentinel_core::Severity::Critical => "red",
        sentinel_core::Severity::High => "yellow",
        sentinel_core::Severity::Medium => "blue",
        sentinel_core::Severity::Low => "cyan",
        sentinel_core::Severity::Info => "white",
    }
}

fn style(text: &str, style_name: &str, caps: &TerminalCapabilities) -> String {
    if !caps.color {
        return text.to_string();
    }
    match style_name {
        "red" => format!("\x1b[31m{}\x1b[0m", text),
        "green" => format!("\x1b[32m{}\x1b[0m", text),
        "yellow" => format!("\x1b[33m{}\x1b[0m", text),
        "blue" => format!("\x1b[34m{}\x1b[0m", text),
        "cyan" => format!("\x1b[36m{}\x1b[0m", text),
        "white" => format!("\x1b[37m{}\x1b[0m", text),
        "bold" => format!("\x1b[1m{}\x1b[0m", text),
        "underline" => format!("\x1b[4m{}\x1b[0m", text),
        _ => text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sentinel_core::{Category, DiagnosticSpan, Finding, RuleId, Severity};

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

    fn caps() -> TerminalCapabilities {
        TerminalCapabilities {
            color: false,
            unicode: false,
            width: 80,
        }
    }

    fn summary(duration_ms: u64) -> ScanOutputSummary {
        ScanOutputSummary {
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

    #[test]
    fn pretty_empty_output() {
        let formatter = PrettyFormatter;
        let mut buf = Vec::new();
        formatter
            .write(&mut buf, &[], &SecurityScore::perfect(), &summary(50), &caps())
            .unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("No issues found"));
        assert!(output.contains("100"));
        assert!(output.contains("test-project"));
    }

    #[test]
    fn pretty_with_findings() {
        let formatter = PrettyFormatter;
        let findings = vec![finding(Severity::Critical, "test-rule", "f.rs", 10, 5)];
        let score = SecurityScore::from_findings(&findings);
        let mut buf = Vec::new();
        formatter
            .write(&mut buf, &findings, &score, &summary(50), &caps())
            .unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("CRITICAL"));
        assert!(output.contains("test-rule"));
        assert!(output.contains("f.rs"));
    }

    #[test]
    fn compact_output() {
        let formatter = CompactFormatter;
        let findings = vec![finding(Severity::High, "some-rule", "a.rs", 5, 1)];
        let mut buf = Vec::new();
        formatter
            .write(&mut buf, &findings, &SecurityScore::perfect(), &summary(0), &caps())
            .unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("high"));
        assert!(output.contains("some-rule"));
        assert!(output.contains("a.rs:5:1"));
    }

    #[test]
    fn json_output_valid() {
        let formatter = JsonFormatter;
        let findings = vec![finding(Severity::High, "json-rule", "j.rs", 3, 7)];
        let score = SecurityScore::from_findings(&findings);
        let mut buf = Vec::new();
        formatter
            .write(&mut buf, &findings, &score, &summary(100), &caps())
            .unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["version"], "1.0.0");
        assert_eq!(parsed["findings"][0]["rule_id"], "json-rule");
        assert_eq!(parsed["score"]["score"], 90);
        assert_eq!(parsed["summary"]["duration_ms"], 100);
        assert!(parsed.get("timings").is_some(), "JSON output must include timings");
        assert_eq!(parsed["summary"]["project_name"], "test-project");
    }

    #[test]
    fn json_output_has_timings() {
        let formatter = JsonFormatter;
        let findings = vec![finding(Severity::Info, "timing-rule", "t.rs", 1, 1)];
        let score = SecurityScore::from_findings(&findings);
        let mut buf = Vec::new();
        formatter
            .write(&mut buf, &findings, &score, &summary(200), &caps())
            .unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let timings = &parsed["timings"];
        assert_eq!(timings["total_ms"], 200);
        assert_eq!(timings["parse_ms"], 10);
        assert_eq!(timings["analysis_ms"], 40);
    }
}
