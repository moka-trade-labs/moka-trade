//! Upstream pins (spec EXE-01). Must match `scripts/pins.env`; the
//! `moka-tests` crate checks that they do.

/// Percolator wrapper (`aeyakovenko/percolator-prog`) commit.
pub const WRAPPER_COMMIT: &str = "5cb331dde354517c6371a8acf92cecb194f3bb73";
/// Percolator engine (`aeyakovenko/percolator`) commit declared by the wrapper.
pub const ENGINE_COMMIT: &str = "4db11a8cb0053815e23a35d3a7d3edc265d8d866";
/// Test-only matcher (`aeyakovenko/percolator-match`) commit.
pub const MATCH_COMMIT: &str = "60aac3a996d0264c47fbefe9a2625d6ee6fb6a47";
/// Agave release providing `cargo-build-sbf`.
pub const AGAVE_VERSION: &str = "v3.0.10";
/// Solana platform-tools release used for SBF builds.
pub const PLATFORM_TOOLS_VERSION: &str = "v1.52";
