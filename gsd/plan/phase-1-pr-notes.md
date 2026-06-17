# ADR-0016 Phase 1 PR notes

These are running notes for the eventual PR description / maintainer handoff. They are not a PR; PR creation remains Jim's explicit gate.

## Reserved-role apply / authoring split

REST role assignment is restricted to `admin`/`member` (`GroupRole::assignable_from_name`), but the signed `MemberRoleUpdated` apply path rejects only `Owner`; validly signed `Moderator`/`Guest` commits remain accepted.

Rationale for David: `Moderator`/`Guest` grant no admin authority because the authority threshold is `role.at_least(Admin)`. An active member of any role is member-level, which is expected and not an admin escalation. The apply path must accept validly signed peer commits from older daemons to preserve replay and live convergence; the admin/member-only vocabulary is enforced at authoring/assignment, not receipt. `Owner` remains rejected on apply in this PR and should receive separate replay/convergence assessment if changed later because it is admin-equivalent.

## Pre-existing ban tombstone behaviour

Banning an agent who is not a member inserts a `Guest`/`Banned` placeholder; unbanning reactivates an active `Guest` member-level entry. This was pre-existing upstream behavior in `GroupInfo::ban_member` (authored 2026-04, commit `ba965266`, present unchanged at base `189b89c`), not introduced by ADR-0016 Phase 1 and not changed in this PR.

Security framing: every path to trigger this requires admin authority; the placeholder is member-level, not admin, has no KEM key, and never received the group secret. Under ADR-0016, Admin is root for the group, so this admin-only, keyless artifact does not add capability beyond what an admin can already do. It is flagged as a separate maintainer item if David wants to make banning a non-member a no-op.

## Creator provenance

Creator provenance is best-effort historical, derived from the base-state snapshot; it is not authority-bearing and is not a tamper-evident guarantee.

## Discovery receive-path scope clarification

The known-local-group `GroupCardPublished` metadata-apply path now enforces that the sender is an active Admin before updating the local discovery card cache.

The global discovery listener, directory shard listener, and ListedToContacts direct-card listener are a different surface: best-effort signed-hint / key-possession discovery caches. They verify the card signature (and privacy placement) but do **not** check the card signer against the receiver's current group roster.

For known local groups, these pre-existing David C.2/D.3 discovery receive paths can cache or override a signed discovery listing without confirming the signer is currently an active Admin. This is cosmetic discovery cache state only, not committed group state, and is flagged to David as a pre-existing observation. Slice 4 intentionally does not harden those discovery receive paths.

## Slice 4 daemon coverage scope

The real three-daemon non-creator-admin invite proof now has both `public_open` and `private_secure` variants in `tests/named_group_join_metadata_event.rs`. The `public_open` variant covers non-TreeKEM convergence; the `private_secure` variant covers the TreeKEM secure-plane end-to-end join shape, Welcome/security-binding convergence, and creator-vs-inviter split. Direct expected-inviter sender/actor rejection is covered by the focused `join_result_requires_stored_expected_inviter` unit regression rather than claimed from the daemon e2e alone. Local execution attempts on macOS still hit the known daemon-startup timeout before assertions; keep the CI/startup-timeout carve-out caveat visible in readiness handoff.
