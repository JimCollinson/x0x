# Verification — Slice 5 remediation (ADR-0016 Phase 1)

- Date: 2026-06-20
- Verifier: OpenAI GPT-5.5 verifier
- Build worktree/head: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build` @ `78abe2577e062127a82d04a4d97ec69983b12c12`
- Planning worktree/head before this artifact: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning` @ `c50ac26147a59b801f82e507f8cf2a32e4e9f595`
- Verdict: **passed**

## Goals result summary

1. **Leave/disband split remains intact — verified.** `POST /groups/:id/state/withdraw` is routed to `withdraw_group_state`; `DELETE /groups/:id` is routed to `leave_group`. CLI/API surfaces expose primary `x0x group disband` with hidden/deprecated `state-withdraw` alias, while `DELETE` remains `x0x group leave` self-removal. Creator `DELETE` emits self-leave `MemberRemoved`, not `GroupDeleted`.
2. **Disband retains a keyless withdrawn shell — verified.** Local disband and received `GroupDeleted` both retain withdrawn `GroupInfo` metadata, clear GSS shared-secret material, remove live MLS/TreeKEM state, and wipe snapshot/journal persistence. The retained shell blocks stale-card/import/join reanimation while leaving no group-scoped key material.
3. **TreeKEM at-rest terminality remediation — verified.** Terminal cleanup removes both `<group_id>.snap` and `<group_id>.journal` for same-stable-group aliases. Snapshot encoding refuses withdrawn `GroupInfo`; in-flight encrypt/decrypt persistence is bound to durable named-group state and removes stale files if withdrawal won the race. Startup journal recovery discards withdrawn journal payloads and, before replaying non-withdrawn payloads, refuses replay when durable named-groups state contains a withdrawn record for the journal group or alias.
4. **Alias coverage — verified.** Same-stable aliases include the request key, stable id, stored map keys, and `mls_group_id` values. Terminal wipe and durable-withdrawn journal replay checks use that alias set; focused tests cover same-stable alias collection and stale non-withdrawn journal rejection over a durable withdrawn alias.
5. **No-durable-record journal replay remains intentionally allowed — verified.** When no durable `named_groups.json` exists, journal replay still restores the journal snapshot and named-group JSON for in-flight atomic recovery. This is intentionally not treated as stale withdrawal because a no-record case is indistinguishable from a crash between journal write and atomic named-groups write; durable withdrawn records are the boundary that disables replay.
6. **Scope boundaries — verified for the remediation delta.** `a9907b0..78abe25` touches only `src/bin/x0xd.rs`; no CI, `.gsd` gate, Cargo, docs, `tests/**`, harness, daemon wrapper, or build-invocation files changed. The remediation adds/updates test code inside `x0xd.rs` only. No wire enum, signed commit, hash/role byte, or persisted journal/snapshot struct field changed.

## Verification evidence inspected

- Source docs read: Slice 5 checkpoint, Phase 1 spec, Phase 1 plan, ADR-0016, repo `CLAUDE.md`, repo `AGENTS.md`.
- Build state inspected:
  - `git status --short` — clean.
  - `git rev-parse HEAD` — `78abe2577e062127a82d04a4d97ec69983b12c12`.
  - `git log --oneline -8` — confirmed remediation commits `9f7922c`, `757cdf1`, `78abe25` on the requested branch.
- Diff/path checks:
  - `git diff --stat a9907b0..HEAD` — one file, `src/bin/x0xd.rs`, 401 insertions / 43 deletions.
  - `git diff --check a9907b0..HEAD` — PASS.
  - `git diff --name-only a9907b0..HEAD -- .github .gsd Cargo.toml Cargo.lock tests src/groups src/api docs scripts justfile` — no output.
- Code inspected:
  - Routes: `src/bin/x0xd.rs:2242-2243`.
  - Disband handler: `src/bin/x0xd.rs:12464-12557`.
  - Leave handler: `src/bin/x0xd.rs:12559-12644`; TreeKEM self-leave helper `12107-12195`.
  - `GroupDeleted` apply path: `src/bin/x0xd.rs:8688-8740`.
  - Alias/withdrawal helpers: `src/bin/x0xd.rs:11751-11834`, `11836-12058`.
  - Snapshot/journal persistence and replay: `src/bin/x0xd.rs:17899-18170`, startup restore `18178-18194`.
  - CLI/API/docs surfaces: `src/bin/x0x.rs:958-963`, `src/cli/commands/group.rs:12`, `431-438`, `src/api/mod.rs:660-665`, `docs/api-reference.md:306-329`.
  - Regression tests: `src/bin/x0xd.rs:20551-21010`, `21148-21190`; `tests/membership_authority.rs:482-626`; ignored daemon proof in `tests/named_group_integration.rs:1529-1741`.

## Commands run by verifier

- `cargo fmt --all -- --check` — PASS.
- `cargo test --all-features --bin x0xd treekem` — PASS, 23/23.
- `cargo test --all-features --bin x0xd withdrawn` — PASS, 10/10.
- `cargo nextest run --all-features -E 'test(leave) or test(disband) or test(withdraw)'` — PASS, 23/23.
- Final `git status --short` in build worktree — clean.

## Honesty / scope assessment

- No-harness-modification: **pass** — no harness/CI/gate/wrapper/build/env changes found in the remediation delta. Test additions are in the approved remediation surface inside `src/bin/x0xd.rs`.
- Baseline-diff for evidence: **pass** — no verifier command failure was dismissed. User-provided prior evidence includes broader clean-head checks; verifier independently reran focused checks at `78abe25`.
- Evidence reproducible from branch: **pass** — commands used committed repo tooling only; no uncommitted scripts, wrappers, or environment variables.
- Local vs CI consistency: **not independently checked** — verifier did not query PR/CI. Treat local evidence as strong but not CI green-of-record.

## Gaps / blockers

- No remediation-goal blocker found.
- Non-blocking note: I did not find a separate source-code comment using the exact phrase "indistinguishable from stale left-group/no-record". The behaviour is captured by the positive no-durable replay test and by this verification artifact; add an in-code comment only if Jim/maintainer wants that rationale closer to `recover_treekem_named_journals`.

## Recommended next gate

Proceed to the remaining GSD gates for substantial work before PR/readiness: clean-context test plus integrated adversarial and Craft Review over the composed branch, then CI/maintainer-side daemon gate per the Phase 1 plan. Do not open a PR without Jim's explicit confirmation.
