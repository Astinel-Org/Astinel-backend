use std::io::{self, Write};
use std::time::Duration;

use crate::core::{Finding, SecurityScore};

/// Options that control how output is rendered.
#[derive(Debug, Clone, Copy)]
pub struct ReportOptions {
    pub color: bool,
    pub unicode: bool,
    pub width: usize,
}

impl Default for ReportOptions {
    fn default() -> Self {
        Self {
            color: true,
            unicode: true,
            width: 80,
        }
    }
}

/// Summary statistics for a scan run.
#[derive(Debug, Clone)]
pub struct ReportSummary {
    pub project_name: String,
    pub total_files: usize,
    pub total_rules: usize,
    pub total_findings: usize,
    pub suppressed_findings: usize,
    pub duration: Duration,
    pub parse_duration: Duration,
    pub rule_duration: Duration,
}

/// Complete report data consumed by every formatter.
#[derive(Debug, Clone)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub score: SecurityScore,
    pub summary: ReportSummary,
    pub options: ReportOptions,
}

impl Report {
    /// Return findings sorted deterministically for display:
    /// severity descending (Critical first), then file, line, column, rule ID ascending.
    pub fn sorted_findings(&self) -> Vec<Finding> {
        let mut sorted = self.findings.clone();
        sorted.sort_by(|a, b| {
            a.severity
                .cmp(&b.severity)
                .then_with(|| a.span.file.cmp(&b.span.file))
                .then_with(|| a.span.line.cmp(&b.span.line))
                .then_with(|| a.span.column.cmp(&b.span.column))
                .then_with(|| a.rule_id.cmp(&b.rule_id))
        });
        sorted
    }
}

/// Trait that every output formatter must implement.
pub trait OutputFormatter {
    fn format(&self, report: &Report) -> String;

    fn write(&self, writer: &mut dyn Write, report: &Report) -> io::Result<()> {
        let output = self.format(report);
        writer.write_all(output.as_bytes())?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Colour / style helpers shared across formatters
// ---------------------------------------------------------------------------

pub(crate) fn sev_color_name(severity: crate::core::Severity) -> &'static str {
    match severity {
        crate::core::Severity::Critical => "red",
        crate::core::Severity::High => "yellow",
        crate::core::Severity::Medium => "blue",
        crate::core::Severity::Low => "cyan",
        crate::core::Severity::Info => "white",
    }
}

pub(crate) fn style(text: &str, style_name: &str, color: bool) -> String {
    if !color {
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

pub(crate) fn score_color_name(score: u8) -> &'static str {
    if score >= 80 {
        "green"
    } else if score >= 50 {
        "yellow"
    } else {
        "red"
    }
}

pub(crate) fn severity_bar(score: u8, unicode: bool) -> String {
    let bar_width = 20usize;
    let filled = (score as usize * bar_width) / 100;
    let empty = bar_width.saturating_sub(filled);
    if unicode {
        format!("{}{}", "█".repeat(filled), "░".repeat(empty))
    } else {
        format!("{}{}", "#".repeat(filled), "-".repeat(empty))
    }
}
