use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::errors::CliError;

/// Resolve a scan path to its canonical form, validating it exists.
pub fn resolve_scan_path(path: &Path) -> Result<PathBuf, CliError> {
    if !path.exists() {
        return Err(CliError::InvalidPath {
            path: path.to_path_buf(),
            detail: "path does not exist".to_string(),
        });
    }
    let canonical = path.canonicalize().map_err(CliError::Io)?;
    Ok(canonical)
}

/// Collect Rust source files under `root`, filtering by `ignore_paths`.
///
/// Hidden directories (`.` prefix), `target`, and `node_modules` are always skipped.
/// Additional ignore patterns from `ignore_paths` are matched as substrings of the
/// relative file path.
/// Collect Rust source files under `root`, filtering by `ignore_paths`.
///
/// Hidden directories (`.` prefix), `target`, and `node_modules` are always skipped.
/// Additional ignore patterns from `ignore_paths` are matched as substrings of the
/// relative file path.
pub fn collect_source_files(root: &Path, ignore_paths: &[String]) -> Vec<PathBuf> {
    if root.is_file() {
        if root.extension().is_some_and(|e| e == "rs") {
            return vec![root.to_path_buf()];
        }
        return Vec::new();
    }

    let mut files = Vec::new();
    'outer: for entry in WalkDir::new(root).into_iter() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(path);
        for component in rel.components() {
            let name = component.as_os_str().to_str().unwrap_or("");
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue 'outer;
            }
        }
        let rel_str = rel.to_string_lossy();
        for pattern in ignore_paths {
            if rel_str.contains(pattern) {
                continue 'outer;
            }
        }
        if entry.file_type().is_file() && path.extension().is_some_and(|e| e == "rs") {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    files
}

/// Walk up from `path` to find the project root (containing `Cargo.toml` or `sentinel.toml`).
pub fn discover_project_root(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        return path.parent().map(|p| p.to_path_buf());
    }
    let mut current = Some(path.to_path_buf());
    while let Some(dir) = current {
        if dir.join("Cargo.toml").exists() {
            return Some(dir);
        }
        if dir.join("sentinel.toml").exists() {
            return Some(dir);
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }
    None
}

/// Check if `path` is a Rust project (`.rs` file or `Cargo.toml` exists).
pub fn is_rust_project(path: &Path) -> bool {
    if path.is_file() && path.extension().is_some_and(|e| e == "rs") {
        return true;
    }
    path.join("Cargo.toml").exists()
}

/// Make `path` relative to `root`.
pub fn make_relative(path: &Path, root: &Path) -> PathBuf {
    if let Ok(rel) = path.strip_prefix(root) {
        rel.to_path_buf()
    } else {
        path.file_name()
            .map(PathBuf::from)
            .unwrap_or_else(|| path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn collect_files_in_directory() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "").unwrap();
        fs::write(dir.path().join("b.rs"), "").unwrap();
        fs::write(dir.path().join("readme.md"), "").unwrap();
        fs::create_dir(dir.path().join("target")).unwrap();
        fs::write(dir.path().join("target").join("lib.rs"), "").unwrap();

        let files = collect_source_files(dir.path(), &[]);
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.ends_with("a.rs")));
        assert!(files.iter().any(|f| f.ends_with("b.rs")));
    }

    #[test]
    fn collect_files_with_ignored_paths() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "").unwrap();
        fs::create_dir(dir.path().join("tests")).unwrap();
        fs::write(dir.path().join("tests").join("test.rs"), "").unwrap();
        fs::create_dir(dir.path().join("examples")).unwrap();
        fs::write(dir.path().join("examples").join("example.rs"), "").unwrap();

        let files = collect_source_files(dir.path(), &["tests".to_string()]);
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.ends_with("a.rs")));
        assert!(files.iter().any(|f| f.ends_with("example.rs")));
    }

    #[test]
    fn single_file_scan() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("contract.rs");
        fs::write(&file, "fn main() {}").unwrap();
        let files = collect_source_files(&file, &[]);
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn discover_project_root_with_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let sub = dir.path().join("src");
        fs::create_dir(&sub).unwrap();

        let root = discover_project_root(&sub).unwrap();
        assert_eq!(root, dir.path().canonicalize().unwrap());
    }

    #[test]
    fn resolve_valid_path() {
        let dir = tempfile::tempdir().unwrap();
        let r = resolve_scan_path(dir.path());
        assert!(r.is_ok());
    }

    #[test]
    fn resolve_invalid_path() {
        let r = resolve_scan_path(Path::new("/nonexistent/sentinel/test"));
        assert!(r.is_err());
    }

    #[test]
    fn make_relative_works() {
        let root = Path::new("/project");
        let abs = Path::new("/project/src/lib.rs");
        let rel = make_relative(abs, root);
        assert_eq!(rel, Path::new("src/lib.rs"));
    }
}
