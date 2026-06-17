# Spec: ADR-0016 Phase 1 — authority alignment

- Status: **Approved by Jim, 2026-06-12.** One open cosmetic item: the user-facing verb for the group-ending act ("disband" proposed, "delete" fallback) is pending the maintainer's preference, asked on issue #107 — a one-line swap either way, non-blocking for planning and implementation.
- Date: 2026-06-12
- Contract: upstream `docs/adr/0016-role-based-group-authority-flat-admin.md` (Accepted, `189b89c`). This spec implements its **Phase 1 only** and must never contradict it; on any conflict, the ADR wins and the conflict is reported.
- Context: upstream issue #107 (maintainer's triage and phasing comments).
- Code sites cited below were **byte-verified with eyes on the code** on upstream `main` @ `189b89c` (2026-06-12): `member.rs` and `state_commit.rs` read in full; every cited `x0xd.rs` site independently extracted and quoted (zero drift); `docs/api-reference.md` read in full. No ADR-0016 implementation exists upstream at this commit. **Line numbers are valid only at that commit** — re-verify against freshly synced `main` before relying on them.

## 1. Purpose and boundaries

Make runtime behaviour match the signed state chain's authority model: administrative acts on a group are authorised by the actor's **role on the committed roster** — never by creator identity. Ships as **one PR** from `feat/adr-0016-phase-1-authority-alignment`.

**In this PR:** everything in §2.
**Deferred, stated openly in the PR description:** delegated ban remains non-operational (target KeyPackage distribution is Phase 2); deterministic-committer and race mitigations are Phase 3; the two-admin metadata race window therefore exists in the interim — exposed, not solved, per the ADR.
**Deliberately kept:** stored `owner` roster entries (readable indefinitely, never rewritten); `Moderator`/`Guest` enum variants (reserved, non-assignable — serde names and `role_byte` feed the roster hash); `GroupMember::new_owner` constructor may remain for legacy-roster tests; `GroupGenesis.creator_agent_id` (history, not authority); audit fields `added_by`/`removed_by` (provenance).

## 2. Behavioural requirements

