use std::io::Write;

/// Controls the verbosity of progress output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressMode {
    /// Show progress on stderr (default)
    Auto,
    /// Suppress all progress output
    Quiet,
    /// Show detailed progress output
    Verbose,
}

/// Simple progress reporter that writes to stderr.
pub struct Progress {
    mode: ProgressMode,
}

impl Progress {
    pub fn new(mode: ProgressMode) -> Self {
        Self { mode }
    }

    /// Show an informational progress message on stderr.
    pub fn info(&self, msg: impl std::fmt::Display) {
        if self.mode != ProgressMode::Quiet {
            let _ = writeln!(std::io::stderr(), "{}", msg);
        }
    }

    /// Show a verbose progress message (only in Verbose mode).
    pub fn verbose(&self, msg: impl std::fmt::Display) {
        if self.mode == ProgressMode::Verbose {
            let _ = writeln!(std::io::stderr(), "{}", msg);
        }
    }

    /// Show a warning message on stderr.
    pub fn warn(&self, msg: impl std::fmt::Display) {
        if self.mode != ProgressMode::Quiet {
            let _ = writeln!(std::io::stderr(), "warning: {}", msg);
        }
    }

    /// Report the number of findings discovered.
    pub fn finding_count(&self, count: usize) {
        self.info(format_args!("Found {} issue(s)", count));
    }

    /// Report the number of files scanned.
    pub fn files_scanned(&self, count: usize) {
        self.info(format_args!("Scanned {} file(s)", count));
    }

    /// Report the number of rules executed (verbose only).
    pub fn rules_run(&self, count: usize) {
        self.verbose(format_args!("Executed {} rule(s)", count));
    }
}

impl From<bool> for ProgressMode {
    fn from(quiet: bool) -> Self {
        if quiet {
            ProgressMode::Quiet
        } else {
            ProgressMode::Auto
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_mode_suppresses_info() {
        let p = Progress::new(ProgressMode::Quiet);
        p.info("should not appear");
        p.warn("should not appear");
        p.finding_count(5);
    }

    #[test]
    fn verbose_mode_enables_all() {
        let p = Progress::new(ProgressMode::Verbose);
        p.info("info");
        p.verbose("verbose detail");
    }
}
