# Checkpoint — Slice 5 final remediation blocker at `1fa5f23`

- Date: 2026-06-21
- Slice/question: ADR-0016 Phase 1 Slice 5 — ingestion terminality + stale test + final MemberJoined rollback remediation
- Prepared by: OpenCode orchestrator
- Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
- Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `1fa5f23bd833a4c71e231efc7f19f4cef63ff13e`
- PR / CI arbiter: PR #5, <https://github.com/JimCollinson/x0x/pull/5>
- Status: **Blocked — adversarial rerun returned NOT-READY with unresolved HIGH durable persistence race.**
- Meaningful work-unit? Yes — non-trivial Rust API/group-authority/security behaviour in a shared/upstream-bound repo.
- Review cadence: per-unit/integrated gauntlet required; clean-context/adversarial/craft rerun completed at `1fa5f23`.

## What happened

- Remediated prior adversarial/verifier findings in two commits:
  - `f70faf9 fix(adr-0016-phase-1): guard metadata key installs`
  - `1fa5f23 fix(adr-0016-phase-1): rollback member-joined treekem add`
- Pushed branch to Jim's fork. GSD pre-push tripwire passed:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- Code review and verifier passed the final local mutation-gap closure.
- PR #5 CI ran at exact head `1fa5f23`.
- Build `linux-arm64-gnu` initially failed due missing `aarch64-linux-gnu-gcc`; rerunning failed Build jobs passed.
- CI Test Suite initially failed two tests; rerun left only the known daemon startup-timeout signature.
- Final clean-context/adversarial/craft gates were run.

## Evidence

Local mandatory/focused checks from operative/code review/verifier:

- `cargo fmt --all` / `cargo fmt --all -- --check` — PASS.
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS.
- `cargo check --workspace --all-targets` — PASS.
- `cargo test --all-features --bin x0xd member_joined_treekem` — PASS, 2/2.
- `cargo test --all-features --bin x0xd lost_race` — PASS, 18/18.
- `cargo test --all-features --bin x0xd withdrawn` — PASS, 15/15.
- `cargo test --all-features --bin x0xd treekem` — PASS, 31/31.
- `cargo nextest run --all-features -E 'test(leave) or test(disband) or test(withdraw)'` — PASS, 23/23.
- Focused withdrawn-card ignored tests — PASS.
- `git diff --check` — PASS.

Code review:

- Reviewer/tool: `codereviewer`.
- Result: **Passed** for `f70faf9` and `1fa5f23`.
- Noted no forbidden scope changes and no blocking quality/security issues.

Verifier:

- Reviewer/tool: `verifier`.
- Result: **Passed** after `1fa5f23`.
- Verified:
  - prior `MemberJoined` mutation gap closed;
  - regressions genuinely exercise lost-race/already-withdrawn rollback;
  - mutation-surface map credible;
  - `f70faf9` remediation intact;
  - no forbidden scope/harness/format changes.

CI arbiter / green of record:

- Location: PR #5, <https://github.com/JimCollinson/x0x/pull/5>
- Head OID: `1fa5f23bd833a4c71e231efc7f19f4cef63ff13e`
- Passing after reruns:
  - Build: Validate release metadata; Build linux-x64-gnu; Build linux-x64-musl; Build linux-arm64-gnu; Build macos-x64; Build macos-arm64; Build windows-x64.
  - CI: Format Check; Clippy Lint; Coverage Gate; Documentation; API + GUI Parity Gate.
  - Integration & Soak Tests: API Coverage Guard; Property Tests; Multi-Agent Integration.
  - Security Audit: Cargo Audit; Panic Scanner.
- Skipped: Soak Test.
- Remaining raw red: CI / Test Suite, run `27907990061`, rerun job `82581144890`.
  - Command: `cargo nextest run --all-features --workspace -E '!binary(x0x_0041_synthetic_kill_restart)'`
  - Summary: `1758/1765 tests run: 1757 passed (1 slow), 1 failed, 164 skipped`
  - Failing test: `x0x::named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected`
  - Verbatim failure: `x0xd pair-alice-47500 did not become healthy within 90s`
  - Failure site: `tests/harness/src/cluster.rs:68:17`
- Base-vs-current proof after final remediation:
  - Base `189b89c0aadb25a1458752fdec040d01df9d2d66`: same focused test failed after `90.174s`, `x0xd pair-alice-55461 did not become healthy within 90s`.
  - Current `1fa5f23`: same focused test failed after `90.210s`, `x0xd pair-alice-62669 did not become healthy within 90s`.
  - Under Jim's explicit instruction, this classifies environmental and the startup-timeout carve-out applies.

