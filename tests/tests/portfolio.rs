//! Phase 2, issue #10: portfolio lifecycle with reconciliation after every
//! step (CAP-13), the rent disclosure facts (API-06A) and the safety
//! properties of the deposit/withdraw/close routes (SEC-09).

use moka_tests::artifacts_available;
use moka_tests::svm::{custom_error, err, Harness, MarketGroup, MarketProfile, Trader};
use percolator_prog::ix::Instruction as WrapperIx;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::signature::Signer;

macro_rules! require_artifacts {
    () => {
        if !artifacts_available() {
            return;
        }
    };
}

/// 1 USDC in atoms (6 decimals).
const USDC: u64 = 1_000_000;

fn setup(seed: u8, wallet: u64) -> (Harness, MarketGroup, Trader) {
    let mut h = Harness::new([seed; 32]).expect("harness");
    let group = h
        .init_market_group(MarketProfile::default())
        .expect("InitMarket");
    let alice = h.new_trader("alice", wallet).expect("trader");
    h.open_portfolio(&group, &alice).expect("open portfolio");
    (h, group, alice)
}

/// Asserts the wallet, vault and engine capital all agree after a step.
fn assert_step(h: &Harness, g: &MarketGroup, t: &Trader, wallet: u64, capital: u64) {
    h.reconcile(&[g]).expect("reconcile");
    assert_eq!(h.token_balance(&t.usdc).expect("wallet"), wallet, "wallet");
    assert_eq!(
        h.portfolio_state(t).expect("portfolio").1,
        u128::from(capital),
        "capital"
    );
    assert_eq!(h.token_balance(&g.vault).expect("vault"), capital, "vault");
}

/// CAP-13: open → deposit → partial withdraw → full withdraw → close, with
/// every balance reconciled after each step. `--nocapture` prints the trace.
#[test]
fn cap_13_portfolio_lifecycle_reconciles() {
    require_artifacts!();
    let mut h = Harness::new([20; 32]).expect("harness");
    let g = h
        .init_market_group(MarketProfile::default())
        .expect("InitMarket");
    let alice = h.new_trader("alice", 1_000 * USDC).expect("trader");
    let mut last = h.snapshot("funded", &[&g]).expect("snapshot");
    let mut step = |h: &Harness, label: &str| {
        let now = h.snapshot(label, &[&g]).expect("snapshot");
        println!("{}", last.trace(&now));
        last = now;
    };

    h.open_portfolio(&g, &alice).expect("open");
    let (owner, capital) = h.portfolio_state(&alice).expect("state");
    assert_eq!(owner, alice.signer.pubkey().to_bytes());
    assert_eq!(capital, 0);
    assert_step(&h, &g, &alice, 1_000 * USDC, 0);
    step(&h, "InitPortfolio");

    h.deposit(&g, &alice, u128::from(600 * USDC))
        .expect("deposit");
    assert_step(&h, &g, &alice, 400 * USDC, 600 * USDC);
    step(&h, "Deposit 600");

    h.withdraw(&g, &alice, u128::from(250 * USDC))
        .expect("withdraw");
    assert_step(&h, &g, &alice, 650 * USDC, 350 * USDC);
    step(&h, "Withdraw 250");

    h.withdraw(&g, &alice, u128::from(350 * USDC))
        .expect("withdraw rest");
    assert_step(&h, &g, &alice, 1_000 * USDC, 0);
    step(&h, "Withdraw 350");

    h.close_portfolio(&g, &alice).expect("close");
    h.reconcile(&[&g]).expect("reconcile after close");
    assert!(
        h.svm
            .get_account(&alice.portfolio.pubkey())
            .is_none_or(|a| a.data.is_empty()),
        "closed portfolio must hold no data"
    );
    step(&h, "ClosePortfolio");
}

/// API-06A: a portfolio is 9,563 bytes, its rent is paid by the trader, and
/// `ClosePortfolio` moves all of it to the market account, not back to the
/// trader. This is the fact the rent disclosure must state.
#[test]
fn api_06a_portfolio_rent_is_paid_by_trader_and_swept_to_market() {
    require_artifacts!();
    assert_eq!(Harness::portfolio_len().expect("len"), 9_563);
    let mut h = Harness::new([21; 32]).expect("harness");
    let g = h
        .init_market_group(MarketProfile::default())
        .expect("InitMarket");
    let alice = h.new_trader("alice", 0).expect("trader");
    let rent = h.portfolio_rent().expect("rent");
    println!("portfolio rent: {rent} lamports");
    // ~0.067 SOL at the default rent rate (HANDOFF §4).
    assert!((67_000_000..68_000_000).contains(&rent), "rent {rent}");

    h.open_portfolio(&g, &alice).expect("open");
    assert_eq!(h.lamports(&alice.portfolio.pubkey()), rent);

    let market_before = h.lamports(&g.market);
    let trader_before = h.lamports(&alice.signer.pubkey());
    h.close_portfolio(&g, &alice).expect("close");
    assert_eq!(h.lamports(&alice.portfolio.pubkey()), 0);
    assert_eq!(
        h.lamports(&g.market),
        market_before + rent,
        "rent swept to market"
    );
    // The harness payer pays fees, so the trader's SOL is unchanged: no refund.
    assert_eq!(
        h.lamports(&alice.signer.pubkey()),
        trader_before,
        "no rent refund"
    );
}

