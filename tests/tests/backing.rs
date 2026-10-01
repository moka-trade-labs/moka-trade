//! Phase 2, issue #12: the backing flow (BST-04/05/07) and owner decision 16,
//! reconciled after every step (CAP-13).
//!
//! Behaviour pinned here, each traced to the wrapper/engine source:
//! - `TopUpBackingBucket` moves provider USDC into the group vault as fresh
//!   backing for one domain (asset x 2 + side).
//! - A provider is paid by the **backing fee**: when a trade grows a lien on
//!   a domain's backing (a trader using backed winnings as margin), the
//!   trader pays `fee_bps` of the newly liened backing; `insurance_share_bps`
//!   of it goes to insurance, the rest to `utilization_fee_earnings`. It is
//!   not a share of ordinary trading fees.
//! - The per-slot utilization rate (BST-07 kink curve) is zero at the pin:
//!   `InitMarket` builds the engine config from `public_user_fund`, whose
//!   base rate and slopes are 0, and no wrapper instruction changes them.
//! - A portfolio holding any position cannot `Withdraw` (`EngineStale`).

use moka_tests::artifacts_available;
use moka_tests::svm::{
    custom_error, err, BackingProvider, Harness, Maker, MarketGroup, MarketProfile, Trader,
    DOMAIN_SHORT,
};

macro_rules! require_artifacts {
    () => {
        if !artifacts_available() {
            return;
        }
    };
}

const USDC: u64 = 1_000_000;
const CONTRACT: i128 = 1_000_000;
const ONE_DOLLAR: u64 = 1_000_000;
const MARK_UP_5: u64 = 1_050_000;
/// Backed-claim amounts are stored with 1e12 extra precision (`_num`).
const NUM_PER_ATOM: u128 = 1_000_000_000_000;
/// Policy used throughout: 50% of newly liened backing, 25% of that fee to
/// insurance (the values upstream's own conservation test uses).
const FEE_BPS: u16 = 5_000;
const INSURANCE_SHARE_BPS: u16 = 2_500;
/// `PermissionlessCrank` returns EngineNonProgress when there is nothing to do.
const ENGINE_NON_PROGRESS: u32 = 22;
/// `Withdraw` from a portfolio with an active leg (engine `withdraw_not_atomic`).
const ENGINE_STALE: u32 = 19;

struct World {
    h: Harness,
    g: MarketGroup,
    maker: Maker,
    alice: Trader,
    provider: BackingProvider,
}

/// Mark $1.00, backing fee policy on the short domain, a 10,000 USDC maker,
/// Alice with 100 USDC, and 1,000 USDC of short-domain backing.
fn world(seed: u8) -> World {
    let mut h = Harness::new([seed; 32]).expect("harness");
    let g = h
        .init_market_group(MarketProfile::default())
        .expect("InitMarket");
    h.warp_to_slot(1);
    h.configure_mark(&g, ONE_DOLLAR).expect("mark");
    h.set_backing_fee_policy(&g, DOMAIN_SHORT, FEE_BPS, INSURANCE_SHARE_BPS)
        .expect("backing fee policy");
    let maker = h
        .setup_maker(&g, "maker", 10_000 * USDC, 10_000)
        .expect("maker");
    h.deposit(&g, &maker.trader, u128::from(10_000 * USDC))
        .expect("maker deposit");
    let alice = h.new_trader("alice", 100 * USDC).expect("alice");
    h.open_portfolio(&g, &alice).expect("open");
    h.deposit(&g, &alice, u128::from(100 * USDC))
        .expect("deposit");
    let provider = h.new_backing_provider(&g, 1_000 * USDC).expect("provider");
    h.top_up_backing(&g, &provider, DOMAIN_SHORT, u128::from(1_000 * USDC), 1_000)
        .expect("top up");
    h.reconcile(&[&g]).expect("reconcile");
    World {
        h,
        g,
        maker,
        alice,
        provider,
    }
}

/// Crank a portfolio; "nothing to do" (EngineNonProgress) is fine.
fn crank(w: &mut World, which: &str) {
    let t = if which == "maker" {
        &w.maker.trader
    } else {
        &w.alice
    };
    let r = w.h.crank(&w.g, t);
    assert!(
        r.is_ok() || custom_error(&r) == Some(ENGINE_NON_PROGRESS),
        "crank {which}: {r:?}"
    );
}

