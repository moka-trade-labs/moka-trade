//! Test support: locating and verifying the pinned upstream artifacts that
//! `scripts/build-upstream.sh` writes to `vendor/artifacts/`, plus the LiteSVM
//! harness in [`svm`].

/// True when `scripts/build-upstream.sh` has produced the program binaries.
/// Tests that need them return early (with a note) when this is false, so
/// `cargo test` still works on machines without the SBF toolchain.
pub fn artifacts_available() -> bool {
    artifacts_dir().join("percolator_prog.so").exists()
}

pub mod svm;

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Repository root (this crate lives in `<root>/tests`).
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

pub fn artifacts_dir() -> PathBuf {
    repo_root().join("vendor").join("artifacts")
}

/// Parses `scripts/upstream-hashes.sha256` (`sha256sum` format) into
/// file name -> hex digest.
pub fn pinned_hashes() -> Result<BTreeMap<String, String>, String> {
    let path = repo_root().join("scripts").join("upstream-hashes.sha256");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let (hash, name) = line
                .split_once("  ")
                .ok_or_else(|| format!("bad line in hash file: {line}"))?;
            Ok((name.trim().to_owned(), hash.trim().to_owned()))
        })
        .collect()
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Loads a pinned program binary, failing if it is missing or its hash does
/// not match the pin manifest. Tests must load programs only through this.
pub fn load_pinned_program(name: &str) -> Result<Vec<u8>, String> {
    let expected = pinned_hashes()?
        .remove(name)
        .ok_or_else(|| format!("{name} is not in the pin manifest"))?;
    let path = artifacts_dir().join(name);
    let bytes = fs::read(&path)
        .map_err(|e| format!("{}: {e} (run scripts/build-upstream.sh)", path.display()))?;
    let actual = sha256_hex(&bytes);
    if actual != expected {
        return Err(format!("{name}: sha256 {actual}, pinned {expected}"));
    }
    Ok(bytes)
}
