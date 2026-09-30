//! LiteSVM harness for the pinned Percolator wrapper (Phase 2, spec TEST-02).
//!
//! Follows upstream's own harness (`vendor/percolator-prog/tests/support/
//! v16_svm.rs`) but stays small: one isolated single-asset market group per
//! token (SEC-09), programs loaded only through the pin manifest, and
//! instructions encoded by the pinned wrapper crate itself.

use crate::load_pinned_program;
use percolator_prog::{ix::Instruction as WrapperIx, state};
use solana_sdk::{
    account::Account,
    compute_budget::ComputeBudgetInstruction,
    hash::hashv,
    instruction::{AccountMeta, Instruction},
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{keypair_from_seed, Keypair, Signer},
    transaction::Transaction,
};
use spl_token::state::{Account as TokenAccount, AccountState, Mint};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Compute-unit limit requested for every transaction (Solana maximum).
pub const TX_CU_LIMIT: u32 = 1_400_000;
/// Single-asset market groups only (SEC-09: one market group per token).
pub const ASSETS_PER_GROUP: u16 = 1;
/// Rent-free lamports given to fixture accounts; LiteSVM does not charge rent
/// for accounts created with `set_account`.
const FIXTURE_LAMPORTS: u64 = 1_000_000_000;

/// Market parameters passed to `InitMarket`.
///
/// These are test placeholders, not the launch profile: the calibrated
/// profile is blocked on TEST-06 (see `docs/HANDOFF.md` §7).
#[derive(Clone, Copy, Debug)]
pub struct MarketProfile {
    pub initial_price_e6: u64,
    pub h_max: u64,
    pub min_nonzero_mm_req: u128,
    pub min_nonzero_im_req: u128,
    pub maintenance_margin_bps: u64,
    pub initial_margin_bps: u64,
    pub max_trading_fee_bps: u64,
    pub liquidation_fee_bps: u64,
    pub liquidation_fee_cap: u128,
    pub min_liquidation_abs: u128,
    pub max_price_move_bps_per_slot: u64,
    pub max_accrual_dt_slots: u64,
    pub max_abs_funding_e9_per_slot: u64,
    pub min_funding_lifetime_slots: u64,
    pub max_bankrupt_close_lifetime_slots: u64,
    pub maintenance_fee_per_slot: u128,
}

impl Default for MarketProfile {
    /// Mirrors upstream's `MarketConfig::default()` so upstream scenarios can
    /// be reproduced before our own profile exists.
    fn default() -> Self {
        Self {
            initial_price_e6: 1_000_000,
            h_max: 10,
            min_nonzero_mm_req: 1,
            min_nonzero_im_req: 2,
            maintenance_margin_bps: 10_000,
            initial_margin_bps: 10_000,
            max_trading_fee_bps: 10_000,
            liquidation_fee_bps: 0,
            liquidation_fee_cap: 0,
            min_liquidation_abs: 0,
            max_price_move_bps_per_slot: 1_000,
            max_accrual_dt_slots: 4,
            max_abs_funding_e9_per_slot: 0,
            min_funding_lifetime_slots: 4,
            max_bankrupt_close_lifetime_slots: 100,
            maintenance_fee_per_slot: 0,
        }
    }
}

/// One isolated market group: its own market account and USDC vault.
#[derive(Debug)]
pub struct MarketGroup {
    pub market: Pubkey,
    pub admin: Keypair,
    pub vault_authority: Pubkey,
    pub vault: Pubkey,
}

pub struct Harness {
    pub svm: litesvm::LiteSVM,
    pub program_id: Pubkey,
    pub matcher_program: Pubkey,
    pub usdc_mint: Pubkey,
    payer: Keypair,
    /// Every USDC token account the harness created, by display name. The
    /// mint supply always equals the sum of these balances (CAP-13).
    token_accounts: BTreeMap<Pubkey, String>,
    seed: [u8; 32],
    next_label: u8,
    tx_sequence: u64,
}