/// CAP-13: over-withdrawal fails and moves nothing.
#[test]
fn cap_13_withdraw_more_than_capital_fails_without_side_effects() {
    require_artifacts!();
    let (mut h, g, alice) = setup(22, 100 * USDC);
    h.deposit(&g, &alice, u128::from(100 * USDC))
        .expect("deposit");
    let before = h.snapshot("before", &[&g]).expect("snapshot");
    let result = h.withdraw(&g, &alice, u128::from(100 * USDC + 1));
    assert_eq!(
        custom_error(&result),
        Some(err::ENGINE_LOCK_ACTIVE),
        "{result:?}"
    );
    let after = h.snapshot("after", &[&g]).expect("snapshot");
    assert_eq!(before.tokens, after.tokens);
    assert_eq!(before.ledgers, after.ledgers);
    h.reconcile(&[&g]).expect("reconcile");
}

/// Funds cannot be stranded: a portfolio with capital cannot be closed.
#[test]
fn cap_13_close_with_capital_is_rejected() {
    require_artifacts!();
    let (mut h, g, alice) = setup(23, 10 * USDC);
    h.deposit(&g, &alice, u128::from(10 * USDC))
        .expect("deposit");
    let result = h.close_portfolio(&g, &alice);
    // The engine refuses to deregister a non-empty portfolio with LockActive.
    assert_eq!(
        custom_error(&result),
        Some(err::ENGINE_LOCK_ACTIVE),
        "{result:?}"
    );
    assert_step(&h, &g, &alice, 0, 10 * USDC);
}

/// Only the portfolio owner can withdraw: a third party signing with its own
/// wallet as the destination is rejected as Unauthorized.
#[test]
fn sec_withdraw_requires_the_portfolio_owner() {
    require_artifacts!();
    let (mut h, g, alice) = setup(24, 50 * USDC);
    h.deposit(&g, &alice, u128::from(50 * USDC))
        .expect("deposit");
    let mallory = h.new_trader("mallory", 0).expect("attacker");
    let (portfolio_id, expected_sequence, _) = h.portfolio_bindings(&alice).expect("bindings");
    let result = h.send_wrapper(
        WrapperIx::Withdraw {
            portfolio_id,
            expected_sequence,
            amount: u128::from(50 * USDC),
        },
        vec![
            AccountMeta::new(mallory.signer.pubkey(), true),
            AccountMeta::new(g.market, false),
            AccountMeta::new(alice.portfolio.pubkey(), false),
            AccountMeta::new(mallory.usdc, false),
            AccountMeta::new(g.vault, false),
            AccountMeta::new_readonly(g.vault_authority, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        &[&mallory.signer],
    );
    assert_eq!(custom_error(&result), Some(err::UNAUTHORIZED), "{result:?}");
    assert_eq!(h.token_balance(&mallory.usdc).expect("mallory"), 0);
    assert_step(&h, &g, &alice, 0, 50 * USDC);
}

/// SEC-09: a deposit through market A cannot land in market B's vault.
#[test]
fn sec_09_deposit_cannot_target_another_groups_vault() {
    require_artifacts!();
    let (mut h, a, alice) = setup(25, 20 * USDC);
    let b = h.init_market_group(MarketProfile::default()).expect("B");
    let (portfolio_id, expected_sequence, _) = h.portfolio_bindings(&alice).expect("bindings");
    let result = h.send_wrapper(
        WrapperIx::Deposit {
            portfolio_id,
            expected_sequence,
            amount: u128::from(20 * USDC),
        },
        vec![
            AccountMeta::new(alice.signer.pubkey(), true),
            AccountMeta::new(a.market, false),
            AccountMeta::new(alice.portfolio.pubkey(), false),
            AccountMeta::new(alice.usdc, false),
            AccountMeta::new(b.vault, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        &[&alice.signer],
    );
    assert_eq!(
        custom_error(&result),
        Some(err::INVALID_VAULT_ACCOUNT),
        "{result:?}"
    );
    h.reconcile(&[&a, &b]).expect("reconcile");
    assert_eq!(h.token_balance(&b.vault).expect("B vault"), 0);
}
