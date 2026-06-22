# ADR-0016 Phase 1 Slice 7 — adversarial blocker at 1c3f17a

Date: 2026-06-22
Role: Orchestrator checkpoint
Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
Head reviewed: `1c3f17a58c94c04f4099014bfad121f04dc1b904`
Review range: Slice 7 `b16c34c62e98933716fda1b1434d8961f6ec168d..1c3f17a58c94c04f4099014bfad121f04dc1b904`; integrated Phase 1 `upstream/main...1c3f17a58c94c04f4099014bfad121f04dc1b904`

## Status

Initially blocked — final adversarial review returned `NOT-READY` with unresolved HIGH findings. Jim then dispositioned both HIGH findings on 2026-06-22: defer the card-bound discovery/request-access limitation to Phase 2 / maintainer follow-up, and accept the integrated startup-timeout carve-out after base-vs-branch reproduction. Craft Review and clean-context still needed to complete the phase gate; this file records the adversarial finding and Jim's disposition.

## CI arbiter classification before adversarial

PR #5 raw CI status at `1c3f17a` was red, with all non-daemon jobs green and two daemon-startup health-timeout failures:

- CI run `27968200087`, job `82767397265` (`Test Suite`): `x0x::named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected` failed at `tests/harness/src/cluster.rs:68:17` with `x0xd pair-alice-60704 did not become healthy within 90s`; summary 1682 passed, 1 failed, 164 skipped, 88 not run due fail-fast.
- Integration & Soak run `27968201873`, job `82767406138` (`Multi-Agent Integration`): `x0x::named_group_integration named_group_admin_disband_propagates_to_peer_after_creator_delete_409` failed at `tests/harness/src/cluster.rs:68:17` with `x0xd pair-alice-24987 did not become healthy within 90s`; summary 2 passed, 1 failed, 24 not run due fail-fast.

This initially matched the startup-timeout signature/isolation portions of `gsd/ci-arbiter.md`; adversarial flagged that the integrated branch-level diff guard is ambiguous/void because Phase 1 changes touch `src/bin/x0xd.rs` initialization inside `async fn main`.

## Adversarial verdict

Reviewer: `adversarial` agent (`gpt-5.5`; implementer model/provider not provided, so cross-provider independence could not be proven).

Verdict: `NOT-READY`.

### HIGH — Non-creator Admin request-access/discovery path still depends on creator/`owner_agent_id`

Evidence called out by adversarial:

- `src/groups/mod.rs:969-983` builds cards with `owner_agent_id` as active legacy owner, otherwise creator.
- `src/bin/x0xd.rs:14633-14639` imports a card by parsing `card.owner_agent_id`.
- `src/bin/x0xd.rs:14691-14697` clears the stub roster and inserts only `card.owner_agent_id` as Admin.
- `src/bin/x0xd.rs:9194-9199` rejects `JoinRequestApproved` unless the approving actor is already an active Admin in the local stub roster.

Adversarial concern: a promoted non-creator Admin can sign/publish a discoverable/request-access card, but a requester importing that card seeds the local stub with only creator/`owner_agent_id`; when the promoted Admin approves the join request, requester-side apply rejects the valid signed commit because the approving actor is not in the stub roster.

Recommended remediation from adversarial: seed imported stubs from authoritative card signer/base-roster semantics, or extend cards to carry enough signed authority/roster state; add a regression for non-creator Admin publishes/imports card, requester submits request, non-creator Admin approves, requester applies approval.

Jim disposition, 2026-06-22: **deferred to Phase 2 / maintainer follow-up; do not fix in Phase 1**. This is to be surfaced in the PR description's "deferred to Phase 2" section alongside delegated-ban and KeyPackage distribution. The framing for David: member-side convergence and any-Admin admission hold through the invite/base-roster path; this is a pre-existing card-bound discovery/request-access stub limitation and the fix belongs to Phase 2 after a #107 sketch, not to the Phase 1 authority-alignment PR.

### HIGH — Final integrated CI is raw red, and the written carve-out is not safely satisfied for the integrated branch

Evidence called out by adversarial:

