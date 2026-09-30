//! Spec EXE-01 / TEST-08: the pins in code, scripts and the pin manifest agree.

use moka_tests::{artifacts_dir, load_pinned_program, pinned_hashes, repo_root};
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
    if !artifacts_dir().exists() {
        eprintln!("skipped: vendor/artifacts missing; run scripts/build-upstream.sh");
        return;
    }
    for name in pinned_hashes().expect("pin manifest").keys() {
        load_pinned_program(name).expect(name);
    }
}
