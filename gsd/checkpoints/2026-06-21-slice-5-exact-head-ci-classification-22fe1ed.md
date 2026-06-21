# Checkpoint — Slice 5 exact-head CI classification at `22fe1ed`

- Date: 2026-06-21
- Slice/question: ADR-0016 Phase 1 Slice 5 — PR #5 exact-head CI after withdrawn-crypto terminality remediation
- Prepared by: OpenCode orchestrator
- Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
- Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `22fe1edd16f6517b864f61686d5f22f6864b30bd`
- PR / CI arbiter: PR #5, <https://github.com/JimCollinson/x0x/pull/5>
- Status: **Blocked — CI startup-timeout reds classified environmental with base proof, but adversarial rerun found unresolved HIGH findings.**
- Meaningful work-unit? Yes — non-trivial Rust API/CLI/group-authority/security behavior in a shared/upstream-bound repo.
- Review cadence: per-unit/integrated gauntlet required; clean-context/adversarial/craft rerun completed after base-vs-branch CI classification.

## What happened

- Remediation from the local follow-up checkpoint was committed and pushed as:
  - `22fe1ed fix(adr-0016-phase-1): enforce withdrawn crypto terminality`
- PR #5 now points at exact head `22fe1edd16f6517b864f61686d5f22f6864b30bd`.
- CI was queried via `gh pr view 5 --repo JimCollinson/x0x --json headRefOid,statusCheckRollup`.
- Saved rerun logs were parsed for the current red jobs.
- The full PR diff was inspected against `upstream/main...HEAD` for the arbiter diff guard in `gsd/ci-arbiter.md`.
- Per Jim's 2026-06-21 direction, both current failing tests were then run on the clean base worktree at `189b89c` and on the branch worktree at `22fe1ed`; base reproducing the same 90s startup timeout with branch no worse is accepted as environmental proof, overriding the overly textual `fn main` diff-guard interpretation for this decision.

## Evidence

CI arbiter / green of record:

- Location: PR #5, <https://github.com/JimCollinson/x0x/pull/5>
- Head OID: `22fe1edd16f6517b864f61686d5f22f6864b30bd`
- Passing checks observed:
  - Build: Validate release metadata; Build linux-x64-gnu; Build linux-x64-musl; Build linux-arm64-gnu; Build macos-x64; Build macos-arm64; Build windows-x64.
  - CI: Format Check; Clippy Lint; Coverage Gate; Documentation; API + GUI Parity Gate.
  - Integration & Soak Tests: API Coverage Guard; Property Tests.
  - Security Audit: Cargo Audit; Panic Scanner.
- Skipped: Soak Test.
- Raw red checks:
  1. CI / Test Suite — run `27898304064`, job `82554583953`.
  2. Integration & Soak Tests / Multi-Agent Integration — run `27898304060`, job `82554620655`.

Current red failure signatures:

1. CI / Test Suite
   - Command: `cargo nextest run --all-features --workspace -E '!binary(x0x_0041_synthetic_kill_restart)'`
   - Summary: `1758/1765 tests run: 1757 passed (1 slow), 1 failed, 164 skipped`
   - Failing test: `x0x::named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected`
   - Verbatim failure: `x0xd pair-alice-34041 did not become healthy within 90s`
   - Failure site: `tests/harness/src/cluster.rs:68:17`

2. Integration & Soak Tests / Multi-Agent Integration
   - Passing earlier command in same job: `cargo nextest run --all-features --test daemon_api_integration -- --ignored` — `89 tests run: 89 passed, 0 skipped`
   - Red command: `cargo nextest run --all-features --test named_group_integration --run-ignored ignored-only`
   - Summary: `6/27 tests run: 5 passed, 1 failed, 0 skipped`
   - Failing test: `x0x::named_group_integration named_group_creator_removal_propagates_to_removed_peer`
   - Verbatim failure: `x0xd pair-bob-46685 did not become healthy within 90s`
   - Failure site: `tests/harness/src/cluster.rs:68:17`

Startup-timeout carve-out assessment (`gsd/ci-arbiter.md`):

- Signature: **satisfied for observed red failures** — both failures are daemon startup health-timeouts before test assertions.
- Isolation: **satisfied** — 2 timed-out tests across current red jobs, within the `<= 3` limit.
- Diff guard: **textually not satisfied under the written rule, but superseded for this decision by Jim-approved base-vs-branch proof**.
  - `gsd/ci-arbiter.md` requires no change to `fn main`, the serve/startup sequence, `/health`, or node/transport/bootstrap initialization.
  - Full PR diff includes an `async fn main() -> Result<()>` hunk in `src/bin/x0xd.rs`:
    - hunk: `@@ -1642,0 +1648 @@ async fn main() -> Result<()> {`
    - added line: `expected_join_result_inviters: StdMutex::new(HashMap::new()),`
    - current source anchor: `src/bin/x0xd.rs:1622-1677`, inside `Arc::new(AppState { ... })` initialization.
  - This looks like state-field initialization, not health/listen/bootstrap logic, but the arbiter text says **no change to `fn main`**.
  - Jim's explicit direction for this decision: if the exact failing tests reproduce the same 90s startup timeout on base and the branch is no worse, classify environmental with proof and apply the carve-out regardless of this textual diff-guard.