/// Alice goes long 95 contracts at $1.00 (95 USDC of 100 USDC capital at
/// 100% margin), then the mark rises 5% one slot later: Alice has 4.75 USDC
/// of winnings backed by the short domain.
fn long_then_shock(w: &mut World) {
    w.h.trade_cpi(&w.g, &w.alice, &w.maker, 95 * CONTRACT, 0, 2 * ONE_DOLLAR)
        .expect("open long");
    w.h.warp_to_slot(2);
    w.h.push_mark(&w.g, MARK_UP_5).expect("push mark");
    crank(w, "maker");
    crank(w, "alice");
    w.h.reconcile(&[&w.g]).expect("reconcile");
}

fn short_bucket(w: &World) -> (u128, u128, u128) {
    let (_, e) = w.h.market_state(&w.g).expect("state");
    let b = &e.source_backing_buckets[usize::from(DOMAIN_SHORT)];
    (
        b.fresh_unliened_backing_num / NUM_PER_ATOM,
        b.valid_liened_backing_num / NUM_PER_ATOM,
        b.utilization_fee_earnings,
    )
}

/// BST-04: a top-up moves the provider's USDC into the vault as fresh,
/// unliened backing for that domain.
#[test]
fn bst_04_top_up_becomes_fresh_backing_in_the_vault() {
    require_artifacts!();
    let w = world(60);
    assert_eq!(w.h.token_balance(&w.provider.usdc).expect("wallet"), 0);
    assert_eq!(
        w.h.token_balance(&w.g.vault).expect("vault"),
        10_000 * USDC + 100 * USDC + 1_000 * USDC
    );
    assert_eq!(short_bucket(&w), (u128::from(1_000 * USDC), 0, 0));
}

/// BST-05: after a price move, the loser's realized loss joins the losing
/// side's backing and the winner holds a claim on it.
#[test]
fn bst_05_shock_moves_loser_loss_into_the_source_domain() {
    require_artifacts!();
    let mut w = world(61);
    long_then_shock(&mut w);
    let a = w.h.position(&w.alice).expect("alice");
    assert_eq!(a.pnl, 4_750_000, "95 contracts x $0.05");
    let (fresh, liened, earnings) = short_bucket(&w);
    assert_eq!(fresh, u128::from(1_000 * USDC) + 4_750_000);
    assert_eq!((liened, earnings), (0, 0));
    let (_, e) = w.h.market_state(&w.g).expect("state");
    assert_eq!(
        e.source_credit[usize::from(DOMAIN_SHORT)].positive_claim_bound_num / NUM_PER_ATOM,
        4_750_000,
        "Alice's winnings are a claim on the short domain"
    );
}

/// Decision 16 / BST-04: using backed winnings as margin liens backing and
/// charges the backing fee, split exactly between provider and insurance.
#[test]
fn dec_16_backing_fee_is_charged_on_lien_growth_and_split() {
    require_artifacts!();
    let mut w = world(62);
    long_then_shock(&mut w);
    let ins_before = w.h.market_state(&w.g).expect("state").1.insurance;
    // +1 contract at $1.05 needs 1.05 USDC more margin than the 100 USDC of
    // capital covers (96 x $1.05 = 100.80), so 0.80 USDC of winnings is used
    // as margin, liening 0.80 USDC of short-domain backing.
    w.h.trade_no_cpi_with_backing_fee_cap(
        &w.g,
        &w.alice,
        &w.maker.trader,
        CONTRACT,
        MARK_UP_5,
        0,
        FEE_BPS,
    )
    .expect("lien-growing trade");
    w.h.reconcile(&[&w.g]).expect("reconcile");

    let (fresh, liened, earnings) = short_bucket(&w);
    let insurance = w.h.market_state(&w.g).expect("state").1.insurance - ins_before;
    let charged = u128::from(100 * USDC) - w.h.position(&w.alice).expect("alice").capital;
    assert_eq!(liened, 800_000, "0.80 USDC liened");
    assert_eq!(fresh, u128::from(1_000 * USDC) + 4_750_000 - 800_000);
    assert_eq!(charged, 400_000, "50% of the 0.80 USDC lien");
    assert_eq!(earnings, 300_000, "75% to the provider");
    assert_eq!(insurance, 100_000, "25% to insurance");
    assert_eq!(charged, earnings + insurance, "no leakage");
}

