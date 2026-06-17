# Verification — Slice 4 final remediation (ADR-0016 Phase 1)

- Date: 2026-06-17
- Verifier: OpenAI GPT-5.5 verifier
- Build worktree/head: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build` @ `04a93afba2cabeb1627c7c484ad16ca9ba6fcd16`
- Planning worktree/head before this artifact: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning` @ `0dbe340d8592adbb0bbc81f671de7d5d90ceea6f`
- Verdict: **passed**

## Goals result summary

1. **Already-done items remain true — verified.** The known-local-group `GroupCardPublished` apply arm is active-Admin role-gated via `info.caller_role(&sender_hex).is_some_and(|role| role.at_least(GroupRole::Admin))`; expected join-result inviters live on `AppState`; creator provenance is documented as best-effort history from base-state/genesis; the `public_open` non-TreeKEM daemon invite e2e still exists.
2. **Discovery remediation scope — verified.** Global discovery, shard discovery, and ListedToContacts receive paths still verify signed discovery cards and cache them without current-roster Admin checks. Planning/spec/PR notes scope those as pre-existing signed-hint/key-possession discovery caches, not current-admin authority, and flag the known-local-group override observation to David. No discovery receive-path hardening was added in the final delta.
3. **TreeKEM coverage claim — verified.** The `private_secure` daemon e2e is now named/commented as secure-plane end-to-end join shape / Welcome-security-binding convergence / creator-vs-inviter split. Direct expected-inviter sender/actor rejection is explicitly attributed to the focused `join_result_requires_stored_expected_inviter` unit regression, not overclaimed from daemon e2e convergence.
4. **Forbidden changes — verified.** Source diffs `9568470..04a93af` and `9901c9c..04a93af` touch only `tests/named_group_join_metadata_event.rs`; final commit `04a93af` only narrows test naming/comment wording. No `tests/harness/**`, CI/gate/wrapper/build invocation/env changes; no role bytes/serde/hash/signing/commit/storage/roster_root/state_hash changes; no discovery receive-path logic hardening.
5. **Planning artifacts — verified.** The prior verifier artifact `gsd/checkpoints/2026-06-17-slice-4-grounded-remediation-verification-9568470.md` is tracked and committed (`080282f`). Final checkpoint/PR-note wording records the TreeKEM claim narrowing and discovery scope clarification.

## Verification evidence

- Read required repo instructions, ADR-0016, Slice 4 packet/spec/PR notes/checkpoints, and prior verifier artifact.
- Inspected source sections for invite issue/join, creator provenance, expected-inviter helpers, `GroupCardPublished`, global/shard/ListedToContacts discovery receive paths, and new public/TreeKEM daemon e2e tests.
- Inspected source diffs `9568470..04a93af` and `9901c9c..04a93af`.
- Ran verifier local checks:
  - `cargo fmt --all -- --check && cargo clippy --all-features --all-targets -- -D warnings && cargo check --workspace --all-targets` — PASS.
  - `cargo nextest run --all-features --test invite_authority` — PASS, 3/3.
  - `cargo nextest run --all-features --all-targets -E 'test(join_result_requires_stored_expected_inviter) or test(treekem_invite_stub_matches_authority_base_hash)'` — PASS, 2/2.

## CI arbiter result

PR #5 head is `04a93afba2cabeb1627c7c484ad16ca9ba6fcd16`.

- PASS: API + GUI Parity Gate, API Coverage Guard, all build matrix jobs including windows/macos/linux, Cargo Audit, Clippy Lint, Coverage Gate, Documentation, Format Check, Multi-Agent Integration, Panic Scanner, Property Tests, Validate release metadata.
- SKIP: Soak Test.
- FAIL accepted only under Jim's live startup-timeout carve-out: `Test Suite`, run `27722220877`, job `82012649602`; sole failure `x0x::named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected`; log line `x0xd pair-alice-12410 did not become healthy within 90s`; summary `1747/1754 tests run: 1746 passed (1 slow), 1 failed, 161 skipped` with 7 not run due fail-fast.
- Caveat: committed `gsd/ci-arbiter.md` has a stricter diff guard naming `fn main`; this checkpoint relies on Jim's live dispatch that startup-timeout-only red is green-of-record and anything else red is a signal.

## Honesty rules

- No-harness-modification: **pass** — no harness/CI/gate/wrapper/build/env changes found.
- Baseline-diff for evidence: **pass with live-carve-out caveat** — the only CI red inspected is a pre-assertion daemon startup timeout; no assertion/logic failure is dismissed.
- Evidence reproducible from branch: **pass** — all inspected code/planning artifacts are committed at the stated heads; verifier commands ran from the build branch without uncommitted wrappers/env.
- Local vs CI consistency: **pass with caveat** — verifier local checks pass; PR #5 is green except the single startup-timeout CI red accepted by live instruction.

## Blockers / open questions

- No remediation-goal blocker found.
- Keep the CI carve-out caveat visible in any PR/readiness handoff; it is an internal arbiter decision, not a claim of fully green upstream CI.
