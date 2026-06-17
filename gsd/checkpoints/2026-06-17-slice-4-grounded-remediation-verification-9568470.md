# Verification — Slice 4 grounded remediation (ADR-0016 Phase 1)

- Date: 2026-06-17
- Verifier: OpenAI GPT-5.5 verifier
- Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
- Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
- Feature branch/head verified: `feat/adr-0016-phase-1-authority-alignment` @ `95684702f8061e42b1b16684cb37f5582dbcee7b`
- Remediation baseline inspected: `8085b340586a94539b1b3cd3e1a19418b493c8fa..95684702f8061e42b1b16684cb37f5582dbcee7b`
- Planning head inspected: `aea64953ff8f776aa43b44ae0a9f71a05844ef80`
- Verdict: **passed**

## Goals result summary

1. **`GroupCardPublished` receive gate role-ified — verified.** `apply_named_group_metadata_event_inner` now uses `info.caller_role(&sender_hex).is_some_and(|role| role.at_least(GroupRole::Admin))`. `caller_role` only returns active-member roles. Existing `card.group_id == info.stable_group_id()` and `card.verify_signature()` checks remain immediately after the role gate.
2. **No remaining creator authority except Slice-5 leave/disband — verified.** Source search found remaining creator uses as API/list/detail provenance, invite base-state provenance, join-request reserved routing, tests/fixtures, and `DELETE /groups/:id` / `treekem_leave_disposition` leave-vs-disband behavior. PR notes and checkpoint right-size creator provenance as best-effort historical, base-state-derived, not authority-bearing, and not tamper-evident.
3. **Real multi-daemon e2e test added — verified as artifact/wiring.** `tests/named_group_join_metadata_event.rs::non_creator_admin_invite_e2e_converges_through_real_daemons` uses `trio_with_extra_config`, creates/promotes a non-creator Admin daemon, issues an invite through the real REST handler, has a separate joiner consume via `POST /groups/join`, and asserts role/`added_by` plus `state_hash`/`roster_root` convergence and creator/inviter split. Local/CI execution is still blocked only by daemon startup-timeout before assertions, not by test assertions.
4. **Expected join-result inviter store moved to `AppState` — verified.** The process-global `EXPECTED_JOIN_RESULT_INVITERS` static is gone. `AppState` owns `expected_join_result_inviters: StdMutex<HashMap<...>>`, initialized in `fn main`. Record/read helpers prune by `JOIN_RESULT_POLL_TIMEOUT`; success/timeout clear paths remain. Synchronous mutex guards are cloned/updated and dropped before any `.await` call.
5. **Planning artifacts/checkpoint — verified.** Planning commit `aea6495` records the grounded remediation in `gsd/checkpoints/2026-06-16-slice-4-invites-per-issuer.md` and includes committed Slice 4 verifier/readiness artifacts. This current verification artifact is intentionally uncommitted per dispatch.
6. **Constraints — verified.** Remediation diff touches only `src/bin/x0xd.rs` and `tests/named_group_join_metadata_event.rs`. No changes to role bytes, serialization, hash/signing/commit/storage formats, `roster_root`/`state_hash` computation, `tests/harness/**`, CI, `.gsd/gate.sh`, daemon wrappers, or build invocation.

## Evidence inspected

- `git status --short --branch` in both worktrees: clean before this verification artifact was written.
- `git rev-parse HEAD` / `git log --oneline`: build head `9568470`; planning head `aea6495`.
- `git diff --name-status 8085b340586a94539b1b3cd3e1a19418b493c8fa..95684702f8061e42b1b16684cb37f5582dbcee7b`: only `src/bin/x0xd.rs` and `tests/named_group_join_metadata_event.rs`.
- Source reads around `GroupCardPublished`, `create_group_invite`, invite join/provenance, expected-inviter helpers, `AppState`, and the new e2e test.
- `gh pr view 5 --repo JimCollinson/x0x --json headRefOid`: PR #5 head is `95684702f8061e42b1b16684cb37f5582dbcee7b`.
- `gh pr checks 5 --repo JimCollinson/x0x`: all checks pass except `Test Suite` and `Multi-Agent Integration`; `Soak Test` skipped.
- Failed-job logs inspected for the two red jobs.

## CI arbiter result

PR #5 counts as green-of-record for this internal gate under Jim's live instruction: **startup-timeout-only red = green-of-record; anything else red = a real signal**.

- PASS: API + GUI Parity Gate, API Coverage Guard, all Builds including `windows-x64`, Documentation, Panic Scanner, Cargo Audit, Clippy Lint, Coverage Gate, Format Check, Property Tests, Validate release metadata.
- SKIP: Soak Test.
- FAIL, accepted under live carve-out only:
  - `Multi-Agent Integration`, run `27691977200`, job `81905098688`: `named_group_import_rejects_tampered_metadata_topic`; `x0xd pair-alice-11680 did not become healthy within 90s`.
  - `Test Suite`, run `27691977893`, job `81905098589`: `forged_member_joined_admin_role_or_secret_is_rejected`; `x0xd pair-alice-44266 did not become healthy within 90s`.

Caveat: `gsd/ci-arbiter.md:17` has a stricter diff guard naming `fn main`, and this remediation necessarily initializes the new `AppState` map inside `fn main`. This verification therefore relies on Jim's live dispatch override for this remediation/checkpoint, not the strict written diff guard alone.

## Honesty rules

- No-harness-modification: **pass** — no harness/CI/gate/wrapper/build-invocation changes.
- Baseline-diff for evidence: **pass with live-instruction caveat** — no current failure is dismissed outside Jim's explicit startup-timeout-only carve-out; no assertion/logic failure is hidden.
- Evidence reproducible from branch: **pass** — inspected committed branch head and GitHub CI logs; no uncommitted script/wrapper/env var required.
- Local vs CI consistency: **pass with caveat** — local focused Rust checks were reported passing by orchestrator; CI format/clippy/build/property and other checks pass; only startup-timeout reds remain under the live carve-out.

## Blockers / open questions

- No remediation-goal blocker found.
- Keep the CI carve-out caveat visible in any PR/readiness handoff; it is an internal arbiter decision, not a claim that upstream CI had zero red jobs.
