use std::io;
use std::path::PathBuf;

use crate::exit::ExitCode;

/// Typed errors for the Sentinel CLI.
///
/// Every error variant maps to a deterministic exit code.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// I/O error (file read/write, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Invalid path provided by the user
    #[error("Invalid path: `{path}` — {detail}")]
    InvalidPath { path: PathBuf, detail: String },

    /// Source code parsing failure
    #[error("Parse error: {0}")]
    Parse(#[from] sentinel_parser::ParserError),

    /// Configuration file error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Invalid CLI arguments
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    /// Output/rendering error
    #[error("Output error: {0}")]
    Output(String),

    /// Permission denied accessing a path
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Internal/unexpected error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Unsupported project type
    #[error("Unsupported project: {0}")]
    Unsupported(String),
}

impl CliError {
    /// Map this error to its deterministic exit code.
    pub fn exit_code(&self) -> ExitCode {
        match self {
            CliError::InvalidArguments(_) => ExitCode::InvalidArguments,
            CliError::Config(_) => ExitCode::InvalidConfiguration,
            CliError::Parse(_) => ExitCode::ParseFailure,
            CliError::InvalidPath { .. } => ExitCode::ProjectNotFound,
            CliError::Unsupported(_) => ExitCode::UnsupportedProject,
            CliError::Io(e) => match e.kind() {
                io::ErrorKind::PermissionDenied => ExitCode::PermissionDenied,
                io::ErrorKind::NotFound => ExitCode::ProjectNotFound,
                _ => ExitCode::InternalError,
            },
            CliError::PermissionDenied(_) => ExitCode::PermissionDenied,
            CliError::Output(_) | CliError::Internal(_) => ExitCode::InternalError,
        }
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        CliError::Output(e.to_string())
    }
}

impl From<crate::errors::CliError> for io::Error {
    fn from(e: crate::errors::CliError) -> Self {
        io::Error::other(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_maps_to_expected_exit_code() {
        assert_eq!(
            CliError::InvalidArguments("bad".into()).exit_code(),
            ExitCode::InvalidArguments
        );
        assert_eq!(
            CliError::Config("bad".into()).exit_code(),
            ExitCode::InvalidConfiguration
        );
        assert_eq!(CliError::Internal("bug".into()).exit_code(), ExitCode::InternalError);
        assert_eq!(
            CliError::Parse(sentinel_parser::ParserError::InvalidProject {
                path: PathBuf::from("x"),
                detail: "x".into()
            })
            .exit_code(),
            ExitCode::ParseFailure
        );
        assert_eq!(
            CliError::Unsupported("no".into()).exit_code(),
            ExitCode::UnsupportedProject
        );
        assert_eq!(
            CliError::PermissionDenied("no".into()).exit_code(),
            ExitCode::PermissionDenied
        );
    }

    #[test]
    fn io_error_kind_maps_correctly() {
        let not_found = CliError::Io(io::Error::new(io::ErrorKind::NotFound, "no"));
        assert_eq!(not_found.exit_code(), ExitCode::ProjectNotFound);

        let denied = CliError::Io(io::Error::new(io::ErrorKind::PermissionDenied, "no"));
        assert_eq!(denied.exit_code(), ExitCode::PermissionDenied);

        let other = CliError::Io(io::Error::other("no"));
        assert_eq!(other.exit_code(), ExitCode::InternalError);
    }

    #[test]
    fn output_error_maps_to_internal() {
        let err = CliError::Output("write failed".into());
        assert_eq!(err.exit_code(), ExitCode::InternalError);
    }

    #[test]
    fn invalid_path_maps_to_not_found() {
        let err = CliError::InvalidPath {
            path: PathBuf::from("/x"),
            detail: "missing".into(),
        };
        assert_eq!(err.exit_code(), ExitCode::ProjectNotFound);
    }

    #[test]
    fn serde_json_error_converts_to_output() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let cli_err: CliError = json_err.into();
        assert!(matches!(cli_err, CliError::Output(_)));
    }

    #[test]
    fn cli_error_converts_to_io_error() {
        let cli_err = CliError::Internal("test".into());
        let io_err: io::Error = cli_err.into();
        assert_eq!(io_err.kind(), io::ErrorKind::Other);
    }
}
