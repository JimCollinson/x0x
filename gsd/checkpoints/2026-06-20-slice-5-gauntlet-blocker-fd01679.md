# Checkpoint — Slice 5 gauntlet blocker at `fd01679`

- Date: 2026-06-20
- Slice/question: ADR-0016 Phase 1 Slice 5 — leave/disband split, withdrawn shell, TreeKEM terminality, stale ignored-test alignment
- Prepared by: OpenCode orchestrator
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `fd01679e73dd4b012882d6825f0ce869463df079`
- Status: **Blocked — adversarial HIGH findings**
- Meaningful work-unit? Yes — non-trivial Rust API/CLI/group-authority behavior in a shared/upstream-bound repo.
- Review cadence: per-unit/integrated gauntlet run; no Jim waiver or review deferral.

## What happened

- Remediation through `78abe25` preserved Slice 5 semantics while blocking withdrawn TreeKEM snapshot/journal persistence and stale journal replay over durable withdrawn records.
- Stale ignored daemon tests were aligned to current Slice 5 semantics in:
  - `c199583 test(adr-0016-phase-1): update named group lifecycle semantics`
  - `072d7f7 test(adr-0016-phase-1): align stale leave integration tests`
  - `fd01679 test(adr-0016-phase-1): align invite rejoin leave semantics`
- Build branch was pushed to Jim's fork for PR #5 CI.
- CI, Craft Review, Clean-context, and Adversarial Review were run/queried for the current head.

## Evidence

CI arbiter / green of record:

- Location: PR #5, <https://github.com/JimCollinson/x0x/pull/5>
- Current check summary from `gh pr checks 5 --repo JimCollinson/x0x`:
  - Passing: API + GUI Parity Gate, API Coverage Guard, all Build jobs, Cargo Audit, Clippy Lint, Coverage Gate, Documentation, Format Check, Multi-Agent Integration, Panic Scanner, Property Tests, Validate release metadata.
  - Skipped by workflow: Soak Test.
  - Raw red: Test Suite.
- Current Test Suite rerun:
  - Run: `27873014199`
  - Job: `82489576165`
  - Summary: `1758/1765 tests run: 1757 passed, 1 failed, 164 skipped`
  - Failing test: `x0x::named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected`
  - Verbatim line: `x0xd pair-alice-41777 did not become healthy within 90s`
  - Classification: startup-timeout-only red in the current attempt. This appears to satisfy `gsd/ci-arbiter.md` signature/isolation, but adversarial review challenged the written diff-guard applicability because `src/bin/x0xd.rs` includes an `async fn main()` hunk.
- Earlier Test Suite attempt in the same run:
  - Job: `82488830294`
  - Had two failures: the same startup-timeout signature plus `x0x::peer_lifecycle_integration direct_send_with_require_ack_round_trips_to_live_peer` timing out on `POST /direct/send`.
  - Follow-up local targeted command passed: `cargo nextest run --all-features --test peer_lifecycle_integration -E 'test(direct_send_with_require_ack_round_trips_to_live_peer)'`.
  - Because current Test Suite attempt no longer has that non-carve-out failure, it is not the current CI blocker, but the earlier failure remains recorded.

Local fast gate / checks:

