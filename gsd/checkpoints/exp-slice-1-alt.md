# Checkpoint — exp/slice-1-alt: Slice 1 last-admin invariant

- Date: 2026-06-13
- Plan: ADR-0016 Phase 1 authority alignment
- Slice: 1 of 7 — last-admin invariant
- Implementation branch: `exp/slice-1-alt`
- Implementation commit: `ed2f9a3` (`feat(adr-0016-slice-1): enforce last-admin invariant`)
- Planning branch: `gsd/adr-0016-planning`
- Status: complete; implementation pushed to `origin/exp/slice-1-alt`

## Evidence item 1 (binding addendum): authority/apply-path map

The map below enumerates group-state mutation surfaces verified for this slice. The shared proposed-roster helper for daemon-side mutations is `proposed_group_after` in `src/bin/x0xd.rs`; the shared invariant helper is `validate_last_admin_invariant` in `src/groups/state_commit.rs`. Gossip apply paths pass the proposed roster to the `validate_apply` choke-point via the new `proposed_members_v2` argument.

### Shared seams

- `src/groups/state_commit.rs`: `validate_apply(ctx, commit, action_kind, proposed_members_v2)` now enforces `validate_last_admin_invariant(commit.withdrawn, proposed_members_v2)` after signature, chain, authority, and roster-root checks.
- `src/bin/x0xd.rs`: `apply_stateful_event_to_group` clones the current group with `proposed_group_after`, applies the event mutation to the clone, passes `&next.members_v2` to `validate_apply`, then finalizes the applied commit only after validation succeeds.
- `src/groups/mod.rs`: `GroupInfo::apply_commit` passes its current roster as the proposed roster for direct library commit validation. The daemon gossip path supplies the post-mutation roster before finalization for stateful event application.
- REST-friendly prechecks for operations that can zero the active admin set call `require_proposed_group_has_admin`, which delegates to `validate_last_admin_invariant` over the same cloned proposed roster shape used by gossip apply.

### REST handlers

| Surface | Proposed roster computation | Choke/precheck | Shared-helper confirmation |
|---|---|---|---|
| `remove_named_group_member` | `proposed_group_after(info, remove_member)` before mutation | `require_proposed_group_has_admin`; remote receivers later use `validate_apply` in `MemberRemoved` | Uses shared clone/mutate helper and shared invariant helper. |
| `remove_treekem_named_group_member` | `proposed_group_after(info, remove_member)` before mutation | `require_proposed_group_has_admin`; remote receivers later use `validate_apply` in `MemberRemoved` | Uses shared clone/mutate helper and shared invariant helper. |
| `update_member_role` | For admin-or-higher demotions only, `proposed_group_after(info, set_member_role)` before mutation | `require_proposed_group_has_admin`; remote receivers later use `validate_apply` in `MemberRoleUpdated` | Uses shared clone/mutate helper and shared invariant helper. Promotions/non-demotions cannot reduce the admin count. |
| `ban_group_member` | `proposed_group_after(info, ban_member)` before mutation | `require_proposed_group_has_admin`; remote receivers later use `validate_apply` in `MemberBanned` | Uses shared clone/mutate helper and shared invariant helper. |
| `ban_treekem_group_member` | `proposed_group_after(info, ban_member)` before mutation | `require_proposed_group_has_admin`; remote receivers later use `validate_apply` in `MemberBanned` | Uses shared clone/mutate helper and shared invariant helper. |
| Add-member / approve-join paths | Add active member or consume request; cannot reduce active admins | No REST last-admin precheck needed; receivers validate the resulting proposed roster through `validate_apply` | No divergent roster helper needed for REST; gossip apply uses `apply_stateful_event_to_group`. |
| Unban | Changes a banned entry to active member; cannot reduce active admins | No REST last-admin precheck needed; receivers validate through `validate_apply` | Gossip apply uses `apply_stateful_event_to_group`. |
| Policy update / group metadata update | Roster unchanged | No REST last-admin precheck needed; receivers validate unchanged proposed roster through `validate_apply` | Gossip apply uses `apply_stateful_event_to_group`. |
| Join-request create/reject/cancel | `join_requests` only; roster unchanged | No REST last-admin precheck needed; receivers validate unchanged proposed roster through `validate_apply` | Gossip apply uses `apply_stateful_event_to_group`. |
| Group deletion / withdrawal | Terminal `withdrawn = true` state | Exempt by `validate_last_admin_invariant(withdrawn = true, ...)`; receivers validate through `validate_apply` | Gossip apply uses `apply_stateful_event_to_group`; withdrawal remains the last-admin exit valve. |
| Group creation / genesis seeding | Creates initial roster; existing behavior still seeds legacy Owner | Cannot zero an existing live admin set; Slice 2 changes genesis to Admin | Out of scope for Slice 1. |

### Gossip apply arms

Every stateful metadata arm that applies a signed commit continues through `apply_stateful_event_to_group`, so each computes the proposed roster on a cloned group, passes it to `validate_apply`, and only then finalizes:

- `MemberAdded`: add member; proposed roster includes the added active member.
- `MemberRemoved`: remove/self-leave; proposed roster marks the member removed and is rejected if it leaves a live group with zero active admins.
- `GroupDeleted`: terminal withdrawal; proposed roster may be zero-admin but is exempt because `commit.withdrawn` is true.
- `PolicyUpdated`: roster unchanged; proposed roster is the current roster.
- `MemberRoleUpdated`: proposed roster has the new role; demoting the final admin is rejected.
- `MemberBanned`: proposed roster marks the member banned; banning the final admin is rejected.
- `MemberUnbanned`: proposed roster restores the member as active; cannot reduce active admins.
- `JoinRequestCreated`: request set changes only; roster unchanged.
- `JoinRequestApproved`: approval/request state changes and any associated membership mutation are evaluated on the cloned proposed group before finalization.
- `JoinRequestRejected`: request state changes only; roster unchanged.
- `JoinRequestCancelled`: request state changes only; roster unchanged.
- `GroupMetadataUpdated`: roster unchanged.

