//! Locate external tools on `$PATH`.
//!
//! The slurp/grim pipeline depends on a few binaries that may not be
//! installed. Centralising the lookup means we can give the user a
//! single, clear "you need to install X" message at startup rather
//! than discovering the problem mid-capture.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WhichError {
    #[error("required tool not found on $PATH: {0}")]
    NotFound(String),
}

/// Find `program` on `$PATH`. Returns the absolute path on success.
///
/// Uses [`which`](https://crates.io/crates/which) semantics: searches
/// each entry of `$PATH` in order and resolves the first executable hit.
/// We avoid pulling the `which` crate for one function — a 60-line
/// reimplementation that respects `$PATH` is enough for v1.
pub fn which<S: AsRef<str>>(program: S) -> Result<PathBuf, WhichError> {
    let program = program.as_ref();

    if program.contains('/') {
        // Caller passed a path — trust it, but only if it's executable.
        let p = PathBuf::from(program);
        if is_executable(&p) {
            return Ok(p);
        }
        return Err(WhichError::NotFound(program.to_string()));
    }

    let path_var =
        std::env::var_os("PATH").ok_or_else(|| WhichError::NotFound(program.to_string()))?;

    for entry in std::env::split_paths(&path_var) {
        let candidate = entry.join(program);
        if is_executable(&candidate) {
            return Ok(candidate);
        }
    }

    Err(WhichError::NotFound(program.to_string()))
}

fn is_executable(p: &std::path::Path) -> bool {
    let meta = match std::fs::metadata(p) {
        Ok(m) => m,
        Err(_) => return false,
    };
    if !meta.is_file() {
        return false;
    }
    // On Unix, check the owner-execute bit. We deliberately ignore
    // group/other to keep the check fast and simple — `execvp` will
    // surface the real permission error at spawn time.
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o100 != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_existing_tool() {
        // /bin/sh is on every Linux box; it's the most portable choice.
        let p = which("sh").expect("sh must be on PATH");
        assert!(p.is_absolute());
    }

    #[test]
    fn missing_tool_is_error() {
        let err = which("definitely-not-a-real-binary-pixelens-test").unwrap_err();
        assert!(matches!(err, WhichError::NotFound(_)));
    }

    #[test]
    fn absolute_path_must_exist_and_be_executable() {
        assert!(which("/bin/sh").is_ok());
        assert!(which("/nonexistent/path/to/binary").is_err());
    }
}