impl Harness {
    /// Loads the pinned wrapper and `auth_matcher` (hash-verified) and
    /// creates a 6-decimal USDC-like mint.
    pub fn new(seed: [u8; 32]) -> Result<Self, String> {
        let mut svm = litesvm::LiteSVM::new(); // includes SPL Token and ATA
        let program_id = percolator_prog::id();
        svm.add_program(program_id, &load_pinned_program("percolator_prog.so")?);

        let mut harness = Self {
            svm,
            program_id,
            matcher_program: Pubkey::default(),
            usdc_mint: Pubkey::default(),
            payer: keypair(&seed, 0),
            token_accounts: BTreeMap::new(),
            seed,
            next_label: 1,
            tx_sequence: 0,
        };
        harness.matcher_program = harness.new_keypair().pubkey();
        let matcher = load_pinned_program("auth_matcher.so")?;
        harness.svm.add_program(harness.matcher_program, &matcher);
        harness
            .svm
            .airdrop(&harness.payer.pubkey(), 100_000_000_000)
            .map_err(|e| format!("airdrop payer: {e:?}"))?;
        harness.usdc_mint = harness.new_keypair().pubkey();
        harness.set_mint(harness.usdc_mint, moka_types::units::USDC_DECIMALS)?;
        Ok(harness)
    }

    /// Deterministic keypair; each call returns a new one.
    pub fn new_keypair(&mut self) -> Keypair {
        let label = self.next_label;
        self.next_label = label
            .checked_add(1)
            .expect("harness keypair labels exhausted");
        keypair(&self.seed, label)
    }

    /// Allocates and initializes one single-asset market group (`InitMarket`).
    pub fn init_market_group(&mut self, profile: MarketProfile) -> Result<MarketGroup, String> {
        let admin = self.new_keypair();
        self.svm
            .airdrop(&admin.pubkey(), 10_000_000_000)
            .map_err(|e| format!("airdrop admin: {e:?}"))?;
        let market = self.new_keypair().pubkey();
        let len = state::market_account_len_for_capacity(usize::from(ASSETS_PER_GROUP))
            .map_err(|e| format!("market len: {e:?}"))?;
        self.set_program_account(market, len)?;

        let vault_authority =
            Pubkey::find_program_address(&[b"vault", market.as_ref()], &self.program_id).0;
        let vault = spl_associated_token_account_address(&vault_authority, &self.usdc_mint);
        let name = format!("vault[{}]", short(&market));
        self.set_token_account(&name, vault, vault_authority, 0)?;

        self.send_init_market(market, &admin, profile)?;
        Ok(MarketGroup {
            market,
            admin,
            vault_authority,
            vault,
        })
    }

    /// Sends `InitMarket` for an already allocated market account.
    pub fn send_init_market(
        &mut self,
        market: Pubkey,
        admin: &Keypair,
        profile: MarketProfile,
    ) -> Result<u64, String> {
        let p = profile;
        self.send_wrapper(
            WrapperIx::InitMarket {
                max_portfolio_assets: ASSETS_PER_GROUP,
                h_min: 0,
                h_max: p.h_max,
                initial_price: p.initial_price_e6,
                min_nonzero_mm_req: p.min_nonzero_mm_req,
                min_nonzero_im_req: p.min_nonzero_im_req,
                maintenance_margin_bps: p.maintenance_margin_bps,
                initial_margin_bps: p.initial_margin_bps,
                max_trading_fee_bps: p.max_trading_fee_bps,
                trade_fee_base_bps: 0,
                liquidation_fee_bps: p.liquidation_fee_bps,
                liquidation_fee_cap: p.liquidation_fee_cap,
                min_liquidation_abs: p.min_liquidation_abs,
                max_price_move_bps_per_slot: p.max_price_move_bps_per_slot,
                max_accrual_dt_slots: p.max_accrual_dt_slots,
                max_abs_funding_e9_per_slot: p.max_abs_funding_e9_per_slot,
                min_funding_lifetime_slots: p.min_funding_lifetime_slots,
                max_account_b_settlement_chunks: 1,
                max_bankrupt_close_chunks: 1,
                max_bankrupt_close_lifetime_slots: p.max_bankrupt_close_lifetime_slots,
                public_b_chunk_atoms: percolator::MAX_VAULT_TVL,
                maintenance_fee_per_slot: p.maintenance_fee_per_slot,
            },
            vec![
                AccountMeta::new(admin.pubkey(), true),
                AccountMeta::new(market, false),
                AccountMeta::new_readonly(self.usdc_mint, false),
            ],
            &[admin],
        )
    }

    /// Decoded wrapper config and engine market group.
    pub fn market_state(
        &self,
        group: &MarketGroup,
    ) -> Result<(state::WrapperConfigV16, state::MarketGroupV16), String> {
        let account = self.account(&group.market)?;
        state::read_market(&account.data).map_err(|e| format!("decode market: {e:?}"))
    }

    pub fn account(&self, key: &Pubkey) -> Result<Account, String> {
        self.svm
            .get_account(key)
            .ok_or_else(|| format!("account {key} missing"))
    }

