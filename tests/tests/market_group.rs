//! Phase 2, issue #9: the harness loads the pinned programs and creates one
//! isolated single-asset market group (TEST-02, SEC-09).

use moka_tests::artifacts_available;
use moka_tests::svm::{Harness, MarketProfile};
use solana_sdk::signature::Signer;

macro_rules! require_artifacts {
    () => {
        if !artifacts_available() {
            return;
        }
    };
}

/// A single-asset market group account is 3,003 bytes (HANDOFF §4).
#[test]
fn sec_09_single_asset_market_group_account_len() {
    let len = percolator_prog::state::market_account_len_for_capacity(1).expect("len");
    assert_eq!(len, 3_003);
}

#[test]
fn test_02_init_market_creates_single_asset_group() {
    require_artifacts!();
    let mut h = Harness::new([7; 32]).expect("harness");
    let group = h
        .init_market_group(MarketProfile::default())
        .expect("InitMarket");

    let (config, engine) = h.market_state(&group).expect("decode");
    assert_eq!(config.marketauth, group.admin.pubkey().to_bytes());
    assert_eq!(config.collateral_mint, h.usdc_mint.to_bytes());
    assert_eq!(engine.vault, 0, "no deposits yet");
    assert_eq!(engine.insurance, 0);
    assert_eq!(h.token_balance(&group.vault).expect("vault"), 0);
}

/// Program-state check only: each `InitMarket` records its own group id and
/// authority. The balance-level isolation test (a loss in A leaves B
/// unchanged) is issue #14.
#[test]
fn sec_09_market_groups_have_distinct_ids_and_authorities() {
    require_artifacts!();
    let mut h = Harness::new([8; 32]).expect("harness");
    let a = h.init_market_group(MarketProfile::default()).expect("A");
    let b = h.init_market_group(MarketProfile::default()).expect("B");
    let (ca, ea) = h.market_state(&a).expect("A state");
    let (cb, eb) = h.market_state(&b).expect("B state");
    assert_ne!(ea.market_group_id, eb.market_group_id);
    assert_eq!(ca.marketauth, a.admin.pubkey().to_bytes());
    assert_eq!(cb.marketauth, b.admin.pubkey().to_bytes());
}

#[test]
fn test_02_init_market_is_one_shot() {
    require_artifacts!();
    let mut h = Harness::new([9; 32]).expect("harness");
    let group = h
        .init_market_group(MarketProfile::default())
        .expect("InitMarket");
    // Re-initializing a live market must fail, even for its own admin.
    let result = h.send_init_market(group.market, &group.admin, MarketProfile::default());
    let err = result.expect_err("second InitMarket on a live market succeeded");
    // PercolatorError::AlreadyInitialized = 2; instruction 3 follows the
    // three compute-budget instructions the harness prepends.
    assert!(
        err.starts_with("InstructionError(3, Custom(2))"),
        "unexpected error: {err}"
    );
}

/// CAP-13: InitMarket moves no tokens, and every reconciliation rule holds
/// for a fresh group. Run with `--nocapture` to see the trace.
#[test]
fn cap_13_reconciles_across_init_market() {
    require_artifacts!();
    let mut h = Harness::new([10; 32]).expect("harness");
    let before = h.snapshot("start", &[]).expect("snapshot");
    let group = h
        .init_market_group(MarketProfile::default())
        .expect("InitMarket");
    h.reconcile(&[&group]).expect("reconcile");
    let after = h.snapshot("InitMarket", &[&group]).expect("snapshot");
    println!("{}", before.trace(&after));
    assert_eq!(after.mint_supply, 0);
    assert!(after.tokens.values().all(|&b| b == 0));
}

/// CAP-13: the custody rule is not vacuous. Tokens that reach the vault
/// without going through the program (a donation) must fail reconciliation.
#[test]
fn cap_13_reconcile_detects_vault_drift() {
    require_artifacts!();
    let mut h = Harness::new([11; 32]).expect("harness");
    let group = h
        .init_market_group(MarketProfile::default())
        .expect("InitMarket");
    let name = "vault (donated)";
    h.set_token_account(name, group.vault, group.vault_authority, 5)
        .expect("donate");
    let err = h.reconcile(&[&group]).expect_err("drift not detected");
    assert!(err.starts_with("custody:"), "{err}");
}
