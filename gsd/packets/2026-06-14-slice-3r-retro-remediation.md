# GSD Work Packet

Date: 2026-06-14
Requested agent/tool: OpenCode
Role requested: Implementer
Project: x0x
Repo/path: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
Planning path: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`

## Goal

Implement focused remediation for Slice 1-3 retro review blockers before Slice 4 starts.

This packet closes the blocking retro findings from `gsd/checkpoints/2026-06-14-slice-1-3-retro-review.md`:

1. non-creator last-admin `DELETE /groups/:id` can mutate live state before `seal_commit` rejects;
2. reserved roles can still be applied through signed `MemberRoleUpdated` gossip apply.

It also applies the retro honesty fixes to the Slice 2 and Slice 3 checkpoint wording before this work rides into the PR.

## Read first

- `gsd/plan/phase-1-plan.md`
- `gsd/plan/phase-1-plan-addendum-2-slice-3r-remediation.md`
- `gsd/checkpoints/2026-06-14-slice-1-3-retro-review.md`
- `gsd/spec/phase-1-authority-alignment.md`
- `gsd/checkpoints/2026-06-12-slice-1-last-admin-invariant.md`
- `gsd/checkpoints/2026-06-14-slice-2-owner-retirement.md`
- `gsd/checkpoints/2026-06-14-slice-3-membership-authority.md`
- `src/bin/x0xd.rs`
- `src/groups/mod.rs`
- `src/groups/member.rs`
- `src/groups/state_commit.rs`
- `tests/last_admin_invariant.rs`
- `tests/owner_retirement.rs`
- `tests/membership_authority.rs`
- relevant ignored daemon named-group tests for maintainer-gate patterns

## Current source of truth

- Feature branch: `feat/adr-0016-phase-1-authority-alignment`
- Current feature head: `6ebac93`
- PR #5 CI green of record: <https://github.com/JimCollinson/x0x/pull/5>
- Retro checkpoint status: blocked before Slice 4.

## Approved remediation

### R2 stop-condition decision update — Jim, 2026-06-14

The pre-code audit found shipped pre-ADR code could author `MemberRoleUpdated` commits assigning `Moderator` / `Guest`. Jim selected Option 1: **accept-and-document; do not reject `Moderator` / `Guest` on signed/gossip apply in Slice 3R.**

Rationale to record in the remediation checkpoint and PR note:

- `Moderator` (rank 2) and `Guest` (rank 0) grant no **admin** authority — the admin authority threshold is `at_least(Admin)`; both sit below it. (`Owner`, the one role that does grant admin authority, is admin-equivalent and already handled.)
- An active member of any role remains member-level; that is expected for legitimate legacy replay. Do not claim reserved roles are globally inert.
- Authority comes from the signed commit + the `at_least(Admin)` check + the last-admin invariant — not from policing target-role vocabulary at apply time.
- In a gossip network the apply path must accept any validly-signed peer commit, including from daemons still on the old version. Rejecting on apply would (a) break byte-for-byte legacy replay and (b) fork live state between upgraded and un-upgraded daemons. The admin/member-only **assignment** rule belongs at **authoring**, not apply.
- Current code must not fabricate an active reserved-role member from a non-member through ban/unban.

Updated R2 scope: confirm the REST role-assignment handler is the only `MemberRoleUpdated` authoring path or gate any other authoring path the same way; add tests for authoring-block, apply-accept, and last-admin invariant behavior. Leave existing `Owner`-on-apply rejection unchanged and carry a PR/gauntlet note that its replay/convergence implications need separate assessment.

### Active-Guest adversarial blocker decision — Jim, 2026-06-14

Adversarial confirmation found that current daemon code can fabricate an active `Guest` by banning an absent target (creating a banned `Guest` tombstone) and then unbanning it (reactivating that tombstone).

Jim selected the smallest fix:

- make ban/unban unable to turn a never-member tombstone into an active member;
- acceptable implementation: unbanning a never-was-a-member tombstone does not activate it, or ban does not create an activatable tombstone for an absent target;
- do not globally make `Guest` inert;
- do not reject legacy `Moderator` / `Guest` signed apply;
- preserve the corrected wording: reserved roles grant no admin authority; a legitimately active member of any role remains member-level; ban/unban can no longer fabricate an active member from a non-member.

Add normal-gate tests for ban-absent → unban and for legitimate legacy active `Guest` replay/member-level behavior as needed.

### R1 — Fix non-creator last-admin self-leave corruption

At current head, after Slice 3:

- an admin can remove the creator when another admin remains;
- a non-creator sole admin can then call `DELETE /groups/:id`;
- non-creator self-leave mutates live `GroupInfo` before `seal_commit`;
- if `seal_commit` rejects, in-memory state can be left as live zero-admin.

Fix this without implementing the full Slice 5 leave/disband split.

Required properties:

- A rejected last-admin self-leave must not mutate live group state.
- Rejection should be a friendly 409, not a post-mutation 500.
- If adding the self-leave-specific §3 string early, use exactly:
  - `a group must always have at least one admin; make another member an admin before leaving`
- Preserve current creator-delete behavior for Slice 5.
- Preserve TreeKEM/self-leave semantics unless the audit shows the same mutate-before-seal bug there; if so, fix only the same corruption pattern.

### R2 — Fix reserved-role signed apply gap

At current head:

- REST role assignment accepts exactly `admin` and `member`.
- Gossip `MemberRoleUpdated` apply rejects `Owner` but still allows `Moderator` / `Guest`.

Fix so signed apply cannot assign reserved roles, subject to the historical replay principle.

Required properties:

- Before changing apply behavior, actively confirm whether legacy `Moderator` / `Guest` role-update commits could exist under pre-ADR-0016 shipped code.
- Principle: reject new reserved-role assignments; keep historical replay valid.
- If historical `Moderator` / `Guest` role-update commits could have been produced by shipped code, stop and surface instead of breaking replay.
- If they were never practically assignable, record the evidence in the checkpoint and reject them on the apply path.
- `MemberRoleUpdated` apply rejects `Owner`, `Moderator`, and `Guest` when safe. **Superseded for Moderator/Guest by Jim's stop-condition decision above: do not reject Moderator/Guest on apply in Slice 3R.**
- `MemberRoleUpdated` apply still accepts `Admin` and `Member`.
- Existing stored legacy `Owner` entries remain parseable and admin-equivalent.
- Owner-to-admin normalization remains valid.
- Do not change serde names, role bytes, roster hash inputs, signing, commit format, or storage format.

### R3 — Correct retro-identified evidence overclaims

- Slice 2 checkpoint: replace “byte-for-byte legacy Owner chain replay” wording with a narrower claim: fixed legacy Owner roster serialization/root stability plus current-code replay over Owner-containing rosters. Do not claim a genuine pre-Slice-2 historical chain fixture unless one is added.
- Slice 3 checkpoint: replace “REST coverage” wording where it implies actual daemon-handler coverage with “REST-semantics/helper coverage”; true daemon HTTP coverage remains maintainer-gate / ignored-suite exposure unless explicitly added.

## Tests This Remediation Adds Or Updates

Fast-gate / normal nextest coverage:

- Non-creator last-admin self-leave rejection does not mutate the original group state.
- If a helper exposes the REST error, exact “before leaving” 409 string is asserted in normal tests.
- Signed/gossip-style `MemberRoleUpdated` accepts `Admin`, `Member`, and legacy `Moderator` / `Guest` per Jim's replay decision.
- Last-admin invariant still rejects sole-admin demotion to below-Admin roles, including `Moderator` / `Guest`.
- Owner-to-admin normalization remains valid.
- Evidence test or checkpoint note confirms whether legacy `Moderator` / `Guest` role-update commits were ever producible by shipped assignment paths.
- Existing Slice 1-3 targeted tests still pass.

Maintainer-gate / ignored daemon coverage, if feasible without broad harness work:

- Create/promote/remove-creator sequence:
  - creator A creates group;
  - A promotes B;
  - B removes A;
  - B attempts `DELETE /groups/:id`;
  - response is 409 and B remains active admin in roster.
- Mark clearly in checkpoint which assertions are fast-gate vs maintainer-gate.

Do not create a flaky multi-daemon mesh test for a deterministic state-level property.

## Out of scope

- Slice 4 invite creator gate.
- Slice 5 full leave/end-group endpoint split.
- Full `DELETE /groups/:id` semantic change for creators.
- CLI `disband` / `state-withdraw` alias work.
- KeyPackage distribution improvements, Phase 2.
- Rekey/committer behavior, Phase 3.
- Broad actor/sender/committer normalization across all gossip arms.
- GUI/docs surface sweep.
- GSD gate, CI workflow, test harness, daemon wrappers, build invocation, environment setup.
- PR creation.

## Verification required

Run in exact mandatory Rust order after code changes:

- `cargo fmt --all`
- `cargo clippy --all-features --all-targets -- -D warnings`
- `cargo check --workspace --all-targets`

Then targeted tests:

- `cargo nextest run --all-features -E 'test(last_admin) or test(role) or test(member)'`
- `cargo nextest run --all-features --test membership_authority`
- any new focused test filter, recorded exactly

Before readiness:

- Push only to Jim’s fork.
- Confirm PR #5 CI green of record.
- Update remediation checkpoint: `gsd/checkpoints/2026-06-14-slice-3r-retro-remediation.md`
- Run adversarial and craft confirmation on the remediation delta, or explicitly record Jim’s waiver.

## Environment / CI heads-up

- Preserve the repo’s `time` 0.3.47 pin.
- Known macOS mesh/event-propagation flakes may appear locally or in CI. If CI reds on that class, rerun without code/harness/env changes and record failed attempt plus rerun result.
- Do not “fix” by changing harness, wrapper, daemon invocation, build invocation, CI workflow, `.gsd/gate.sh`, or environment.
- No wrappers/shims for readiness evidence.
- CI PR #5 is the green of record; local checks are supporting evidence only.

## Stop conditions

Stop and report if:

- fixing self-leave corruption requires implementing the full Slice 5 leave/disband model;
- preserving current creator-delete behavior becomes incompatible with the safe self-leave fix;
- TreeKEM leave semantics require broader rekey/committer work;
- legacy `Moderator` / `Guest` role-update commits could exist and the proposed apply rejection would break historical replay;
- reserved-role apply rejection appears to break required legacy chain replay;
- any fix requires changing serde names, role bytes, hashing, signing, commit format, invite wire format, or storage format;
- work appears to require changing `.gsd/gate.sh`, CI workflow, test harness, daemon wrapper, build invocation, or environment setup;
- scope expands into Slice 4, Slice 5 full semantics, Phase 2, Phase 3, GUI/docs, or PR creation.

## Required output

Return:

- files changed;
- commits created;
- how each HIGH/CONFORMANCE retro finding was fixed or dispositioned;
- evidence on legacy `Moderator` / `Guest` role-update assignability;
- fast-gate vs maintainer-gate test coverage;
- local verification commands and results;
- PR #5 CI status;
- adversarial/craft confirmation results;
- checkpoint update status;
- blockers or risks.