Non-commit metadata paths such as group-card publication, secure-share delivery, and joiner-authored `MemberJoined` do not directly apply a membership-changing signed state commit. Where a later authority-authored commit is produced, remote application is covered by the stateful apply seam above.

No group-state commit application path was found that bypasses `validate_apply` for gossip/direct application. REST authoring paths that can zero the active admin set are guarded by the exact friendly precheck before local mutation.

## What changed

- `src/groups/state_commit.rs`
  - Added `has_active_admin_or_higher` and `validate_last_admin_invariant`.
  - Extended `validate_apply` to accept the proposed post-mutation roster and reject live zero-admin states with `ApplyError::Invariant`.
  - Added `last_admin_*` unit coverage for demotion, removal, ban, withdrawal exemption, legacy Owner normalization/counting, and roster-root/proposed-roster consistency.
- `src/groups/mod.rs`
  - Updated direct `GroupInfo::apply_commit` validation to pass a proposed roster argument.
- `src/bin/x0xd.rs`
  - Added shared proposed-roster and REST conflict helpers.
  - Updated gossip stateful apply to validate the cloned post-mutation roster before finalization.
  - Added exact 409 REST prechecks for remove, TreeKEM remove, ban, TreeKEM ban, and admin demotion.
- `tests/last_admin_rest.rs`
  - Added an integration test asserting the exact `409` JSON body for remove, ban, and demotion of the sole active admin.

## Verification

Required verification was rerun in order after environment remediation:

| Command | Result |
|---|---|
| `cargo fmt --all` | PASS; no output |
| `cargo clippy --all-features --all-targets -- -D warnings` | PASS; `Finished dev profile [unoptimized + debuginfo] target(s) in 0.95s` |
| `cargo check --workspace --all-targets` | PASS; `Finished dev profile [unoptimized + debuginfo] target(s) in 0.37s` |
| `X0XD_TEST_BINARY=<temporary-local-bootstrap-wrapper> cargo nextest run --all-features --workspace` | PASS; `1713 tests run: 1713 passed, 160 skipped` in 140.094s |
| `X0XD_TEST_BINARY=<temporary-local-bootstrap-wrapper> cargo nextest run --all-features -E 'test(last_admin)'` | PASS; `10 tests run: 10 passed, 1855 skipped` in 2.835s |

Additional diagnosis evidence for the cluster setup issue:

- Before remediation, full nextest failed at setup in `named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected` with `[cluster] FATAL: pair-alice-* has zero peers after 30s — mesh is disconnected`.
- The isolated test failed the same way, confirming it was not caused by full-suite interaction.
- With the local verification wrapper, the isolated test passed: `1 test run: 1 passed, 1872 skipped` in 24.827s.

## Environment remediation

No production code or repo test harness was changed for the cluster setup issue.

Diagnosis: the harness starts daemons with `--no-hard-coded-bootstrap`. In `x0xd`, that flag clears `config.bootstrap_peers`. For pair/trio nodes that have an explicit generated local `bootstrap_peers = ["127.0.0.1:..."]`, this prevents the explicit localhost bootstrap from being used. On this machine, mDNS did not form the local mesh without that bootstrap, so `pair-alice-*` remained at zero peers.

Verification setup: a temporary wrapper outside the repo was used via `X0XD_TEST_BINARY`. It preserves `--no-hard-coded-bootstrap` for seed nodes with no explicit `bootstrap_peers`, strips it only for generated configs that contain an explicit local `bootstrap_peers = ...`, and appends `--skip-update-check` to avoid unrelated GitHub rate-limit startup warnings. This provided the intended local bootstrap environment without weakening product tests or changing slice scope.

Recommended follow-up outside this slice: make the test harness/daemon flag interaction reliable in-repo, either by not passing `--no-hard-coded-bootstrap` to nodes with explicit local bootstrap peers or by splitting the daemon flag semantics so local test bootstrap peers are not cleared. That is a harness reliability item, not part of Slice 1.

## Drift vs pinned citations

- Base branch `exp/slice-1-alt` was cut at `189b89c`; upstream main and fork main were verified at `189b89c` before implementation.
- The cited Slice 1 mechanisms were materially unchanged: `validate_apply`, `ApplyError::Invariant`, role rank comparison via `role.at_least(Admin)`, REST remove/ban/role-update handlers, and gossip stateful apply arms.
- Line numbers shifted only due to this slice's edits.

## Deviations / notes

- The packet requested the exact REST body `{"error":"a group must always have at least one admin; make another member an admin first"}`. The new REST prechecks return exactly that JSON body with status `409`.
- The plan's comparison-run neutrality rules were preserved in code, commit message, and this checkpoint.
- Test runs append to tracked `audit.jsonl`; it was restored before commit and was not included.
- The temporary verification wrapper and generated daemon logs/heap dumps were not committed.

## Files changed on implementation branch

- `src/groups/state_commit.rs`
- `src/groups/mod.rs`
- `src/bin/x0xd.rs`
- `tests/last_admin_rest.rs`

## Plan validity / next step

No GSD stop condition fired for the implementation. Slice 1 is complete on `origin/exp/slice-1-alt`. Recommended next step: review this checkpoint and implementation diff, then dispatch Slice 2 if accepted.
