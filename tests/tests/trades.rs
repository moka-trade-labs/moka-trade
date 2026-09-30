//! Phase 2, issue #11: trades through `TradeNoCpi` and `TradeCpi` (via the
//! pinned `auth_matcher`), reconciled after every trade (CAP-13).
//!
//! Behaviour pinned here, each traced to the wrapper/engine source:
//! - `TradeNoCpi` charges the `fee_bps` both owners signed, to each side.
//! - `TradeCpi` charges only the market's `trade_fee_base_bps` (the LP does
//!   not sign CPI trades); the taker's `fee_bps` is only its consent bound.
//! - Trading fees are credited to the market's insurance, not to the maker.
//! - In AuthMark mode every fill settles at the mark: the engine uses the
//!   execution price only as fee notional (upstream `F-TRADENOCPI-FEE`), so a
//!   matcher spread moves no value.
//! - A maker position change outside a matcher fill revokes its matcher grant.

use moka_tests::artifacts_available;
use moka_tests::svm::{custom_error, err, Harness, Maker, MarketGroup, MarketProfile, Trader};
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
/// One contract in `size_q` units (`POS_SCALE`).
const CONTRACT: i128 = 1_000_000;
/// $1.00 in `_e6` price units.
const ONE_DOLLAR: u64 = 1_000_000;
/// Fee on 10 contracts at $1.00 and 10 bps: $10 x 0.001 = $0.01.
const FEE_10_CONTRACTS_10_BPS: u128 = 10_000;

struct World {
    h: Harness,
    g: MarketGroup,
    maker: Maker,
    alice: Trader,
}

/// A funded maker (1,000 USDC, matcher granted) and taker (100 USDC) in one
/// group whose mark is $1.00.
fn world(seed: u8, trade_fee_base_bps: u64) -> World {
    let mut h = Harness::new([seed; 32]).expect("harness");
    let profile = MarketProfile {
        trade_fee_base_bps,
        ..MarketProfile::default()
    };
    let g = h.init_market_group(profile).expect("InitMarket");
    h.warp_to_slot(1);
    h.configure_mark(&g, ONE_DOLLAR).expect("mark");
    let maker = h
        .setup_maker(&g, "maker", 1_000 * USDC, 10_000)
        .expect("maker");
    h.deposit(&g, &maker.trader, u128::from(1_000 * USDC))
        .expect("maker deposit");
    let alice = h.new_trader("alice", 100 * USDC).expect("alice");
    h.open_portfolio(&g, &alice).expect("open");
    h.deposit(&g, &alice, u128::from(100 * USDC))
        .expect("deposit");
    h.reconcile(&[&g]).expect("reconcile");
    World { h, g, maker, alice }
}

fn insurance(w: &World) -> u128 {
    w.h.market_state(&w.g).expect("state").1.insurance
}

/// EXE-02: a signed bilateral trade gives equal and opposite exposure, and
/// each side pays the `fee_bps` it signed, into insurance.
#[test]
fn exe_02_trade_no_cpi_equal_and_opposite_with_signed_fee() {
    require_artifacts!();
    let mut w = world(40, 0);
    let before = w.h.snapshot("funded", &[&w.g]).expect("snapshot");
    w.h.trade_no_cpi(
        &w.g,
        &w.alice,
        &w.maker.trader,
        10 * CONTRACT,
        ONE_DOLLAR,
        10,
    )
    .expect("TradeNoCpi");
    w.h.reconcile(&[&w.g]).expect("reconcile");
    println!(
        "{}",
        before.trace(&w.h.snapshot("TradeNoCpi", &[&w.g]).expect("snapshot"))
    );

    let a = w.h.position(&w.alice).expect("alice");
    let m = w.h.position(&w.maker.trader).expect("maker");
    assert_eq!(a.size_q, 10 * CONTRACT);
    assert_eq!(m.size_q, -a.size_q, "equal and opposite");
    assert_eq!(a.capital, u128::from(100 * USDC) - FEE_10_CONTRACTS_10_BPS);
    assert_eq!(
        m.capital,
        u128::from(1_000 * USDC) - FEE_10_CONTRACTS_10_BPS
    );
    assert_eq!(insurance(&w), 2 * FEE_10_CONTRACTS_10_BPS);
}