- Record: this checkpoint records run/job IDs, failing tests, verbatim timeout lines, and diff-guard status.

Base-vs-branch proof run on 2026-06-21:

- Base worktree: `/var/folders/f_/j942sskj6nx67b6gk3rqgsqm0000gn/T/opencode/x0x-base-189b89c-focused-22fe1ed`
- Base commit: `189b89c0aadb25a1458752fdec040d01df9d2d66` (`upstream/main` / merge-base)
- Branch worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
- Branch commit: `22fe1edd16f6517b864f61686d5f22f6864b30bd`
- Both worktrees were clean after test execution (`git status --short --branch` showed no modified/untracked files beyond branch headers).

1. `forged_member_joined_admin_role_or_secret_is_rejected`
   - Command on both worktrees: `cargo nextest run --all-features --test named_group_join_metadata_event -E 'test(forged_member_joined_admin_role_or_secret_is_rejected)'`
   - Base result: FAIL after `90.250s`; `0 passed, 1 failed, 5 skipped`.
   - Base verbatim failure: `x0xd pair-alice-34154 did not become healthy within 90s` at `tests/harness/src/cluster.rs:68:17`.
   - Branch result: FAIL after `90.283s`; `0 passed, 1 failed, 7 skipped`.
   - Branch verbatim failure: `x0xd pair-alice-52438 did not become healthy within 90s` at `tests/harness/src/cluster.rs:68:17`.
   - Comparison: same pre-assertion 90s daemon startup timeout; branch is not meaningfully worse than base.

2. `named_group_creator_removal_propagates_to_removed_peer`
   - Command on both worktrees: `cargo nextest run --all-features --test named_group_integration --run-ignored ignored-only -E 'test(named_group_creator_removal_propagates_to_removed_peer)'`
   - Base result: FAIL after `90.181s`; `0 passed, 1 failed, 22 skipped`.
   - Base verbatim failure: `x0xd pair-alice-7850 did not become healthy within 90s` at `tests/harness/src/cluster.rs:68:17`.
   - Branch result: FAIL after `90.144s`; `0 passed, 1 failed, 26 skipped`.
   - Branch verbatim failure: `x0xd pair-alice-16256 did not become healthy within 90s` at `tests/harness/src/cluster.rs:68:17`.
   - Comparison: same pre-assertion 90s daemon startup timeout; branch is marginally faster / no worse than base.

Classification:

- Both exact failing tests reproduce the same 90s startup health-timeout on the clean base worktree.
- Branch failures are the same signature and no worse than base.
- Under Jim's explicit 2026-06-21 instruction, these current raw CI reds are environmental with proof; the PR #5 internal startup-timeout carve-out applies despite the textual `fn main` diff-guard concern.

Local/baseline classification evidence already available from the prior handoff:

- Branch local targeted `named_group_import_rejects_tampered_metadata_topic` failed with startup timeout: `x0xd pair-alice-19284 did not become healthy within 90s`.
- Detached `upstream/main` baseline at `189b89c` failed the same targeted command with startup timeout: `x0xd pair-alice-3703 did not become healthy within 90s`.
- Branch local targeted `direct_send_without_require_ack_omits_ack_block` passed 1/1.
- Baseline local targeted `direct_send_without_require_ack_omits_ack_block` passed 1/1 after compile-time retry.
- These support an environmental startup-flake diagnosis generally, but they do **not** override the written PR #5 diff guard for exact-head CI.

## Honesty rules check

- No-harness-modification: PASS for the final remediation commit — final commit touched `docs/api-reference.md` and `src/bin/x0xd.rs`; no `tests/harness/**`, CI, `.gsd/gate.sh`, daemon wrapper, build invocation, or environment setup changes.
- Baseline-diff for evidence: PASS — both exact current failing tests were run on clean base and branch worktrees; base reproduces the same 90s startup timeout and branch is no worse.
- Evidence reproducible-from-branch: PASS for the code branch at `22fe1ed`; planning checkpoint artifacts remain uncommitted in the planning worktree.
- Local vs CI consistency: PASS under PR #5 internal arbiter carve-out — exact CI reds match locally reproduced base/branch startup-timeout signatures.

## Review findings status

Clean-context test:

- Reviewer/tool: `cleancontext` agent.
- Result: **Concerns**, no clean-context blocker.
- Sources/checks included repo instructions, ADR-0016, Phase 1 spec/plan/PR notes, Slice 5 checkpoints, relevant source/docs/tests, PR #5 status, CI logs, diff scope, `git diff --check`, `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo check --workspace --all-targets`, `cargo test --all-features --bin x0xd lost_race`, `cargo test --all-features --bin x0xd withdrawn`, `cargo test --all-features --bin x0xd treekem`, and `cargo nextest run --all-features -E 'test(leave) or test(disband) or test(withdraw)'` — all local commands passed.
- Concerns carried:
  - no dedicated Slice 5 packet under `gsd/packets/`; scope is recoverable from plan/checkpoints/PR notes;
  - exact-head CI classification checkpoint is currently an untracked planning artifact;
  - PR #5 CI is raw red and classification depends on Jim-approved base-vs-branch proof;
  - non-canonical docs/surfaces remain stale and should stay in Slice 7 backlog, not be overclaimed here.
- CI classification: acceptable under the provided base proof and Jim's instruction.
- Clean-context perspective: can proceed to final checkpoint only if adversarial/final gates clear.

Adversarial review:

- Required? Yes — meaningful upstream-bound work-unit; no waiver/deferral.
- Reviewer/tool: `adversarial` agent.
- Result: **NOT-READY**.
- Blocking findings:
  1. **HIGH:** `tests/named_group_integration.rs` still contains a stale ignored test asserting the pre-remediation behaviour that an admin-signed withdrawn `GroupCard` terminates/wipes a live keyed group.
     - Anchor: `tests/named_group_integration.rs:1579-1651`, especially assertion that `shared_secret` is null after importing a withdrawn card.
     - Current implementation deliberately ignores withdrawn cards for protected live/keyed local groups and requires a signed withdrawal commit: `src/bin/x0xd.rs:14359-14376`.
     - CI fail-fast prevented the remaining ignored `named_group_integration` tests from running, so this latent stale assertion is not covered by the current startup-timeout carve-out.
     - Required disposition: replace/update the ignored test so card-only withdrawal does **not** terminate/wipe live/keyed local groups, then get evidence that the ignored command runs past fail-fast / this stale test.
  2. **HIGH:** metadata crypto/key-material ingestion paths still lack post-crypto withdrawn rechecks before storing secret material.
     - Anchors cited by adversarial:
       - `SecureShareDelivered` checks withdrawn before envelope open at `src/bin/x0xd.rs:9411-9417`, then opens/stores shared secret at `src/bin/x0xd.rs:9452-9477`.
       - `store_named_group_info` blindly overwrites the current stored record at `src/bin/x0xd.rs:7498-7508`.
       - TreeKEM welcome/commit apply paths do crypto/persistence at `src/bin/x0xd.rs:8452-8479` without a post-crypto terminality reread.
     - Required disposition: before storing shared secrets, TreeKEM snapshots, or named-group state after crypto, reread current group/same-stable aliases under the relevant write path and reject if withdrawn; add regression tests for metadata delivery paths, not only HTTP endpoint responses.
- Test-quality note: HTTP lost-race tests are useful but synthetic and do not cover metadata delivery paths that store secret material; one ignored integration test still asserts the old unsafe card-only terminality behaviour.
- Overstatement warnings:
  - Do not say required ignored CI suite is green; fail-fast stopped after 6/27 tests.
  - Do not overclaim “post-crypto terminality remediation” across all crypto/key-material ingestion paths.
  - Planning checkpoint artifacts remain uncommitted, weakening branch-alone evidence discovery.

Craft Review:

- Reviewer/tool: `craft` agent.
- Verdict: **Pass**.
- CONFORMANCE findings: none.
- SIMPLICITY carried: tiny `DISBAND_VERB` / alias wrapper note, non-blocking.

## Status

The CI/arbiter blocker is **cleared for PR #5's internal green-of-record** under Jim's base-vs-branch proof rule: both exact current failing tests reproduce the same startup timeout on base and branch is no worse.

Slice 5 / PR #5 is still **blocked / not Done / not PR-ready** because adversarial rerun returned `NOT-READY` with unresolved HIGH findings. Under GSD, these block the slice checkpoint until fixed or explicitly waived by Jim.

## Recommended next step

Recommended next step: create/execute a remediation slice for the two adversarial HIGH findings:

1. Update stale ignored `named_group_integration` withdrawn-card test to assert card-only withdrawal cannot wipe/terminate a live keyed local group; gather evidence that the ignored suite can run beyond this stale assertion despite the known startup flake.
2. Add post-crypto terminality rechecks to metadata key-ingestion paths before storing shared secrets / TreeKEM snapshots / named-group state, with regression coverage.

After remediation: run mandatory Rust checks (`cargo fmt --all`, `cargo clippy --all-features --all-targets -- -D warnings`, `cargo check --workspace --all-targets`), focused tests, push to PR #5, classify CI, then rerun clean-context/adversarial/craft as needed. Do not call Slice 5 done, do not mark PR #5 ready, and do not open an upstream PR until those gates clear.

Process note for later, not gating this decision: update `gsd/ci-arbiter.md` so the diff guard keys on whether the slice touches startup behaviour (IO/network/blocking in the boot path, `/health`, serve/listen/bootstrap/transport initialization), not any textual `fn main` hunk. Inert field initialization should not trip it, while boot-path changes tucked in helper/setup functions should.