    pub fn token_balance(&self, key: &Pubkey) -> Result<u64, String> {
        let account = self.account(key)?;
        TokenAccount::unpack(&account.data)
            .map(|t| t.amount)
            .map_err(|e| format!("decode token account {key}: {e:?}"))
    }

    /// Sends one wrapper instruction; returns compute units consumed.
    pub fn send_wrapper(
        &mut self,
        ix: WrapperIx,
        accounts: Vec<AccountMeta>,
        signers: &[&Keypair],
    ) -> Result<u64, String> {
        let ix = Instruction {
            program_id: self.program_id,
            accounts,
            data: ix.encode(),
        };
        self.send(vec![ix], signers)
    }

    /// Sends instructions with a compute budget prefix, paid by the harness
    /// payer. A per-transaction priority fee keeps signatures unique.
    pub fn send(&mut self, ixs: Vec<Instruction>, signers: &[&Keypair]) -> Result<u64, String> {
        self.tx_sequence = self
            .tx_sequence
            .checked_add(1)
            .ok_or("tx sequence overflow")?;
        let mut all = vec![
            ComputeBudgetInstruction::request_heap_frame(256 * 1024),
            ComputeBudgetInstruction::set_compute_unit_limit(TX_CU_LIMIT),
            ComputeBudgetInstruction::set_compute_unit_price(self.tx_sequence),
        ];
        all.extend(ixs);
        let mut all_signers: Vec<&Keypair> = vec![&self.payer];
        all_signers.extend_from_slice(signers);
        let tx = Transaction::new_signed_with_payer(
            &all,
            Some(&self.payer.pubkey()),
            &all_signers,
            self.svm.latest_blockhash(),
        );
        self.svm
            .send_transaction(tx)
            .map(|meta| meta.compute_units_consumed)
            .map_err(|e| format!("{:?}: {}", e.err, e.meta.logs.join("\n")))
    }

    fn set_program_account(&mut self, key: Pubkey, len: usize) -> Result<(), String> {
        self.set(key, self.program_id, vec![0u8; len])
    }