## Final review findings

Clean-context test:

- Reviewer/tool: `cleancontext`.
- Result: **Concerns**, no clean-context blocker.
- Concerns:
  - PR #5 raw red in GitHub UI; acceptance depends on Jim-approved startup/cluster environmental classification.
  - Existing checkpoint filename centered on `22fe1ed`; final `1fa5f23` artifact was not previously discoverable.
  - Planning artifacts remain untracked, so evidence is not fully branch-alone.
  - Local daemon integration remains not reliably exercisable; base/current both fail before assertions.
  - Broader stale docs/surfaces remain Slice 7 backlog.

Craft Review:

- Reviewer/tool: `craft`.
- Verdict: **Pass for Craft**.
- CONFORMANCE findings: none.
- SIMPLICITY carried: optional `DISBAND_VERB` / alias wrapper cleanup if touching CLI again.
- NIT carried: optionally shorten long `DELETE /groups/:id` table row in `docs/api-reference.md`.

Adversarial review:

- Reviewer/tool: `adversarial`, `openai/gpt-5.5`.
- Verdict: **NOT-READY**.
- Blocking finding:
  1. **HIGH:** TreeKEM “atomic” install can still persist non-withdrawn state after a terminal-withdrawal race.
     - Cited anchors:
       - `src/bin/x0xd.rs:7555-7581` performs `ensure_named_group_key_material_install_allowed(...)`, then `persist_treekem_and_named_groups_atomic_with_info(...)`, then another `ensure...`; the second check is after durable writes.
       - `src/bin/x0xd.rs:18369-18405` captures/writes `named_groups_json` after one pre-check, but has no re-check immediately before durable rename and no `named_groups.json` repair on a late terminality race.
       - `src/bin/x0xd.rs:11955-11971` terminality detection removes TreeKEM persistence but does not repair stale `named_groups.json`.
       - `src/bin/x0xd.rs:14494-14537` card import can terminally retain a withdrawn shell for keyless local state without taking `group_membership_locks`.
     - Why it matters: a keyless discovered/joining group can be withdrawn after the pre-check but before/during `persist_treekem_and_named_groups_atomic_with_info`; install may write stale non-withdrawn `named_groups.json`, later reject memory install, and leave durable resurrection state.
     - Required disposition: make terminality and durable write atomic with respect to terminal mutation — either serialize card-import withdrawal with the same per-group lock, or re-check under the write path immediately before durable rename and repair/abort safely. Add a disk-level race regression proving `named_groups.json` remains withdrawn and no snapshot/journal survives.
- Additional non-blocking findings:
  - **MEDIUM:** new tests do not exercise the post-precheck durable-write race window; they force withdrawal before the check and don't inspect `named_groups.json`.
  - **MEDIUM:** evidence package is not cleanly reproducible from branch alone because exact-head checkpoint evidence lives in planning artifacts, not committed branch files.

## Honesty rules check

- No-harness-modification: PASS — final remediation touched `src/bin/x0xd.rs` and `tests/named_group_integration.rs`; no `tests/harness/**`, CI, `.gsd/gate.sh`, daemon wrapper, build invocation, environment setup, Cargo, wire/commit/hash/storage-format changes.
- Baseline-diff for evidence: PASS for the remaining startup-timeout CI red — exact base and current branch focused test both reproduce the same 90s startup timeout, branch no worse.
- Evidence reproducible-from-branch: CONCERN — code/tests/check commands reproduce from branch; final checkpoint/review evidence is currently planning-worktree artifact and uncommitted.
- Local vs CI consistency: PASS under PR #5 internal startup-timeout carve-out, but raw CI is red in GitHub UI.

## Status

Slice 5 / PR #5 is **blocked / not Done / not PR-ready** because adversarial returned `NOT-READY` with an unresolved HIGH finding. Under GSD, this blocks the slice checkpoint until fixed or explicitly waived by Jim.

## Recommended next step

Stop for Jim decision / remediation planning:

1. Fix the durable persistence race by making TreeKEM install persistence terminality-safe atomically:
   - either serialize keyless card-import terminality with the same per-group lock, or
   - re-check/repair under the durable write path immediately before `named_groups.json` rename.
2. Add a disk-level regression that injects withdrawal after the first pre-check / after JSON capture and asserts:
   - durable `named_groups.json` remains withdrawn;
   - no TreeKEM snapshot/journal key material survives;
   - no in-memory install survives.
3. Rerun mandatory Rust checks, focused tests, verifier, push, CI classification, and final clean-context/adversarial/craft gates.

Do not call Slice 5 done, do not mark PR #5 ready, and do not open an upstream PR.
