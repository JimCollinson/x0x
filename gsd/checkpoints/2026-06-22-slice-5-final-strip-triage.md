# Checkpoint — Slice 5 final-strip triage

- Date: 2026-06-22
- Slice/question: ADR-0016 Phase 1 Slice 5 — final-strip nit triage
- Prepared by: OpenCode operative (`openai/gpt-5.5`)
- Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
- Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
- Starting build head: `f1ecb9f2d2719f4afbab72223cc7abe570db8570`
- Resulting build commits:
  - `ea86d235a8b38fccb58d29959e254b5fce1e97c9` (`fix(adr-0016-phase-1): guard ban group writes`)
  - `df6637509cbb4f640746c218b92d50cf3af7b6c2` (`fix(adr-0016-phase-1): keep ban write under guard lock`)
- Status: implementation, local gates, PR #5 CI-arbiter classification, and independent final gates complete. Build branch pushed to Jim's fork only; no upstream push and no PR creation/edit action taken.

## Approved nit buckets

1. **Ours + cheap** — non-TreeKEM `ban_group_member` post-rotate write was clean to substitute, but the first substitution in `ea86d23` dropped and reacquired the `named_groups` write guard. Independent code review found that this introduced a same-group lost-update window against unguarded local mutators. Follow-up commit `df66375` resolves it by sharing the `store_named_group_info` terminality check through a locked helper and applying the ban update while the original `named_groups` write guard is still held. If the store guard rejects because the group became withdrawn, the handler returns `409 group is withdrawn` and does not save/publish.
2. **Pre-existing / separate subsystem** — raw `POST`/`DELETE /mls/groups/:id/members` endpoints are outside named-group terminality. They use separate `state.mls_groups`, which disband wipes, and cannot resurrect a named group. Recorded in `gsd/plan/phase-1-pr-notes.md` for David; not fixed here.
3. **Cosmetic/docs-only** — long API/docs wording and CLI alias cleanup remain deferred to Slice 7.

## Scope notes

- TreeKEM direct ban was left unchanged. It uses the TreeKEM durable persist path and is not the non-TreeKEM post-rotate raw GSS write covered by this final-strip nit.
- No changes to tests/harness, CI workflows, `.gsd/gate.sh`, daemon wrappers, build invocation, environment setup, Cargo files, wire/commit/hash logic, or storage formats.

## Evidence

Mandatory Rust checks run in the build worktree after the Rust edit:

1. `cargo fmt --all` — PASS after final edit.
2. `cargo clippy --all-features --all-targets -- -D warnings` — PASS after final edit.
3. `cargo check --workspace --all-targets` — PASS after final edit.

Local fast gate / GSD pre-push hook during fork push:

- `cargo fmt --all -- --check` — PASS.
- `cargo clippy --all-targets --all-features -- -D warnings` — PASS.
- Push target: `fork/feat/adr-0016-phase-1-authority-alignment` only.

Full suite run by orchestrator at final build commit `df6637509cbb4f640746c218b92d50cf3af7b6c2`:

- `cargo nextest run --all-features --workspace` — raw FAIL, classified as the known startup-timeout environmental red under `gsd/ci-arbiter.md`.
- Summary: `1759/1766 tests run: 1758 passed, 1 failed, 164 skipped`; `7/1766` not run due fail-fast.
- Failing test: `x0x::named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected`.
- Failure site/signature: `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-11342 did not become healthy within 90s`.
- Signature: PASS — daemon startup health-timeout at harness bring-up, before Slice 5 assertions.
- Isolation: PASS — one timed-out test, within the `<= 3` arbiter threshold.
- Diff guard: PASS for final-strip commits — `f1ecb9f..df66375` touches only `src/bin/x0xd.rs` and changes the non-TreeKEM ban write path plus its local named-group store helper; no `tests/harness/**`, CI, startup/health/network/bootstrap/presence, daemon wrapper, or build invocation changes.
- Base proof: PASS — the same focused test has already been reproduced at clean base `189b89c0aadb25a1458752fdec040d01df9d2d66` with the same 90s startup-timeout signature (`x0xd pair-alice-55461 did not become healthy within 90s` in `2026-06-21-slice-5-final-remediation-blocker-1fa5f23.md`, and `x0xd pair-alice-34154 did not become healthy within 90s` in `2026-06-21-slice-5-exact-head-ci-classification-22fe1ed.md`).
- Test side effect: local run dirtied tracked `audit.jsonl`; orchestrator restored it before further work and did not commit it.

PR #5 CI arbiter at final build commit `df6637509cbb4f640746c218b92d50cf3af7b6c2`:

