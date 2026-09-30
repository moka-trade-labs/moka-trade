//! Spec EXE-01 / TEST-08: the pins in code, scripts and the pin manifest agree.

use moka_tests::{artifacts_available, load_pinned_program, pinned_hashes, repo_root};
use moka_types::pins;
use std::fs;

fn pins_env(key: &str) -> String {
    let text = fs::read_to_string(repo_root().join("scripts/pins.env")).unwrap_or_default();
    text.lines()
        .find_map(|l| l.strip_prefix(&format!("{key}=")))
        .map(str::to_owned)
        .unwrap_or_default()
}

#[test]
fn exe_01_pins_match_scripts() {
    assert_eq!(pins_env("WRAPPER_COMMIT"), pins::WRAPPER_COMMIT);
    assert_eq!(pins_env("ENGINE_COMMIT"), pins::ENGINE_COMMIT);
    assert_eq!(pins_env("MATCH_COMMIT"), pins::MATCH_COMMIT);
    assert_eq!(pins_env("AGAVE_VERSION"), pins::AGAVE_VERSION);
    assert_eq!(
        pins_env("PLATFORM_TOOLS_VERSION"),
        pins::PLATFORM_TOOLS_VERSION
    );
}

/// The wrapper crate used for instruction encoding is the pinned commit.
#[test]
fn exe_01_test_crate_git_deps_use_pins() {
    let manifest = fs::read_to_string(repo_root().join("tests/Cargo.toml")).unwrap_or_default();
    for (repo, rev) in [
        ("percolator-prog", pins::WRAPPER_COMMIT),
        ("percolator", pins::ENGINE_COMMIT),
    ] {
        let line = manifest
            .lines()
            .find(|l| l.starts_with(&format!("{repo} = ")))
            .unwrap_or_default();
        assert!(line.contains(&format!("rev = \"{rev}\"")), "{repo}: {line}");
    }
}

#[test]
fn test_08_manifest_lists_every_program() {
    let hashes = pinned_hashes().expect("pin manifest");
    for name in [
        "percolator_prog.so",
        "auth_matcher.so",
        "hostile_matcher.so",
        "percolator_match.so",
    ] {
        let hash = hashes.get(name).expect(name);
        assert_eq!(hash.len(), 64, "{name}");
    }
}

/// Runs only after `scripts/build-upstream.sh`; skipped (with a note) otherwise,
/// so `cargo test` works on machines without the SBF toolchain.
#[test]
fn test_08_built_artifacts_match_pins() {
    if !artifacts_available() {
        return;
    }
    for name in pinned_hashes().expect("pin manifest").keys() {
        load_pinned_program(name).expect(name);
    }
}
