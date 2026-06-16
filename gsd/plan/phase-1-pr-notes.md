# ADR-0016 Phase 1 PR notes

These are running notes for the eventual PR description / maintainer handoff. They are not a PR; PR creation remains Jim's explicit gate.

## Reserved-role apply / authoring split

REST role assignment is restricted to `admin`/`member` (`GroupRole::assignable_from_name`), but the signed `MemberRoleUpdated` apply path rejects only `Owner`; validly signed `Moderator`/`Guest` commits remain accepted.

Rationale for David: `Moderator`/`Guest` grant no admin authority because the authority threshold is `role.at_least(Admin)`. An active member of any role is member-level, which is expected and not an admin escalation. The apply path must accept validly signed peer commits from older daemons to preserve replay and live convergence; the admin/member-only vocabulary is enforced at authoring/assignment, not receipt. `Owner` remains rejected on apply in this PR and should receive separate replay/convergence assessment if changed later because it is admin-equivalent.

## Pre-existing ban tombstone behaviour

Banning an agent who is not a member inserts a `Guest`/`Banned` placeholder; unbanning reactivates an active `Guest` member-level entry. This was pre-existing upstream behavior in `GroupInfo::ban_member` (authored 2026-04, commit `ba965266`, present unchanged at base `189b89c`), not introduced by ADR-0016 Phase 1 and not changed in this PR.

Security framing: every path to trigger this requires admin authority; the placeholder is member-level, not admin, has no KEM key, and never received the group secret. Under ADR-0016, Admin is root for the group, so this admin-only, keyless artifact does not add capability beyond what an admin can already do. It is flagged as a separate maintainer item if David wants to make banning a non-member a no-op.
