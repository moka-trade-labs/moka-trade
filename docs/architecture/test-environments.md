# Test environments: LiteSVM, Mollusk, Surfpool (mainnet fork), devnet

- Date: 2026-09-30
- Answers: "Can we run on a forked mainnet first? How do LiteSVM/Mollusk get mainnet parameters?"
- Requirement IDs: TEST-02, TEST-03, TEST-07, TEST-08, OPS-01.

## 1. Short answer

Yes, forking mainnet is realistic and is Phase 6 of the plan. Surfpool is built for it: it is a drop-in `solana-test-validator` replacement that fetches any mainnet account the first time a transaction touches it (copy-on-read). Anchor 1.0 made it the default local validator. We deploy **our own build** of the pinned Percolator wrapper and our three programs into the fork. Real PumpSwap/Raydium/Meteora pools, token mints and Pyth accounts are pulled from mainnet on demand. **No real funds are involved:** it is a local chain, and every balance change stays on your machine.

Use each tool for what it is good at:

| Tool | What it is | Use it for | Mainnet data |
| --- | --- | --- | --- |
| **Mollusk** | Harness that runs **one instruction** against one program, no bank | Unit tests of a single instruction, exact CU measurement, account-validation edge cases in our programs | Pass accounts you built or loaded from JSON fixtures |
| **LiteSVM** | In-process SVM with a bank, **multi-transaction** flows, fast (ms) | Phase 2 slice, protocol property tests, fuzzing, CI (upstream's suite uses it) | Load account snapshots with `set_account`; load programs with `add_program` |
| **Surfpool** | Full local validator with RPC, lazy mainnet fork, time-travel cheatcodes | Phase 6 rehearsal with real pools and Pyth; SDK/keeper/web app against realistic state | Automatic: any account read is fetched from mainnet |
| **Devnet** | Public test cluster | Phase 8 public MVP | PumpSwap exists on devnet; we create our own test tokens and pools |

## 2. Getting mainnet state into LiteSVM/Mollusk (golden fixtures)

1. Pick the accounts: a PumpSwap pool and its two vault token accounts, the token mint, the Pyth SOL/USD `PriceUpdateV2` account, and programs we need to execute (e.g. PumpSwap, to simulate a manipulation swap).
2. Dump them once, reproducibly:
   ```bash
   # account data (base64 JSON) at a known slot
   solana account <PUBKEY> -u mainnet-beta --output json-compact > tests/fixtures/mainnet/<name>.json
   # a program's executable
   solana program dump <PROGRAM_ID> tests/fixtures/programs/<name>.so -u mainnet-beta
   ```
   Record the slot, the RPC and the reason in `tests/fixtures/mainnet/MANIFEST.md` (TEST-08 provenance). A private RPC key goes in your shell environment, **never** in the repo.
3. Load them in tests:
   ```rust
   let mut svm = LiteSVM::new();
   svm.add_program(PUMPSWAP_ID, &std::fs::read("tests/fixtures/programs/pumpswap.so")?);
   let acct: Account = fixture_json_to_account("tests/fixtures/mainnet/pool.json")?;
   svm.set_account(POOL_PUBKEY, acct)?;
   // Freshness checks: warp the clock to the fixture's slot, or rewrite
   // publish_time in the Pyth fixture for staleness tests.
   svm.warp_to_slot(FIXTURE_SLOT);
   ```
4. Mollusk takes the same `Account` values in its `process_instruction(&ix, &accounts)` call, so the fixture loader is shared.

Fixtures give deterministic, offline, CI-friendly tests. Surfpool gives breadth. We use both.

## 3. Surfpool rehearsal (Phase 6) outline

```bash
surfpool start --rpc-url "$MAINNET_RPC_URL"     # lazy fork of mainnet; the RPC URL comes from your env
solana program deploy target/deploy/percolator_prog.so --url http://127.0.0.1:8899
solana program deploy target/deploy/{matcher,lp_vault,market_registry}.so --url http://127.0.0.1:8899
pnpm --filter scripts rehearsal --cluster http://127.0.0.1:8899   # list → trade → shock → liquidate → redeem
```

Known issues to handle in Phase 6, all expected and solvable:

- **Stale Pyth:** the forked Pyth account stops updating. Use Surfpool's account-override/time-travel cheatcodes (verify exact RPC names against the installed version) or post a fresh `PriceUpdateV2` through the Pyth receiver.
- **Price shocks:** swap against the forked pool with a funded local wallet (Surfpool can airdrop and set token balances) to create real-looking moves.
- **Program IDs:** our wrapper gets a local program ID, so the SDK takes program IDs from a cluster manifest, never hard-coded.
- **Determinism:** mainnet state changes between runs. For assertions, snapshot the fetched accounts into fixtures (§2) and replay them in LiteSVM.

## 4. What this environment cannot prove

A fork proves the code works against real account layouts and real liquidity snapshots. It does not prove that real users, keepers and arbitrageurs behave as modeled, and it cannot test real network congestion. Those come from devnet (Phase 8) and, much later and only with explicit authorization, a capped mainnet pilot.

## 5. Sources

- Surfpool: https://github.com/txtx/surfpool, https://solana.com/docs/tools/surfpool, https://www.helius.dev/blog/surfpool
- Mainnet forking guide: https://www.quicknode.com/guides/solana-development/accounts-and-data/fork-programs-to-localnet
- LiteSVM: https://github.com/LiteSVM/litesvm ; Mollusk: https://github.com/anza-xyz/mollusk
