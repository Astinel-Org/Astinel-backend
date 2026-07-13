use std::path::Path;

use clap::Parser;

use crate::cli::config::CliArgs;
use crate::cli::errors::CliError;
use crate::cli::exit::ExitCode;
use crate::cli::progress::{Progress, ProgressMode};
use crate::cli::scan;
use crate::cli::terminal::ColorPreference;

#[derive(Parser, Debug)]
#[command(
    name = "sentinel",
    version,
    about = "A deterministic static analysis toolkit for Soroban smart contracts",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
    /// Scan a Soroban project for security issues
    Scan(Box<ScanArgs>),
    /// Show version information
    Version,
}

#[derive(Parser, Debug)]
pub struct ScanArgs {
    /// Path to scan (file or directory)
    #[arg(default_value = ".")]
    pub path: String,

    /// Output findings as JSON
    #[arg(long)]
    pub json: bool,

    /// Compact output (one line per finding)
    #[arg(long)]
    pub compact: bool,

    /// Suppress progress output
    #[arg(long, short = 'q')]
    pub quiet: bool,

    /// Verbose output
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Color output: auto, always, never
    #[arg(long, default_value = "auto")]
    pub color: String,

    /// Fail if any finding at or above this severity
    #[arg(long)]
    pub fail_on: Option<String>,

    /// Minimum severity to report
    #[arg(long)]
    pub severity: Option<String>,

    /// Category to filter by
    #[arg(long)]
    pub category: Option<String>,

    /// Only run specific rules (comma-separated)
    #[arg(long)]
    pub rule: Option<String>,

    /// Exclude files matching these path patterns (comma-separated)
    #[arg(long)]
    pub exclude: Option<String>,

    /// Show timing breakdown
    #[arg(long)]
    pub timings: bool,

    /// Show security score
    #[arg(long)]
    pub score: bool,

    /// Number of threads
    #[arg(long)]
    pub threads: Option<usize>,

    /// Path to configuration file
    #[arg(long)]
    pub config: Option<String>,
}

/// Dispatch the CLI command to its handler, returning an exit code or error.
pub fn dispatch(cli: Cli) -> Result<ExitCode, CliError> {
    match cli.command {
        Commands::Scan(args) => handle_scan(*args),
        Commands::Version => handle_version(),
    }
}

fn handle_scan(args: ScanArgs) -> Result<ExitCode, CliError> {
    let color_pref = ColorPreference::parse(&args.color).ok_or_else(|| {
        CliError::InvalidArguments(format!(
            "Invalid color value: `{}`. Use auto, always, or never.",
            args.color
        ))
    })?;

    let progress_mode = if args.quiet {
        ProgressMode::Quiet
    } else if args.verbose {
        ProgressMode::Verbose
    } else {
        ProgressMode::Auto
    };
    let progress = Progress::new(progress_mode);

    let cli_args = CliArgs {
        path: args.path,
        json: args.json,
        compact: args.compact,
        quiet: args.quiet,
        verbose: args.verbose,
        color: args.color,
        fail_on: args.fail_on,
        severity: args.severity,
        category: args.category,
        rule: args.rule,
        exclude: args.exclude,
        timings: args.timings,
        score: args.score,
        threads: args.threads,
        config_path: args.config,
    };

    let config = cli_args.merge_into_run_config()?;

    let path = Path::new(&cli_args.path);
    scan::run_scan(
        path,
        &config,
        &progress,
        cli_args.json,
        cli_args.compact,
        color_pref,
    )
}

fn handle_version() -> Result<ExitCode, CliError> {
    println!("sentinel v{}", env!("CARGO_PKG_VERSION"));
    Ok(ExitCode::Success)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_command_succeeds() {
        let cli = Cli::parse_from(["sentinel", "version"]);
        let result = dispatch(cli);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ExitCode::Success);
    }

    #[test]
    fn invalid_color_returns_error() {
        let cli = Cli::parse_from(["sentinel", "scan", "--color", "invalid"]);
        let result = dispatch(cli);
        assert!(result.is_err());
    }

    #[test]
    fn scan_default_args_ok() {
        let cli = Cli::parse_from(["sentinel", "scan"]);
        let args = match cli.command {
            Commands::Scan(ref a) => a,
            _ => panic!("expected scan command"),
        };
        assert_eq!(args.path, ".");
        assert!(!args.json);
        assert!(!args.compact);
    }

    #[test]
    fn scan_json_flag_set() {
        let cli = Cli::parse_from(["sentinel", "scan", "--json"]);
        let args = match cli.command {
            Commands::Scan(ref a) => a,
            _ => panic!("expected scan command"),
        };
        assert!(args.json);
    }

    #[test]
    fn scan_compact_flag_set() {
        let cli = Cli::parse_from(["sentinel", "scan", "--compact"]);
        let args = match cli.command {
            Commands::Scan(ref a) => a,
            _ => panic!("expected scan command"),
        };
        assert!(args.compact);
    }

    #[test]
    fn scan_quiet_flag_set() {
        let cli = Cli::parse_from(["sentinel", "scan", "-q"]);
        let args = match cli.command {
            Commands::Scan(ref a) => a,
            _ => panic!("expected scan command"),
        };
        assert!(args.quiet);
    }

    #[test]
    fn scan_verbose_flag_set() {
        let cli = Cli::parse_from(["sentinel", "scan", "-v"]);
        let args = match cli.command {
            Commands::Scan(ref a) => a,
            _ => panic!("expected scan command"),
        };
        assert!(args.verbose);
    }

    #[test]
    fn scan_severity_flag() {
        let cli = Cli::parse_from(["sentinel", "scan", "--severity", "high"]);
        let args = match cli.command {
            Commands::Scan(ref a) => a,
            _ => panic!("expected scan command"),
        };
        assert_eq!(args.severity.as_deref(), Some("high"));
    }

    #[test]
    fn scan_exclude_flag() {
        let cli = Cli::parse_from(["sentinel", "scan", "--exclude", "tests,examples"]);
        let args = match cli.command {
            Commands::Scan(ref a) => a,
            _ => panic!("expected scan command"),
        };
        assert_eq!(args.exclude.as_deref(), Some("tests,examples"));
    }

    #[test]
    fn scan_all_flags_set() {
        let cli = Cli::parse_from([
            "sentinel",
            "scan",
            "src",
            "--json",
            "--compact",
            "-q",
            "-v",
            "--severity",
            "medium",
            "--fail-on",
            "high",
            "--category",
            "security",
            "--rule",
            "missing-require-auth",
            "--exclude",
            "tests",
            "--timings",
            "--score",
            "--threads",
            "4",
            "--config",
            "sentinel.toml",
            "--color",
            "never",
        ]);
        let args = match cli.command {
            Commands::Scan(ref a) => a,
            _ => panic!("expected scan command"),
        };
        assert_eq!(args.path, "src");
        assert!(args.json);
        assert!(args.compact);
        assert!(args.quiet);
        assert!(args.verbose);
        assert!(args.timings);
        assert!(args.score);
        assert_eq!(args.threads, Some(4));
        assert_eq!(args.severity.as_deref(), Some("medium"));
        assert_eq!(args.fail_on.as_deref(), Some("high"));
        assert_eq!(args.category.as_deref(), Some("security"));
        assert_eq!(args.rule.as_deref(), Some("missing-require-auth"));
        assert_eq!(args.exclude.as_deref(), Some("tests"));
        assert_eq!(args.config.as_deref(), Some("sentinel.toml"));
        assert_eq!(args.color, "never");
    }
}