- `gsd/ci-arbiter.md` diff guard says the carve-out requires no change in `src/bin/x0xd.rs` to `fn main`, serve/startup sequence, `/health`, or node/transport/bootstrap initialization.
- Integrated Phase 1 diff touches `async fn main`; current head includes AppState initialization at `src/bin/x0xd.rs:1648`: `expected_join_result_inviters: StdMutex::new(HashMap::new()),`.
- PR #5 checks are raw red in `Test Suite` and `Multi-Agent Integration`.

Recommended remediation from adversarial: get a true green CI run, or get Jim's explicit acceptance of an updated integrated-branch carve-out with base reproduction for both failures and documented rationale that the `fn main` AppState field initialization is non-startup/non-health.

Jim disposition, 2026-06-22: **accept the integrated carve-out if base-vs-branch reproduction at `1c3f17a` shows base reproduces the 90s startup timeout and branch is no worse**. The `async fn main` hunk is the inert AppState field initialization `expected_join_result_inviters: StdMutex::new(HashMap::new())`, not startup/health/network/bootstrap behavior.

Base-vs-branch reproduction performed after freeing scratch build space:

- Base `189b89c0aadb25a1458752fdec040d01df9d2d66`, command `cargo test --all-features --test named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected`: failed before assertions at `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-43735 did not become healthy within 90s`; 0 passed, 1 failed, 5 filtered out, finished in 90.31s.
- Branch `1c3f17a58c94c04f4099014bfad121f04dc1b904`, same command: failed before assertions at `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-47536 did not become healthy within 90s`; 0 passed, 1 failed, 7 filtered out, finished in 90.25s.
- Branch `1c3f17a`, command `cargo test --all-features --test named_group_integration named_group_admin_disband_propagates_to_peer_after_creator_delete_409 -- --ignored`: failed before assertions at `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-26163 did not become healthy within 90s`; 0 passed, 1 failed, 26 filtered out, finished in 90.22s.
- Base `189b89c` does not contain the renamed disband test; its direct predecessor is `named_group_creator_delete_propagates_to_peer`. Command `cargo test --all-features --test named_group_integration named_group_creator_delete_propagates_to_peer -- --ignored`: failed before assertions at `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-22805 did not become healthy within 90s`; 0 passed, 1 failed, 22 filtered out, finished in 90.19s.

Conclusion: branch is no worse than base/equivalent for both CI-failing surfaces; PR #5 remains raw red, but internally accepted under the Jim-approved startup-timeout carve-out with this recorded evidence.

### MEDIUM — Some signed metadata apply paths still gate on transport sender, not just signed commit authority

Evidence called out by adversarial:

- `src/bin/x0xd.rs:8915-8919` (`MemberRoleUpdated`)
- `src/bin/x0xd.rs:9194-9199` (`JoinRequestApproved`)
- `src/bin/x0xd.rs:9470-9475` (`GroupMetadataUpdated`)
- `src/bin/x0xd.rs:9438-9443` (`GroupCardPublished`)

Concern: relayed/catch-up delivery of exact signed events can be rejected when the transport sender is not the original actor.

### LOW — Residual current-doc “owner roster” wording remains in required surfaces

Evidence called out by adversarial:

- `docs/api-reference.md:463-468`
- `docs/primers/groups.md:58-62`

Recommended remediation: qualify as historical x0x 0.21.0 language or update to authority/Admin wording.

## Test-quality note from adversarial

Good coverage exists for role parsing, last-admin invariant, CLI help, API manifest, and non-creator Admin invite paths. Missing critical coverage: non-creator Admin + discoverable/request-access card import + approval apply. Property tests mostly exercise library/helper semantics, not the actual card-import/request-access HTTP path.

## Overstatement list

- Do not call the branch PR-ready or final integrated ready.
- Do not call raw PR #5 CI green; at most it is raw red with a disputed internal carve-out.
- Do not claim Slice 7 done: Craft Review and clean-context are pending, and adversarial HIGH findings are unresolved.

## Recommended next checkpoint

Jim decision/remediation planning is needed before continuing:

1. Approve a remediation slice for the non-creator Admin discovery/request-access path and associated regression, plus the low docs wording cleanup; then rerun mandatory Rust checks, code review, verifier, CI classification, adversarial, Craft Review, and clean-context.
2. Separately decide whether the integrated CI startup-timeout carve-out can cover the existing Phase 1 `async fn main` AppState field initialization, or require a true green PR #5 run/base reproduction before readiness.