- No committed `.gsd/gate.sh` exists in the build worktree.
- Pre-push hook/gate on push passed `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- Local mandatory/current checks previously recorded as pass at `fd01679`:
  - `cargo fmt --all`
  - `cargo clippy --all-features --all-targets -- -D warnings`
  - `cargo check --workspace --all-targets`
  - focused stale ignored named-group tests for `join_via_invite`, `leave`, `rejoin_after_leave`, and `full_lifecycle`
  - `cargo nextest run --all-features -E 'test(leave) or test(disband) or test(withdraw)'` — 23/23 pass
- Clean-context independently reran focused checks with a separate target dir and got:
  - `cargo fmt --all -- --check` — PASS
  - `cargo clippy --all-targets --all-features -- -D warnings` — PASS
  - `cargo nextest run --all-features -E 'test(leave) or test(disband) or test(withdraw)'` — PASS, 23/23
  - `cargo test --all-features --bin x0xd treekem` — PASS, 23/23
  - `cargo test --all-features --bin x0xd withdrawn` — PASS, 10/10
  - `cargo check --workspace --all-targets` — PASS
  - ignored multi-daemon disband propagation proof failed before the test body with startup timeout: `x0xd pair-alice-28467 did not become healthy within 90s`

## Review findings

Clean-context test:

- Reviewer/tool: `cleancontext` agent
- Result: **Concerns**
- Summary:
  - No Slice 5 behaviour blocker found.
  - CI is raw red but appears within declared startup-timeout carve-out.
  - Local multi-daemon propagation proof did not exercise assertions due daemon startup timeout.
  - No dedicated Slice 5 packet found in `gsd/packets/`; scope is recoverable from plan/checkpoints/PR notes.
  - Adjacent stale ADR-0016 wording remains but is tracked for Slice 7.

Adversarial review:

- Reviewer/tool: `adversarial` agent, `openai/gpt-5.5`.
- Required? Yes — meaningful upstream-bound work-unit; no waiver/deferral.
- Result: **NOT-READY**
- Blocking findings:
  1. **HIGH:** Live/keyed groups can be terminally wiped by a withdrawn `GroupCard` alone, bypassing signed withdrawal commit validation.
     - Review anchors:
       - `src/bin/x0xd.rs:14225-14233` applies a withdrawn card to local group info and calls `retain_withdrawn_group_shell`.
       - `src/bin/x0xd.rs:11848-11855` predicate checks card supersession and roster admin but not a signed terminal `GroupStateCommit`.
       - `src/bin/x0xd.rs:637-642` allows card supersession by revision or equal revision/newer timestamp.
       - `tests/named_group_integration.rs:72-81` fabricates a withdrawn card state hash and expects live key material to be wiped.
     - Required disposition: require a verified signed terminal `GroupStateCommit` before a live/keyed local group is marked withdrawn/wiped, or obtain an explicit Jim/maintainer decision that card-only terminal marking of live keyed groups is acceptable.
  2. **HIGH:** CI green-of-record is not established under the written carve-out because adversarial found an `async fn main()` diff hunk in `src/bin/x0xd.rs`, while `gsd/ci-arbiter.md` says the startup-timeout carve-out requires no `fn main` change.
     - Required disposition: get a fully green Test Suite, or produce human-approved arbiter clarification plus proof that the `fn main` hunk cannot affect startup/health for this timeout.
- Additional finding:
  - **MEDIUM:** In-flight TreeKEM decrypt can clone an `Arc`, decrypt, log persistence failure after withdrawal wins, and still return plaintext. Suggested remediation: serialize/recheck terminality after decrypt and before returning plaintext, and add a race regression.

Craft Review:

- Reviewer/tool: `craft` agent, `openai/gpt-5.5`.
- Required? Yes — upstream-bound work-unit.
- Verdict: **Pass for Craft**
- CONFORMANCE findings: none.
- Non-blocking notes:
  - SIMPLICITY: optional cleanup of redundant internal CLI alias wrapper / `DISBAND_VERB` if not intentionally retained.
  - NIT: optional shortening of the `DELETE /groups/:id` API table row to match nearby table style.

## Honesty rules check

- No-harness-modification: PASS — latest test remediation touched `tests/named_group_integration.rs` only; no changes to `tests/harness/**`, CI, `.gsd/gate.sh`, daemon wrappers, build invocation, or environment.
- Baseline-diff for evidence: CONCERN — current raw CI red is startup-timeout-only in the latest attempt and covered by declared arbiter if its diff guard holds. Earlier rerun had a non-carve-out direct-send timeout that disappeared on the next CI attempt and passed locally; it is recorded, not used as current green evidence.
- Evidence reproducible-from-branch: CONCERN — build-branch code evidence is reproducible; this new checkpoint and the 2026-06-20 verification artifact are planning-branch artifacts and currently uncommitted.
- Local vs CI consistency: CONCERN — local focused checks pass, CI Test Suite remains raw red with startup-timeout signature.

## Status

Slice 5 is **blocked**. Adversarial returned `NOT-READY` with unresolved HIGH findings. Under GSD rules this blocks calling the work-unit done or PR-ready.

## Recommended next step

Create a remediation slice for Jim/maintainer decision:

1. Decide whether withdrawn card import may ever terminally mark/wipe a live/keyed local group without a signed terminal withdrawal commit.
   - If no: change card import so card-only withdrawal can supersede discovery/keyless stubs but cannot wipe live/keyed groups; require the signed withdrawal commit path for live/keyed terminality.
   - If yes: record the decision explicitly because it weakens the signed-chain terminality model.
2. Decide how to handle in-flight TreeKEM decrypt/encrypt vs withdrawal terminality.
   - Likely fix: recheck durable/local withdrawal after decrypt and before returning plaintext; treat terminality as conflict.
3. Resolve CI arbiter status.
   - Prefer a fully green Test Suite rerun.
   - Otherwise clarify/approve whether the current `src/bin/x0xd.rs` `fn main` hunk is outside the intended startup diff guard.

Do not call Slice 5 done, do not mark PR #5 ready, and do not open any upstream PR until these are resolved and adversarial re-review passes.
