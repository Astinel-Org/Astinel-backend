use clap::Parser;

use crate::commands::Cli;
use crate::exit::ExitCode;

/// Run the CLI application, returning the appropriate exit code.
///
/// Parses CLI arguments, dispatches to the appropriate command handler,
/// and handles errors with structured output.
pub fn run() -> ExitCode {
    sentinel_utils::logging::init_logging();
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::InvalidArguments;
        }
    };

    let result = crate::commands::dispatch(cli);

    match result {
        Ok(exit_code) => exit_code,
        Err(err) => {
            let exit_code = err.exit_code();
            match exit_code {
                ExitCode::InternalError => {
                    eprintln!("Error: {} (This is a bug. Please report it.)", err);
                }
                _ => {
                    eprintln!("Error: {}", err);
                }
            }
            exit_code
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_via_run() {
        let cli = Cli::parse_from(["sentinel", "version"]);
        let result = crate::commands::dispatch(cli);
        assert!(result.is_ok());
    }
}