/// The fee consent bound is enforced: a trader who caps the backing fee
/// below the policy cannot grow a lien.
#[test]
fn dec_16_backing_fee_needs_the_traders_consent() {
    require_artifacts!();
    let mut w = world(63);
    long_then_shock(&mut w);
    let result = w.h.trade_no_cpi_with_backing_fee_cap(
        &w.g,
        &w.alice,
        &w.maker.trader,
        CONTRACT,
        MARK_UP_5,
        0,
        FEE_BPS - 1,
    );
    // The wrapper checks each debited account's signed cap against the
    // policy fee, only when a fee is actually charged.
    assert_eq!(custom_error(&result), Some(err::UNAUTHORIZED), "{result:?}");
    assert_eq!(short_bucket(&w).1, 0, "no lien");
}

/// BST-07 at the pin: the per-slot utilization rate is zero. The market's
/// stored config has base rate and slopes at 0, and a lien held for 48
/// slots, cranked every 4, earns nothing beyond the one-off backing fee.
#[test]
fn bst_07_per_slot_utilization_fee_is_zero_at_the_pin() {
    require_artifacts!();
    let mut w = world(64);
    let (_, e) = w.h.market_state(&w.g).expect("state");
    assert_eq!(e.config.backing_fee_base_rate_e9_per_slot, 0);
    assert_eq!(e.config.backing_fee_slope_at_kink_e9_per_slot, 0);
    assert_eq!(e.config.backing_fee_slope_above_kink_e9_per_slot, 0);

    long_then_shock(&mut w);
    w.h.trade_no_cpi_with_backing_fee_cap(
        &w.g,
        &w.alice,
        &w.maker.trader,
        CONTRACT,
        MARK_UP_5,
        0,
        FEE_BPS,
    )
    .expect("lien");
    let after_fee = short_bucket(&w);
    let mut slot = 2;
    for _ in 0..12 {
        slot += e.config.max_accrual_dt_slots;
        w.h.warp_to_slot(slot);
        w.h.push_mark(&w.g, MARK_UP_5).expect("push");
        crank(&mut w, "maker");
        crank(&mut w, "alice");
    }
    w.h.reconcile(&[&w.g]).expect("reconcile");
    assert_eq!(short_bucket(&w), after_fee, "nothing accrued over 48 slots");
}

/// BST-05: the provider syncs its ledger and withdraws its earnings to its
/// wallet; every balance reconciles.
#[test]
fn bst_05_provider_syncs_and_withdraws_earnings() {
    require_artifacts!();
    let mut w = world(65);
    long_then_shock(&mut w);
    w.h.trade_no_cpi_with_backing_fee_cap(
        &w.g,
        &w.alice,
        &w.maker.trader,
        CONTRACT,
        MARK_UP_5,
        0,
        FEE_BPS,
    )
    .expect("lien");
    let earnings = short_bucket(&w).2;
    w.h.sync_backing_ledger(&w.g, &w.provider, DOMAIN_SHORT)
        .expect("sync ledger");
    w.h.withdraw_backing_earnings(&w.g, &w.provider, DOMAIN_SHORT, earnings)
        .expect("withdraw earnings");
    w.h.reconcile(&[&w.g]).expect("reconcile");
    assert_eq!(
        u128::from(w.h.token_balance(&w.provider.usdc).expect("wallet")),
        earnings
    );
    assert_eq!(short_bucket(&w).2, 0, "earnings paid out");
}

/// A portfolio holding any position cannot withdraw, however much spare
/// capital it has; once flat it can. Applies to the LP maker (Phase 4
/// redemptions) and to traders.
#[test]
fn lp_withdraw_is_blocked_while_holding_a_position() {
    require_artifacts!();
    let mut w = world(66);
    w.h.trade_cpi(&w.g, &w.alice, &w.maker, 10 * CONTRACT, 0, 2 * ONE_DOLLAR)
        .expect("open");
    for t in [&w.maker.trader, &w.alice] {
        let result = w.h.withdraw(&w.g, t, u128::from(USDC));
        assert_eq!(
            custom_error(&result),
            Some(ENGINE_STALE),
            "{}: {result:?}",
            t.name
        );
    }
    w.h.trade_cpi(&w.g, &w.alice, &w.maker, -10 * CONTRACT, 0, 0)
        .expect("close");
    for t in [&w.maker.trader, &w.alice] {
        w.h.withdraw(&w.g, t, u128::from(USDC))
            .unwrap_or_else(|e| panic!("{} flat withdraw: {e}", t.name));
    }
    w.h.reconcile(&[&w.g]).expect("reconcile");
}
