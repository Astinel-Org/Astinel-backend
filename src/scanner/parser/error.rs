use std::path::PathBuf;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ParserError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("invalid project at `{path}`: {detail}")]
    InvalidProject { path: PathBuf, detail: String },

    #[error("parse error in `{path}`: {detail}")]
    ParseError { path: PathBuf, detail: String },

    #[error("unsupported syntax in `{path}`: {detail}")]
    UnsupportedSyntax { path: PathBuf, detail: String },
}

impl From<std::io::Error> for ParserError {
    fn from(err: std::io::Error) -> Self {
        ParserError::Io(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_io_error() {
        let err: ParserError =
            std::io::Error::new(std::io::ErrorKind::NotFound, "file not found").into();
        assert!(err.to_string().contains("I/O error"));
    }

    #[test]
    fn display_invalid_project() {
        let err = ParserError::InvalidProject {
            path: PathBuf::from("/bad/path"),
            detail: "missing Cargo.toml".into(),
        };
        assert!(err.to_string().contains("/bad/path"));
        assert!(err.to_string().contains("missing Cargo.toml"));
    }

    #[test]
    fn clone() {
        let err = ParserError::InvalidProject {
            path: PathBuf::from("x"),
            detail: "test".into(),
        };
        let cloned = err.clone();
        assert!(matches!(cloned, ParserError::InvalidProject { .. }));
    }
}
