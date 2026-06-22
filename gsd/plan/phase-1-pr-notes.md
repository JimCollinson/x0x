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

The global discovery listener, directory shard listener, and ListedToContacts direct-card listener are a different surface: best-effort signed-hint / key-possession discovery caches. Shard/contact delivery require a card signature; global discovery preserves a legacy unsigned-card compatibility path and verifies the card signature when one is present. None of these discovery receive paths check the card signer against the receiver's current group roster.

For known local groups, these pre-existing David C.2/D.3 discovery receive paths can cache or override a signed discovery listing without confirming the signer is currently an active Admin. This is cosmetic discovery cache state only, not committed group state, and is flagged to David as a pre-existing observation. Slice 4 intentionally does not harden those discovery receive paths.

The user-initiated `POST /groups/cards/import` path is also pre-existing and out of ADR-0016 Slice 4 scope. Unlike passive discovery receive, it may refresh an existing local `GroupInfo` from an explicitly imported signed card (including withdrawn cards) after key-possession signature verification, without checking the signer against the current roster. This is not a peer-receive authority path and is not changed here; flag it to David as a separate maintainer decision if he wants imports for known local groups to require current Admin authority or signed-state application.

## Slice 4 daemon coverage scope

The real three-daemon non-creator-admin invite proof now has both `public_open` and `private_secure` variants in `tests/named_group_join_metadata_event.rs`. The `public_open` variant covers non-TreeKEM convergence; the `private_secure` variant covers the TreeKEM secure-plane end-to-end join shape, Welcome/security-binding convergence, and creator-vs-inviter split. Direct expected-inviter sender/actor rejection is covered by the focused `join_result_requires_stored_expected_inviter` unit regression rather than claimed from the daemon e2e alone. Local execution attempts on macOS still hit the known daemon-startup timeout before assertions; keep the CI/startup-timeout carve-out caveat visible in readiness handoff.

## Pre-existing daemon-startup harness flake

PR #5's remaining red is the pre-existing harness daemon-startup flake: `x0xd pair-alice-… did not become healthy within 90s`. The Slice 4 change set does not touch startup or networking code; the relevant diffs are invite/provenance handling and tests. Flag to David: the likely harness hardening is to spawn test daemons with `--no-hard-coded-bootstrap` (the flag exists and is used by the multi-instance example) so local daemon tests do not try to use the hard-coded bootstrap path.

## Slice 5 — leave/disband split

`DELETE /groups/:id` is now pure self-leave for every rank. Creator identity is no longer a leave/delete authority switch: creators, admins, legacy `owner` entries, and plain members all leave by self-removal, with the last active admin blocked by the ADR-0016 §3 `before leaving` error. TreeKEM creator self-leave routes through the same TreeKEM leave path, including the friendly last-admin pre-check before TreeKEM seal work.

The explicit group-ending act is the existing signed terminal withdrawal: `POST /groups/:id/state/withdraw`, surfaced primarily as `x0x group disband <group_id>`. The old `x0x group state-withdraw <group_id>` spelling remains a hidden/deprecated CLI alias for compatibility. No wire, commit, storage, signing, hashing, role serialization, or withdrawal format changed.

`GroupDeleted` production emission is reparented from creator-`DELETE` to explicit disband. The event shape is unchanged: `NamedGroupMetadataEvent::GroupDeleted { group_id, revision, actor, commit }` carries the existing signed withdrawal commit. Creator `DELETE` emits only self-leave `MemberRemoved`; disband publishes `GroupDeleted` on the metadata topic, direct-delivers it to active members, refreshes the withdrawn-card path for public discovery supersession, and removes live local state. Receivers validate the terminal withdrawal commit under `AdminOrHigher` and drop live local group/TreeKEM/listener state.

The earlier hidden/private propagation blocker is resolved by the `GroupDeleted` reparent in `a141d83`. The durable-terminality direction was clarified on 2026-06-19: disband should align with David's withdraw semantics, not delete semantics. A withdrawn group is a keyless metadata shell: keep the withdrawn `GroupInfo` record as the stale-card reanimation guard while wiping all crypto material. ADR-0012's "leave nothing behind" targets key material (TreeKEM snapshots / forward secrecy), not the metadata record. Security rationale: no key material survives disband, and the retained terminal record prevents stale-card reanimation. Privacy rationale: the retained record is a local association trace, consistent with David's existing withdraw design; retention/GC policy remains a maintainer decision and is not silently added here.

Local implementation attempt `939ab8c fix(adr-0016-phase-1): retain withdrawn disband shell`, plus fix pass `a9907b0 fix(adr-0016-phase-1): harden withdrawn shell terminality`, are **not pushed and not accepted**. Local fmt/clippy/check and focused/parity tests pass, including explicit withdrawn-card live-vs-stub and alias/queue regressions. Independent code review still found blockers: in-flight TreeKEM encrypt/decrypt can recreate a `.snap` after terminality, and TreeKEM persistence journals can retain/replay snapshot material because terminal wipe removes snapshots but not `<group_id>.journal`. Remediate those before describing Slice 5 as ready.

Scope assumption for David/future slices: file transfer, KV-store, and task-list writes are not named-group-bound today, so they are outside Slice 5 withdrawal terminality. If a future slice binds those data planes to named groups, the withdrawal sweep must be revisited for those surfaces.

## Raw MLS members endpoints maintainer note

The raw `POST`/`DELETE /mls/groups/:id/members` endpoints are a separate legacy MLS surface, not named-group terminality. They operate on the separate `state.mls_groups` keyspace, which Slice 5 disband/withdraw wipes, and they cannot resurrect a named group, signed named-group metadata, or the retained withdrawn named-group shell. This is flagged for David as a separate maintainer decision if the raw MLS API should be deprecated or given its own terminality guard; it is not fixed in Slice 5.

## Slice 7 deferred surface backlog

Slice 5 deliberately did **not** perform the full R9 user-surface language sweep. These items are deferred, not forgotten, and must be handled in Slice 7 before PR-ready handoff:

- `src/gui/x0x-gui.html`: update owner-gated state controls and user-facing `Withdraw` language to any-admin disband language.
- `tests/gui_named_group_parity.rs`: update GUI expectations if labels/data hooks change.
- `docs/api.md`: update `DELETE /groups/:id`, `/state/withdraw`, `state-withdraw`, and creator-authored member rows.
- `docs/primers/groups.md`: update `owner` and `withdraw / hide` wording.
- `docs/api-reference.md`: finish adjacent stale rows outside Slice 5 scope, especially `Creator-authored member add/removal` and state-chain owner/admin wording.
- `README.md`, proof reports, and design notes: classify remaining `owner`, `creator-authored`, `withdraw`, `delete group`, and `Leave or delete` occurrences as intended legacy/internal vocabulary or stale user-facing text.
- Broader R9 grep before PR: search docs/GUI/API/CLI for `owner`, `creator-authored`, `withdraw`, `delete group`, `Leave or delete`, and `state-withdraw`; fix stale user-facing text or record intentional legacy/internal uses.
