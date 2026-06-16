# Checkpoint — Slice 4 invites per issuer + creator provenance (ADR-0016 Phase 1)

- Date: 2026-06-16
- Slice/question: Slice 4 — Invites per-issuer + creator provenance (R7/R8)
- Prepared by: OpenCode implementer
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `680198b38c55c380bafc8adc3da1ac0a0b2f5607`
- Status: **CI gate satisfied under internal known-mesh-flake carve-out; review gates pending**

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
  - This is a daemon-startup health-timeout at harness bring-up.
- `Multi-Agent Integration`: run `27613332279`, job `81642971049`
  - Failing test: `x0x::named_group_integration::named_group_creator_delete_propagates_to_peer`
  - Failure: `x0xd pair-bob-41349 did not become healthy within 90s`
  - This is a daemon-startup health-timeout at harness bring-up.

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

All other reported PR #5 checks were passing or skipped-by-workflow (`Soak Test`). Current status: **green of record modulo known mesh flake** under `gsd/ci-arbiter.md`'s generalized daemon-startup timeout carve-out.

### Known-flake carve-out invocation

The latest invocation qualifies under the internal arbiter's signature / isolation / diff-guard rule:

- Signature: both red jobs failed only on daemon-startup health-timeouts before any test assertion.
  - `Test Suite`: run `27613332295`, job `81646084682`; failing test `x0x::named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected`; verbatim line: `x0xd pair-alice-42997 did not become healthy within 90s`.
  - `Multi-Agent Integration`: run `27613332279`, job `81646084684`; failing test `x0x::named_group_integration::named_group_creator_delete_propagates_to_peer`; verbatim line: `x0xd pair-alice-20427 did not become healthy within 90s`.
  - No assertion failure, diagnostic-counter mismatch, or timeout inside already-running test logic was reported. Tests not run due fail-fast are not counted as failures.
- Isolation: 2 timed-out tests across all jobs (`<= 3`); the rest of PR #5's reported checks were green or skipped-by-workflow.
- Diff guard: satisfied. Slice 4 changed only:
  - `src/groups/invite.rs` — `SignedInvite::creator_agent_id_from_base_state` and invite provenance tests;
  - `src/bin/x0xd.rs` — invite/join-result/member-join handler hunks in `create_group_invite`, `join_group_via_invite`, `handle_join_result_message`, and `apply_named_group_metadata_event_inner` wording/MemberJoined path;
  - `tests/invite_authority.rs` — Slice 4 invite authority/provenance coverage.
  It changed nothing under `tests/harness/`, nothing under `src/network*`, `src/bootstrap*`, or `src/presence*`, and no `src/bin/x0xd.rs` startup/health code (`fn main`, serve/startup sequence, `/health`, node/transport/bootstrap initialization).
- Upstream provenance: both failing tests exist at untouched base `189b89c`:
  - `tests/named_group_join_metadata_event.rs:556`
  - `tests/named_group_integration.rs:1296`

Determination: **green of record (modulo known mesh flake)** for the internal PR #5 arbiter. This records and reasons about the red CI jobs; it does not claim upstream CI passed, and the harness flake remains flagged to David.

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

Maintainer-gate daemon/mesh assertions remain affected by the pre-existing startup-health flake and are not claimed as cleanly exercised by the failing CI jobs:

- promoted non-creator Admin issuing through the real daemon;
- joiner consuming against issuing daemon;
- join-result polling routing to non-creator inviter;
- creator-issued invite end-to-end.

## Honesty rules check

- No-harness-modification: PASS — no changes to tests/harness, CI workflow, `.gsd/gate.sh`, daemon wrappers, build invocation, or environment.
- Baseline-diff for evidence: PASS with recorded carve-out — the CI failures are classified only under the approved internal arbiter rule because they match the daemon-startup timeout signature, are isolated to 2 tests, and the Slice 4 diff guard shows no startup/health/networking changes. Both failing tests exist at base `189b89c`.
- Evidence reproducible-from-branch: PASS for local checks; PR #5 remains the green of record and is satisfied modulo the recorded known mesh flake under the internal arbiter.
- Local vs CI consistency: PASS with caveat — local checks pass; PR #5 is green of record modulo the recorded known mesh flake under the internal arbiter.

## Review gates

- Clean-context test: Not run — deferred until behaviour is complete enough to exercise from repo/docs / PR-readiness.
- Adversarial review: Pending — required before accepting Slice 4.
- Craft Review: Pending — required before accepting Slice 4.

## Current gate status

Slice 4's CI arbiter gate is satisfied under the approved generalized internal known-flake carve-out. No harness, CI workflow, `.gsd/gate.sh`, daemon wrapper, build invocation, or environment setup was changed.

## Recommended next step

Run independent code review, verifier, adversarial review, and Craft Review before accepting Slice 4.
