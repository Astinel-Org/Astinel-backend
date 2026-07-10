use std::process;

/// Deterministic exit codes for the Sentinel CLI.
///
/// Every possible exit condition has a unique, predictable code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    FindingsDetected = 1,
    InvalidArguments = 2,
    InvalidConfiguration = 3,
    ParseFailure = 4,
    InternalError = 5,
    PermissionDenied = 6,
    ProjectNotFound = 7,
    UnsupportedProject = 8,
}

impl ExitCode {
    /// Convert this exit code to its numeric value.
    pub fn to_i32(self) -> i32 {
        self as i32
    }

    /// Exit the process with this exit code.
    pub fn exit(self) -> ! {
        process::exit(self.to_i32());
    }

    /// Determine the exit code based on findings and the `--fail-on` threshold.
    ///
    /// Returns `FindingsDetected` if there are findings at or above the severity
    /// threshold, `Success` otherwise.
    pub fn from_findings_and_severity(
        has_findings: bool,
        fail_on: sentinel_core::Severity,
        findings: &[sentinel_core::Finding],
    ) -> Self {
        if !has_findings {
            return ExitCode::Success;
        }
        let most_severe = findings
            .iter()
            .map(|f| f.severity)
            .min()
            .unwrap_or(sentinel_core::Severity::Info);
        if most_severe <= fail_on {
            ExitCode::FindingsDetected
        } else {
            ExitCode::Success
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sentinel_core::{Category, DiagnosticSpan, Finding, RuleId, Severity};

    fn finding(severity: Severity) -> Finding {
        Finding::new(
            RuleId::new("test").unwrap(),
            severity,
            Category::Security,
            DiagnosticSpan::new("f.rs", 1, 1),
            "msg",
            "fix",
        )
    }

    #[test]
    fn no_findings_is_success() {
        let code = ExitCode::from_findings_and_severity(false, Severity::Low, &[]);
        assert_eq!(code, ExitCode::Success);
    }

    #[test]
    fn findings_below_threshold_are_success() {
        let f = finding(Severity::Info);
        let code = ExitCode::from_findings_and_severity(true, Severity::Low, &[f]);
        assert_eq!(code, ExitCode::Success);
    }

    #[test]
    fn findings_at_threshold_return_findings_detected() {
        let f = finding(Severity::Critical);
        let code = ExitCode::from_findings_and_severity(true, Severity::Low, &[f]);
        assert_eq!(code, ExitCode::FindingsDetected);
    }

    #[test]
    fn exit_code_values() {
        assert_eq!(ExitCode::Success.to_i32(), 0);
        assert_eq!(ExitCode::FindingsDetected.to_i32(), 1);
        assert_eq!(ExitCode::InvalidArguments.to_i32(), 2);
        assert_eq!(ExitCode::InvalidConfiguration.to_i32(), 3);
        assert_eq!(ExitCode::ParseFailure.to_i32(), 4);
        assert_eq!(ExitCode::InternalError.to_i32(), 5);
        assert_eq!(ExitCode::PermissionDenied.to_i32(), 6);
        assert_eq!(ExitCode::ProjectNotFound.to_i32(), 7);
        assert_eq!(ExitCode::UnsupportedProject.to_i32(), 8);
    }

    #[test]
    fn findings_at_or_above_fail_on_are_detected() {
        let f = finding(Severity::High);
        let code = ExitCode::from_findings_and_severity(true, Severity::High, &[f]);
        assert_eq!(code, ExitCode::FindingsDetected);
    }

    #[test]
    fn finding_below_fail_on_allows_success() {
        let f = finding(Severity::Info);
        let code = ExitCode::from_findings_and_severity(true, Severity::Critical, &[f]);
        assert_eq!(code, ExitCode::Success);
    }

    #[test]
    fn empty_findings_is_success() {
        let code = ExitCode::from_findings_and_severity(false, Severity::Critical, &[]);
        assert_eq!(code, ExitCode::Success);
    }
}