- PR #5 head: `df6637509cbb4f640746c218b92d50cf3af7b6c2`.
- Build run `27938381337` — completed success.
  - `Validate release metadata` job `82665524153` — PASS.
  - `Build linux-x64-gnu` job `82665571509` — PASS.
  - `Build linux-x64-musl` job `82665571548` — PASS.
  - `Build linux-arm64-gnu` job `82665571611` — PASS.
  - `Build macos-x64` job `82665571527` — PASS.
  - `Build macos-arm64` job `82665571528` — PASS.
  - `Build windows-x64` job `82665571489` — PASS.
- Security Audit run `27938381341` — completed success.
  - `Cargo Audit` job `82665523996` — PASS.
  - `Panic Scanner` job `82665523980` — PASS.
- CI run `27938381445` — completed raw failure, accepted under `gsd/ci-arbiter.md` startup-timeout carve-out.
  - `Format Check` job `82665524422` — PASS.
  - `Clippy Lint` job `82665524376` — PASS.
  - `Documentation` job `82665524332` — PASS.
  - `API + GUI Parity Gate` job `82665524367` — PASS.
  - `Coverage Gate` job `82665524368` — PASS.
  - `Test Suite` job `82665524359` — raw FAIL: `forged_member_joined_admin_role_or_secret_is_rejected`, `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-61704 did not become healthy within 90s`; summary `1758/1765 tests run: 1757 passed (1 slow), 1 failed, 164 skipped`.
- Integration & Soak run `27938381397` — completed raw failure, accepted under `gsd/ci-arbiter.md` startup-timeout carve-out.
  - `API Coverage Guard` job `82665524078` — PASS.
  - `Property Tests` job `82665524121` — PASS.
  - `Soak Test` job `82665524912` — SKIPPED, expected for non-schedule run.
  - `Multi-Agent Integration` job `82665524079` — raw FAIL: `named_group_admin_disband_propagates_to_peer_after_creator_delete_409`, `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-13090 did not become healthy within 90s`; summary `3/27 tests run: 2 passed, 1 failed, 0 skipped`; later integration steps skipped due fail-fast.
- Arbiter classification: internally green by carve-out, not raw GitHub green. Signature PASS (all failures are daemon startup health-timeouts at harness bring-up); isolation PASS (2 timed-out tests across all jobs, within `<= 3`); diff guard PASS (`f1ecb9f..df66375` changes only `src/bin/x0xd.rs` helper/store path and non-TreeKEM ban store call, with no tests/harness, CI, startup/health/network/bootstrap/presence, daemon wrapper, build invocation, or Cargo changes).

## Files changed

- Build: `src/bin/x0xd.rs`
- Planning: `gsd/plan/phase-1-pr-notes.md`, this checkpoint file

## Review findings addressed

- `@codereviewer` reviewed `ea86d23` and found one MEDIUM issue: the drop/reacquire window around `store_named_group_info(&state, &id, next)` could overwrite another same-group local update from an unguarded `named_groups` writer.
- Disposition: fixed in `df66375` by introducing `store_named_group_info_locked(...)` and calling it while `ban_group_member` still holds the original `named_groups` write guard. The async wrapper remains for existing paths that intentionally acquire the lock themselves.

## Final independent gates

- `@codereviewer` on `df6637509cbb4f640746c218b92d50cf3af7b6c2`: PASS. No code-quality or safety blockers; confirmed final-strip diff touches only `src/bin/x0xd.rs`; noted earlier full-hash transcription error, corrected above.
- `@verifier` on `df6637509cbb4f640746c218b92d50cf3af7b6c2`: PASS. Goal achieved; no gaps/blockers; full nextest remains raw red only under startup-timeout carve-out.
- `@adversarial`: initial NOT-READY while PR #5 checks were still in progress; re-check after completed CI is READY-WITH-NITS. Prior HIGH CI-not-final finding resolved by run/job evidence above. Remaining nit: no focused regression specifically exercising the non-TreeKEM ban locked-store terminality guard; carry as test-quality/backlog, not a blocker.
- `@craft`: PASS. No CONFORMANCE, SIMPLICITY, or NIT findings.
- `@cleancontext`: Concerns, no blockers. Cold reviewer could understand and exercise the final-strip change from repo/planning files; concerns are raw-red CI requiring internal carve-out, full nextest dirtying `audit.jsonl` before restoration, and lack of a dedicated final-strip regression.

## Blockers / risks

- No blocker found for the approved final-strip implementation after final CI-arbiter classification and adversarial re-check.
- Remaining risk/caveat is the explicitly recorded raw MLS endpoint separate-subsystem note, the known startup-timeout flake classification, the lack of a dedicated final-strip ban-guard regression, and Slice 7 wording backlog.
