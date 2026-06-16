# Checkpoint — Slice 4 invites per issuer + creator provenance (ADR-0016 Phase 1)

- Date: 2026-06-16
- Slice/question: Slice 4 — Invites per-issuer + creator provenance (R7/R8)
- Prepared by: OpenCode implementer
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `680198b38c55c380bafc8adc3da1ac0a0b2f5607`
- Status: **Blocked — PR #5 CI arbiter red after rerun**

## What changed

- Removed the invite issue-side creator-only gate and made active Admin-or-higher role lookup the issuing authority.
- Preserved per-issuer invite issue/consume/track routing: `invite.inviter` remains the join-result polling and delivery target.
- Added deterministic creator provenance derivation from invite base-state member snapshot so a joiner's `GroupInfo.creator` comes from seeded genesis/base state, never unsigned `invite.inviter` metadata.
- Added fast, gate-runnable coverage for promoted non-creator Admin issuing an invite, plain member rejection, creator-issued invite regression, and base-state creator provenance when inviter differs from creator.

## Commits

- `680198b38c55c380bafc8adc3da1ac0a0b2f5607` — `feat(adr-0016-phase-1): allow admin-issued invites`

## Files changed on feature branch

- `src/bin/x0xd.rs`
- `src/groups/invite.rs`
- `tests/invite_authority.rs`

## Local verification evidence

Mandatory Rust order after code changes:

- `cargo fmt --all` — PASS
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS
- `cargo check --workspace --all-targets` — PASS

Targeted and supporting checks:

- `cargo nextest run --all-features -E 'test(invite)'` — PASS, 25 tests across 69 binaries
- `cargo nextest run --all-features --test invite_authority` — PASS, 3/3
- `cargo nextest run --all-features -E 'test(creator_provenance) or test(invite_authority)'` — PASS, 6/6
- `cargo nextest run --all-features -E 'test(invite) & !binary(named_group_join_metadata_event)'` — PASS, 23 tests
- `git diff --check` — PASS

Pre-push hook evidence on feature branch push:

- `cargo fmt --all -- --check` — PASS
- `cargo clippy --all-targets --all-features -- -D warnings` — PASS

## PR #5 CI arbiter status

Green of record source: PR #5, <https://github.com/JimCollinson/x0x/pull/5>.

Pushed head `680198b` to Jim's fork. Initial PR #5 checks reported two failing jobs:

- `Test Suite`: run `27613332295`, job `81642971059`
  - Failing test: `x0x::named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected`
  - Failure: `x0xd pair-alice-25018 did not become healthy within 90s`
  - This is the enumerated known-flaky test signature in `gsd/ci-arbiter.md`, but the carve-out could not apply because another job was also red.
- `Multi-Agent Integration`: run `27613332279`, job `81642971049`
  - Failing test: `x0x::named_group_integration::named_group_creator_delete_propagates_to_peer`
  - Failure: `x0xd pair-bob-41349 did not become healthy within 90s`
  - This test is outside the enumerated known-flaky set, so the internal arbiter remained red.

Per the packet's CI heads-up, reran failed jobs only, without code, harness, gate, environment, or workflow changes:

- `gh run rerun 27613332295 --failed --repo JimCollinson/x0x`
- `gh run rerun 27613332279 --failed --repo JimCollinson/x0x`

Rerun results:

- `Test Suite`: run `27613332295`, rerun job `81646084682` — FAIL
  - Failing test: `x0x::named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected`
  - Failure: `x0xd pair-alice-42997 did not become healthy within 90s`
  - Summary: `1747/1752 tests run: 1746 passed (1 slow), 1 failed, 161 skipped`; 5 not run due fail-fast.
- `Multi-Agent Integration`: run `27613332279`, rerun job `81646084684` — FAIL
  - Failing test: `x0x::named_group_integration::named_group_creator_delete_propagates_to_peer`
  - Failure: `x0xd pair-alice-20427 did not become healthy within 90s`
  - Summary: `5/24 tests run: 4 passed, 1 failed`; 19 not run due fail-fast.

All other reported PR #5 checks were passing or skipped-by-workflow (`Soak Test`). Current status: **red under normal blocking rule** because `gsd/ci-arbiter.md` only permits the narrow single-job/single-enumerated-test carve-out, and `Multi-Agent Integration` is repeatedly red on a non-enumerated daemon startup timeout.

## Closing creator-authority sweep

Current-head search of `src/bin/x0xd.rs` for remaining creator/inviter comparisons and uses:

- `sender_hex != creator_hex` in the public group-card receive path — creator/provenance/public-card path, not invite authority.
- API output fields rendering `"creator"` — metadata/output only.
- invite join path uses `invite.creator_agent_id_from_base_state()` for creator provenance and `invite.inviter` for routing/result delivery — intended Slice 4 split.
- leave/delete/disband creator checks and `treekem_leave_disposition` tests — Slice 5 scope.
- join-request notification/routing placeholder using `creator_hex` — routing placeholder, not invite issue authority.

No remaining invite issue-side creator authority gate was found.

## Coverage notes

Fast-gate / normal nextest coverage completed:

- promoted non-creator Admin can pass invite issue authorization at the helper/state seam;
- plain `member` cannot issue an invite;
- creator-issued invite path still passes;
- when inviter differs from creator, creator provenance derives from base-state creator rather than unsigned inviter;
- consume-side inviter-admin role check remains the authority at consume/apply.

Maintainer-gate daemon/mesh assertions remain blocked by PR #5 red CI:

- promoted non-creator Admin issuing through the real daemon;
- joiner consuming against issuing daemon;
- join-result polling routing to non-creator inviter;
- creator-issued invite end-to-end.

## Honesty rules check

- No-harness-modification: PASS — no changes to tests/harness, CI workflow, `.gsd/gate.sh`, daemon wrappers, build invocation, or environment.
- Baseline-diff for evidence: CONCERN — no CI failure is being dismissed as environmental/flaky for readiness. The known Test Suite flake matches the enumerated signature, but the repeated Multi-Agent failure is outside the carve-out and remains blocking.
- Evidence reproducible-from-branch: PASS for local checks; CI remains the green of record and is red.
- Local vs CI consistency: CONCERN — local checks pass, but PR #5 CI arbiter is red.

## Review gates

- Clean-context test: Not run — slice is blocked before readiness.
- Adversarial review: Not run — slice is blocked before readiness.
- Craft Review: Not run — slice is blocked before readiness.

## Blocker

PR #5 is red after one failed-job rerun. The blocking failure is a repeated non-enumerated daemon startup health timeout in `Multi-Agent Integration`:

`x0x::named_group_integration::named_group_creator_delete_propagates_to_peer` → `x0xd ... did not become healthy within 90s`.

Under `gsd/ci-arbiter.md`, extending the internal known-flake carve-out or changing CI/harness/gate behavior is not allowed during slice execution. Debugging by changing harness, daemon wrapper, build invocation, CI workflow, environment setup, or `.gsd/gate.sh` is also forbidden.

## Recommended next step

Stop for orchestrator/Jim decision. Options:

1. Treat the repeated `named_group_creator_delete_propagates_to_peer` startup timeout as a separate CI/harness investigation outside Slice 4.
2. Approve a deliberate update to `gsd/ci-arbiter.md` if Jim wants this exact test added to the internal known-flake set (requires reviewed planning/gate change, not implementer discretion).
3. Authorize a focused debug slice for daemon startup health timeouts, with explicit permission boundaries for any harness/CI/environment investigation.

Do not mark Slice 4 Done until PR #5 is green under the existing arbiter or Jim approves a revised arbiter/checkpoint disposition.
