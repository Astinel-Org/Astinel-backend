use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct ScanMetrics {
    pub total_files: usize,
    pub total_rules: usize,
    pub total_findings: usize,
    pub suppressed_findings: usize,
    pub critical_findings: usize,
    pub high_findings: usize,
    pub medium_findings: usize,
    pub low_findings: usize,
    pub info_findings: usize,
    pub duration: Duration,
    pub parse_duration: Duration,
    pub rule_duration: Duration,
    pub report_duration: Duration,
    pub ai_duration: Option<Duration>,
}