    fn set_mint(&mut self, key: Pubkey, decimals: u8) -> Result<(), String> {
        let mut data = vec![0u8; Mint::LEN];
        Mint::pack(
            Mint {
                mint_authority: COption::None,
                supply: 0,
                decimals,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            &mut data,
        )
        .map_err(|e| format!("pack mint: {e:?}"))?;
        self.set(key, spl_token::ID, data)
    }

    /// Creates (or overwrites) a USDC token account holding `amount` and
    /// moves the mint supply by the balance change, so supply always equals
    /// the sum of tracked balances. `name` labels it in reconciliation traces.
    pub fn set_token_account(
        &mut self,
        name: &str,
        key: Pubkey,
        owner: Pubkey,
        amount: u64,
    ) -> Result<(), String> {
        let previous = if self.token_accounts.contains_key(&key) {
            self.token_balance(&key)?
        } else {
            0
        };
        let mut data = vec![0u8; TokenAccount::LEN];
        TokenAccount::pack(
            TokenAccount {
                mint: self.usdc_mint,
                owner,
                amount,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            &mut data,
        )
        .map_err(|e| format!("pack token account: {e:?}"))?;
        self.set(key, spl_token::ID, data)?;
        self.token_accounts.insert(key, name.to_owned());

        let mint = self.usdc_mint;
        let mut mint_account = self.account(&mint)?;
        let mut mint_state =
            Mint::unpack(&mint_account.data).map_err(|e| format!("unpack mint: {e:?}"))?;
        mint_state.supply = mint_state
            .supply
            .checked_sub(previous)
            .and_then(|s| s.checked_add(amount))
            .ok_or("mint supply out of range")?;
        Mint::pack(mint_state, &mut mint_account.data).map_err(|e| format!("pack mint: {e:?}"))?;
        self.svm
            .set_account(mint, mint_account)
            .map_err(|e| format!("set mint: {e:?}"))
    }

    /// Balances and engine ledger totals at one point in a scenario.
    pub fn snapshot(&self, label: &str, groups: &[&MarketGroup]) -> Result<Snapshot, String> {
        let mut tokens = BTreeMap::new();
        for (key, name) in &self.token_accounts {
            tokens.insert(name.clone(), self.token_balance(key)?);
        }
        let mint = Mint::unpack(&self.account(&self.usdc_mint)?.data)
            .map_err(|e| format!("unpack mint: {e:?}"))?;
        let mut ledgers = BTreeMap::new();
        for group in groups {
            let (_, engine) = self.market_state(group)?;
            ledgers.insert(
                format!("market[{}]", short(&group.market)),
                Ledger {
                    vault: engine.vault,
                    insurance: engine.insurance,
                    capital: engine.c_tot,
                    backing_earnings: engine.backing_provider_earnings_total,
                },
            );
        }
        Ok(Snapshot {
            label: label.to_owned(),
            tokens,
            mint_supply: mint.supply,
            ledgers,
        })
    }

    /// CAP-13 reconciliation, run after every step of a scenario:
    /// 1. conservation: tracked token balances sum to the mint supply;
    /// 2. custody: each group's SPL vault balance equals the engine `vault`
    ///    ledger (upstream's own census rule);
    /// 3. solvency: capital + insurance + backing earnings never exceed the
    ///    vault (a subset of upstream's explicit stocks, so a weaker but
    ///    sound bound).
    pub fn reconcile(&self, groups: &[&MarketGroup]) -> Result<(), String> {
        let mut sum: u128 = 0;
        for key in self.token_accounts.keys() {
            sum = sum
                .checked_add(u128::from(self.token_balance(key)?))
                .ok_or("token sum overflow")?;
        }
        let supply = Mint::unpack(&self.account(&self.usdc_mint)?.data)
            .map_err(|e| format!("unpack mint: {e:?}"))?
            .supply;
        if sum != u128::from(supply) {
            return Err(format!(
                "conservation: balances sum {sum} != supply {supply}"
            ));
        }
        for group in groups {
            let (_, engine) = self.market_state(group)?;
            let spl_vault = u128::from(self.token_balance(&group.vault)?);
            if engine.vault != spl_vault {
                return Err(format!(
                    "custody: market {} engine vault {} != SPL vault {spl_vault}",
                    group.market, engine.vault
                ));
            }
            let claims = engine
                .c_tot
                .checked_add(engine.insurance)
                .and_then(|x| x.checked_add(engine.backing_provider_earnings_total))
                .ok_or("claims overflow")?;
            if claims > engine.vault {
                return Err(format!(
                    "solvency: market {} claims {claims} > vault {}",
                    group.market, engine.vault
                ));
            }
        }
        Ok(())
    }

    fn set(&mut self, key: Pubkey, owner: Pubkey, data: Vec<u8>) -> Result<(), String> {
        self.svm
            .set_account(
                key,
                Account {
                    lamports: FIXTURE_LAMPORTS,
                    data,
                    owner,
                    executable: false,
                    rent_epoch: 0,
                },
            )
            .map_err(|e| format!("set account {key}: {e:?}"))
    }
}

/// Engine ledger totals for one market group, in USDC atoms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ledger {
    pub vault: u128,
    pub insurance: u128,
    pub capital: u128,
    pub backing_earnings: u128,
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub label: String,
    pub tokens: BTreeMap<String, u64>,
    pub mint_supply: u64,
    pub ledgers: BTreeMap<String, Ledger>,
}

impl Snapshot {
    /// Human-readable before/after trace of every value that changed.
    pub fn trace(&self, after: &Snapshot) -> String {
        let mut out = format!("[{} -> {}]\n", self.label, after.label);
        let mut changed = false;
        for (name, &new) in &after.tokens {
            let old = self.tokens.get(name).copied().unwrap_or_default();
            if old != new {
                changed = true;
                let _ = writeln!(out, "  token  {name:<24} {old} -> {new}");
            }
        }
        for (name, new) in &after.ledgers {
            let old = self.ledgers.get(name).copied();
            if old != Some(*new) {
                changed = true;
                let _ = writeln!(out, "  ledger {name:<24} {old:?} -> {new:?}");
            }
        }
        if self.mint_supply != after.mint_supply {
            changed = true;
            let _ = writeln!(
                out,
                "  supply {} -> {}",
                self.mint_supply, after.mint_supply
            );
        }
        if !changed {
            out.push_str("  (no balance or ledger changes)\n");
        }
        out
    }
}

fn short(key: &Pubkey) -> String {
    key.to_string().chars().take(6).collect()
}

fn keypair(seed: &[u8; 32], label: u8) -> Keypair {
    let derived = hashv(&[b"moka-tests", seed, &[label]]).to_bytes();
    keypair_from_seed(&derived).expect("32-byte seed is always valid")
}

/// The associated token account address the wrapper expects for its vault.
pub fn spl_associated_token_account_address(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    let ata_program: Pubkey = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        .parse()
        .expect("static ATA program id");
    Pubkey::find_program_address(
        &[owner.as_ref(), spl_token::ID.as_ref(), mint.as_ref()],
        &ata_program,
    )
    .0
}