### R1 — Role-based authority everywhere
Delete and replace with the existing role lookup (`require_admin_or_above` def x0xd.rs:12026 / roster lookup in `validate_apply`):
- the five creator gates: invite 10625–10628; add-member 11043–44, 11170–74; remove-member 11350–51, 11578–83 (all x0xd.rs);
- `require_owner` (def 12037, sole use 12131 in `update_group_policy`);
- the `OwnerOnly` action class (`state_commit.rs` 474/489/571) and its three daemon apply sites (8587 GroupDeleted, 8633 PolicyUpdated, 8695 role-change) — all become `AdminOrHigher`;
- the two-tier role matrix, both enforcement sites (gossip-apply 8681–8691; REST `update_member_role` matrix 12261–12267);
- the ownership-transfer 400 stub (12223–12229);
- the **receive-path creator gate** in the `PolicyUpdated` gossip arm (`creator_auth` at ~8625: `sender_hex == creator_hex && actor == sender_hex`) — the gossip path has its own literal creator check beyond the `OwnerOnly` action kind;
- in-code doc comments describing owner-only semantics (e.g. `state_commit.rs` module header: "Owner for policy changes"; `ActionKind::OwnerOnly` doc) updated to the flat model.
Every former owner-only act (end group, update policy, change an admin's role) becomes an ordinary admin act. Note: authority is currently enforced through **two coexisting mechanisms** — literal `!= info.creator` string checks AND the role-based layer (`caller_role`/`at_least`, `ActionKind`, `require_*` helpers). Phase 1 deletes the former entirely; the latter is the survivor. `require_admin_or_above` has 8 existing call sites (12064, 12331, 12483, 12626, 12683, 12837, 13009, 13166) that already model the target pattern.

### R2 — Last-admin invariant
New check at the `validate_apply` choke-point (`state_commit.rs:521`; authority is the final step): reject any commit whose **post-mutation, non-withdrawn** state contains zero active members of rank ≥ Admin. Legacy `Owner` counts as Admin (automatic if implemented via `role.at_least(Admin)` — Owner rank 4 > Admin rank 3). Group ending (withdrawn state) is exempt — it is the last admin's exit valve. Enforced on **every delivery path** (REST and gossip-apply); REST handlers additionally return friendly pre-check errors (§3).

**Implementation subtlety (verified in code):** the commit object carries only `roster_root` — a *hash* of the post-mutation roster, not the roster itself — and `ApplyContext.members_v2` is the *parent* state. The invariant must therefore be evaluated over the **proposed post-mutation roster computed by the applier**, fed to the check at the same choke-point on every path. Whether that's a new argument to `validate_apply` or an adjacent mandatory check is implementer latitude; the binding constraints are: same choke-point semantics, all delivery paths, evaluated post-mutation. `ApplyError::Invariant` ("invariant violation") already exists and is the natural error for rejections.

Consequences: the last admin cannot self-demote or self-leave; a sole legacy Owner self-normalising to Admin passes (admin count stays ≥ 1).

### R3 — Owner-target special cases deleted
"cannot remove creator" (11348, 11574) and "cannot ban owner" (12339, 12489) are deleted; R2 subsumes the legitimate protection they provided.

### R4 — Legacy alias, no new assignments
Stored roster entries are never rewritten (hash preservation). `GroupRole::Owner` remains in the enum (serde `"owner"`, `as_u8` 0) and evaluates as Admin-equivalent at validation time. No API path may assign it. An ordinary `MemberRoleUpdated` self-demotion (owner → admin) is a valid normalization commit; never required.

### R5 — Role-assignment API accepts exactly `admin` and `member`
`owner`, `moderator`, `guest` are rejected with the exact errors in §3. `GroupRole::from_name` (`member.rs` 6–71) keeps parsing all stored names (deserialization of history must not break); restriction applies to the assignment path, not parsing.

### R6 — Genesis seeds an ordinary first Admin
`new_owner` call sites (`groups/mod.rs` 266, 570; `x0xd.rs` 678, 13590, 13631) seed the creator as first **Admin**. New groups never contain an `owner` entry. (Test helper `state_commit.rs:597` may keep the constructor for legacy-roster tests.)

### R7 — Invites per-issuer, by any Admin
Any Admin may issue invites; the issue/consume/track flow runs on the issuing daemon (the existing mechanic, no longer creator-locked).

### R8 — Creator provenance correction
A joiner's `GroupInfo.creator` is no longer seeded from unsigned `invite.inviter` metadata; it derives from the seeded base state / genesis. No authority decision may consult it (it is history). Inviter identity remains the routing target for join-result polling — the two identities are distinct variables.

**Slice 4 scope clarification (2026-06-17):** the known-local-group metadata-apply path enforces active Admin authority before accepting `GroupCardPublished` into the local card cache. The global discovery listener, directory shard listener, and ListedToContacts direct-card listener are best-effort discovery caches, not current group-admin authority checks. Shard/contact delivery require signature/key possession; global discovery retains a legacy unsigned-card compatibility path and verifies signatures when present. For known local groups, those pre-existing David C.2/D.3 receive paths can cache or override a discovery listing without confirming the signer is currently an active Admin. That is cosmetic discovery cache state, not committed group state, and is flagged to David rather than fixed in Slice 4. Separately, user-initiated `POST /groups/cards/import` is pre-existing and out of Slice 4 scope; it can refresh known local `GroupInfo` from an explicitly imported signed card without current-roster Admin validation, so it should be treated as a separate maintainer decision if David wants known-group imports hardened.

### R9 — Surfaces audit (final slice, completes before PR)
Sweep GUI surface, `src/cli/`, `src/api/`, `src/bin/gui_coverage.rs`, and `docs/api-reference.md` so that **no surface requires an owner to exist** and legacy `owner` renders readably wherever roles display. Known findings in `docs/api-reference.md` (full read, 2026-06-12): the AdminOnly write-access description ("only `Admin` or `Owner` may send"); `POST /groups/:id/state/seal` documented as "(owner/admin)"; `POST /groups/:id/state/withdraw` documented as "(owner)"; the add-member and remove-member rows described as "Creator-authored". Fix findings in this PR. **Stop-rule:** if a finding is unexpectedly large, stop and surface to Jim before expanding scope.

### R10 — Documentation bluntness and role help
Repo docs state plainly: **Admin is root for the group** — a hostile or compromised admin can admit, remove, rekey, change policy, and end the group; keep the admin set small; do not map softer application roles onto x0x Admin. Role-assignment docs reflect the admin/member-only contract. Additionally (verified gap: the CLI is a thin pass-through with no role help today): the `x0x group set-role` CLI help text lists the valid roles (`admin`, `member`) with a one-line meaning for each, and `docs/api-reference.md` gains a short plain-language "Roles" explainer (admin = full control including ending the group; member = participant; `owner` = legacy alias rendered for old groups, equivalent to admin, not assignable) near the role-assignment endpoint. This is where the error messages' "valid roles" pointer finds its depth — errors stay terse, the docs carry the explanation.

## 3. Error-message contract (exact strings — implementer may not improvise)

| Condition | HTTP | Body |
|---|---|---|
| Assign role `owner` | 400 | `{"error":"'owner' is a legacy role and cannot be assigned; valid roles: 'admin', 'member'"}` |
| Assign role `moderator` or `guest` | 400 | `{"error":"role '<name>' is reserved and cannot be assigned; valid roles: 'admin', 'member'"}` |
| Demote/remove/ban would leave zero admins | 409 | `{"error":"a group must always have at least one admin; make another member an admin first"}` |
| Last admin attempts self-leave | 409 | `{"error":"a group must always have at least one admin; make another member an admin before leaving"}` |

Style rules (Jim, 2026-06-12): short, instructive, name the valid options; do **not** suggest ending the group as a remedy in last-admin errors; no ADR/roadmap references; no URLs (matches house style — existing daemon errors are terse one-liners, with at most a short instructive suffix). The existing 424 KeyPackage error on delegated ban is **unchanged** (explained in the PR description instead). Exact status codes to be confirmed against repo conventions during slice work — the strings are fixed; codes follow repo precedent (record deviations in the slice checkpoint).

### Resolved design point — exactly two user-facing actions (decided by Jim, 2026-06-12)

Background: `DELETE /groups/:id` is documented as "**Leave or delete** the group" — one endpoint whose meaning today depends on caller identity (creator ⇒ delete the group; anyone else ⇒ leave). Post-Phase-1 a mechanical creator→admin conversion of that switch would let an admin who merely wants to leave silently destroy the group. Decision:

1. **Leave group** — `DELETE /groups/:id` means *leave* for **everyone, regardless of rank**: a pure self-act. Blocked only when the leaver is the last active admin (R2; legacy Owner counts), with the §3 message.
2. **End the group** — a separate, explicit, deliberate action; any admin may perform it at any time, including as last admin (exempt from the invariant per the ADR). Mechanically this is the existing terminal-withdrawal commit; the endpoint path (`POST /groups/:id/state/withdraw`) is kept for wire/API compatibility. **The user-facing verb is pending the maintainer's answer on #107**: "disband" proposed (accurate — the group ends as a unit; its signed history remains; cannot be misread as a member-level act), "delete" the fallback (conventional). Whichever wins: one primary CLI command (`x0x group disband <id>` or `x0x group delete <id>`), `state-withdraw` retained as a quiet deprecated alias, api-reference describing the endpoint as "<Verb> the group for everyone (permanent; propagates to all members)". A one-line swap; does not block any slice.
3. **"Withdraw/withdrawal" becomes internal vocabulary only** (the chain's terminal `withdrawn` record — wire-frozen, cannot be renamed without breaking historical verification). The R9 audit includes a language sweep: no user-facing surface (CLI help, GUI, api-reference) may describe the group-ending act as "withdrawing" without the chosen verb alongside.

Verify current `DELETE /groups/:id` handler semantics at slice time before implementing.

## 4. Out of scope

KeyPackage distribution in `MemberAdded` (Phase 2 — wire sketch on #107 required first). Deterministic committer, rebase-and-retry, sibling diagnostics (Phase 3). Equal-revision fork-choice (future ADR). Any change to hashing, signing, commit format, or the validation pipeline beyond the authority step + R2. Any rewrite of stored roster history. New roles or mandate machinery.

## 5. Acceptance criteria (all testable in this PR)

1. A promoted Admin can invite, add, change roles, change policy, and end the group, with commits that validate and converge to all members including the actor.
2. **Property test:** across generated commit sequences on both delivery paths (REST and gossip-apply), no non-withdrawn state with zero active admins is ever reachable; legacy `Owner` counts as admin.
3. Historical chains containing `Owner` entries verify byte-for-byte; a legacy Owner administers unchanged and can self-normalize to Admin with one ordinary role commit.
4. Role API rejects `owner`/`moderator`/`guest` with §3's exact errors; rosters render legacy `owner` readably.
5. Both-path coverage is explicit: every authority behaviour exercised via REST **and** via gossip-apply (the choke-point check, not just the pre-checks).
6. Quality gates: no production `unwrap`/`expect`/`panic`; `cargo fmt --check`, `clippy -D warnings`, nextest all green; the `#[ignore]`d daemon-API suite and multi-daemon convergence tests green on the maintainer-side local machine (final gate).

**Explicitly not claimed by this PR:** the full #107 repro with the non-inviting admin *banning* (passes only after Phase 2); self-leave PCS rekey criterion (Phase 3).

## 6. Process bindings

One PR; branch `feat/adr-0016-phase-1-authority-alignment` cut from freshly-synced `main`; rebase before review and again before PR. Conventional Commits. Gauntlet (independent clean-context test + adversarial review) before PR. **PR creation requires Jim's explicit confirmation.** Slices are internal working order — the invariant (R2) lands before or with the deletions (R1/R3) in the working sequence.