/// EXE-04: the MVP path. Only the taker signs; the matcher fills; exposure
/// is equal and opposite and both sides pay the market base fee.
#[test]
fn exe_04_trade_cpi_equal_and_opposite_with_base_fee() {
    require_artifacts!();
    let mut w = world(41, 10);
    w.h.trade_cpi(&w.g, &w.alice, &w.maker, 10 * CONTRACT, 10, 2 * ONE_DOLLAR)
        .expect("TradeCpi");
    w.h.reconcile(&[&w.g]).expect("reconcile");
    let a = w.h.position(&w.alice).expect("alice");
    let m = w.h.position(&w.maker.trader).expect("maker");
    assert_eq!(a.size_q, 10 * CONTRACT);
    assert_eq!(m.size_q, -a.size_q, "equal and opposite");
    assert_eq!(a.capital, u128::from(100 * USDC) - FEE_10_CONTRACTS_10_BPS);
    assert_eq!(
        m.capital,
        u128::from(1_000 * USDC) - FEE_10_CONTRACTS_10_BPS
    );
    assert_eq!(insurance(&w), 2 * FEE_10_CONTRACTS_10_BPS);

    // Closing through the matcher returns both to flat.
    w.h.trade_cpi(&w.g, &w.alice, &w.maker, -10 * CONTRACT, 10, 0)
        .expect("close");
    w.h.reconcile(&[&w.g]).expect("reconcile");
    assert_eq!(w.h.position(&w.alice).expect("alice").size_q, 0);
    assert_eq!(w.h.position(&w.maker.trader).expect("maker").size_q, 0);
}

/// EXE-04 / upstream INV-047: `TradeCpi` ignores the taker's `fee_bps` above
/// the base fee. With a 0 base fee, a 50 bps consent charges nothing. This is
/// why CPI and NoCpi routes "diverge" on fees: by design, not a bug.
#[test]
fn exe_04_trade_cpi_charges_only_the_market_base_fee() {
    require_artifacts!();
    let mut w = world(42, 0);
    w.h.trade_cpi(&w.g, &w.alice, &w.maker, 10 * CONTRACT, 50, 2 * ONE_DOLLAR)
        .expect("TradeCpi");
    w.h.reconcile(&[&w.g]).expect("reconcile");
    assert_eq!(
        w.h.position(&w.alice).expect("alice").capital,
        u128::from(100 * USDC)
    );
    assert_eq!(insurance(&w), 0);
}

/// EXE-04: a taker cannot consent to less than the market base fee.
#[test]
fn exe_04_trade_cpi_rejects_fee_consent_below_base() {
    require_artifacts!();
    let mut w = world(43, 10);
    let result =
        w.h.trade_cpi(&w.g, &w.alice, &w.maker, 10 * CONTRACT, 5, 2 * ONE_DOLLAR);
    assert_eq!(
        custom_error(&result),
        Some(err::INVALID_INSTRUCTION),
        "{result:?}"
    );
    assert_eq!(w.h.position(&w.alice).expect("alice").size_q, 0);
}

/// EXE-04: the matcher's price must respect the taker's limit.
#[test]
fn exe_04_trade_cpi_enforces_the_taker_limit_price() {
    require_artifacts!();
    let mut w = world(44, 0);
    w.h.set_matcher_spreads(&w.maker, 0, 100)
        .expect("1% ask spread");
    let result =
        w.h.trade_cpi(&w.g, &w.alice, &w.maker, 10 * CONTRACT, 0, 1_005_000);
    assert_eq!(
        custom_error(&result),
        Some(err::INVALID_INSTRUCTION),
        "{result:?}"
    );
    w.h.trade_cpi(&w.g, &w.alice, &w.maker, 10 * CONTRACT, 0, 1_010_000)
        .expect("limit at the ask fills");
}

/// Design fact for Phase 3/4 (spec §8.3): fills settle at the mark. Buying
/// at a 1% premium and selling at the mark costs the taker nothing but
/// fees, so a matcher spread transfers no value to the maker in AuthMark
/// mode. If upstream changes this, this test fails and the LP economics
/// must be revisited.
#[test]
fn design_fills_settle_at_mark_so_matcher_spread_moves_no_value() {
    require_artifacts!();
    let mut w = world(45, 0);
    w.h.set_matcher_spreads(&w.maker, 0, 100)
        .expect("1% ask spread");
    w.h.trade_cpi(&w.g, &w.alice, &w.maker, 10 * CONTRACT, 0, 1_010_000)
        .expect("buy at $1.01");
    w.h.trade_cpi(&w.g, &w.alice, &w.maker, -10 * CONTRACT, 0, 0)
        .expect("sell at $1.00");
    w.h.reconcile(&[&w.g]).expect("reconcile");
    let a = w.h.position(&w.alice).expect("alice");
    let m = w.h.position(&w.maker.trader).expect("maker");
    assert_eq!((a.size_q, a.capital, a.pnl), (0, u128::from(100 * USDC), 0));
    assert_eq!(
        (m.size_q, m.capital, m.pnl),
        (0, u128::from(1_000 * USDC), 0)
    );
}

/// A maker position change outside a matcher fill revokes its matcher grant
/// until the owner renews it (upstream fixes #412/#414). The LP vault must
/// re-grant after any such change (Phase 4).
#[test]
fn sec_non_matcher_position_change_revokes_matcher_grant() {
    require_artifacts!();
    let mut w = world(46, 0);
    w.h.trade_no_cpi(&w.g, &w.alice, &w.maker.trader, CONTRACT, ONE_DOLLAR, 0)
        .expect("TradeNoCpi");
    let result =
        w.h.trade_cpi(&w.g, &w.alice, &w.maker, CONTRACT, 0, 2 * ONE_DOLLAR);
    assert_eq!(custom_error(&result), Some(err::UNAUTHORIZED), "{result:?}");
    w.h.grant_matcher(&w.g, &w.maker, 10_000).expect("re-grant");
    w.h.trade_cpi(&w.g, &w.alice, &w.maker, CONTRACT, 0, 2 * ONE_DOLLAR)
        .expect("trades again after re-grant");
    w.h.reconcile(&[&w.g]).expect("reconcile");
}

/// Only the owner of the taker portfolio can trade it through the matcher.
#[test]
fn sec_trade_cpi_requires_the_taker_owner() {
    require_artifacts!();
    let mut w = world(47, 0);
    let mallory = w.h.new_trader("mallory", 0).expect("attacker");
    let (a_id, _, a_epoch) = w.h.portfolio_bindings(&w.alice).expect("alice");
    let (b_id, b_seq, b_epoch) = w.h.portfolio_bindings(&w.maker.trader).expect("maker");
    let market_id = w.h.market_state(&w.g).expect("state").1.assets[0].market_id;
    let result = w.h.send_wrapper(
        WrapperIx::TradeCpi {
            account_a_portfolio_id: a_id,
            account_a_position_epoch: a_epoch,
            account_b_portfolio_id: b_id,
            account_b_position_epoch: b_epoch,
            account_b_matcher_sequence: b_seq,
            asset_index: 0,
            market_id,
            size_q: 10 * CONTRACT,
            fee_bps: 0,
            limit_price: 2 * ONE_DOLLAR,
            backing_fee_cap_bps: 0,
        },
        vec![
            AccountMeta::new(mallory.signer.pubkey(), true),
            AccountMeta::new(w.g.market, false),
            AccountMeta::new(w.alice.portfolio.pubkey(), false),
            AccountMeta::new(w.maker.trader.portfolio.pubkey(), false),
            AccountMeta::new_readonly(w.h.matcher_program, false),
            AccountMeta::new(w.maker.matcher_context, false),
            AccountMeta::new_readonly(w.maker.matcher_delegate, false),
        ],
        &[&mallory.signer],
    );
    assert_eq!(custom_error(&result), Some(err::UNAUTHORIZED), "{result:?}");
    assert_eq!(w.h.position(&w.alice).expect("alice").size_q, 0);
}
