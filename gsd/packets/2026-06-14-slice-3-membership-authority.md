# GSD Work Packet

Date: 2026-06-14
Requested agent/tool: OpenCode
Role requested: Implementer
Project: x0x
Repo/path: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
Planning path: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`

## Goal

Implement ADR-0016 Phase 1 Slice 3: membership authority for add/remove/ban.

Add/remove/ban must be authorized by committed-roster role, never creator identity. Owner-target special cases vanish; Slice 1 last-admin invariant is the only protection for last admin / legacy owner cases.

## Read first

- `gsd/plan/phase-1-plan.md`
  - Universal slice preamble / per-slice verification rules.
  - Slice 3 section.
- `gsd/spec/phase-1-authority-alignment.md`
- `gsd/checkpoints/2026-06-12-slice-1-last-admin-invariant.md`
- `gsd/checkpoints/2026-06-14-slice-2-owner-retirement.md`
- `src/bin/x0xd.rs`
- `tests/last_admin_invariant.rs`
- `tests/owner_retirement.rs`
- relevant named-group integration tests

## Current source of truth

- Feature branch: `feat/adr-0016-phase-1-authority-alignment`
- Current feature head: `b9f6b37`
- PR #5 CI green of record: <https://github.com/JimCollinson/x0x/pull/5>
- Slice 2 checkpoint: complete and green.

The plan's pinned `x0xd.rs` line numbers were taken at `189b89c` and predate Slices 1 and 2. Re-confirm all add/remove/ban creator gates and owner-target guards at current head before editing; do not rely on old line numbers.

## Prework

- Follow the plan's universal slice preamble.
- Sync fork/main as required by the plan.
- Rebase only if required by the plan/current branch state, then re-run verification from the rebased/current head.
- Inspect current add/remove/ban sites before patching.
- Record current-head grep evidence for creator comparisons in add/remove/ban authority paths.

## Approved slice

Slice 3: Membership authority: add, remove, ban.

## Scope

- Delete creator gates in add-member and TreeKEM add-member paths.
- Delete creator gates in remove-member and TreeKEM remove-member paths.
- Delete `cannot remove creator` guards.
- Delete `cannot ban owner` guards.
- Ensure role lookup / `require_admin_or_above` is the surviving REST authority layer.
- Ensure signed state-commit apply validation is the surviving gossip-apply authority layer.
- Preserve Slice 1 last-admin pre-check ordering before mutation / TreeKEM side effects.
- Use clone-first where a path otherwise mutates then seals.
- Exercise add/remove/ban on both paths:
  - REST handler path.
  - gossip-apply / state-commit choke-point path.
- Keep exact 409 error assertions gate-runnable in normal tests, not parked behind ignored maintainer-gate tests.

## Tests This Slice Adds Or Updates

- Promoted Admin adds members through REST.
- Promoted Admin add converges through gossip-apply / signed commit validation.
- Promoted Admin removes members through REST.
- Promoted Admin remove converges through gossip-apply / signed commit validation.
- Promoted Admin can remove a legacy `Owner` who is not the last admin.
- Promoted Admin can ban a legacy `Owner` who is not the last admin.
- Removing/banning/demoting the last admin through these handlers returns exact Section 3 409:
  - `a group must always have at least one admin; make another member an admin first`
- TreeKEM ban error precedence is pinned:
  - last-admin ban returns the last-admin 409 before missing-KeyPackage 424.
  - separate non-last-admin delegated-ban case without target material returns existing 424, not creator-identity 403.
- Plain `member` cannot add/remove/ban by REST.
- Plain `member` cannot add/remove/ban by gossip-apply.

## Out of scope

- Invite creator gate, Slice 4.
- Leave/end group endpoint split, Slice 5.
- KeyPackage distribution improvements, Phase 2.
- Rekey/committer behavior, Phase 3.
- Full #107 repro with non-inviting admin banning.
- Serde names, role bytes, hashing, signing, commit format, storage format.
- GSD gate, CI workflow, test harness, daemon wrappers, build invocation, environment setup.
- PR creation.

## Verification required

Run in exact mandatory Rust order after code changes:

- `cargo fmt --all`
- `cargo clippy --all-features --all-targets -- -D warnings`
- `cargo check --workspace --all-targets`

Then targeted tests:

- `cargo nextest run --all-features -E 'test(member) and (test(add) or test(remove) or test(ban))'`
- Adjust targeted filter if needed and record exact command.

Before readiness:

- Push only to Jim's fork.
- Confirm PR #5 CI green of record.
- Update Slice 3 checkpoint in planning branch.

## Environment / CI heads-up

- Preserve the repo's `time` 0.3.47 pin.
- Known macOS mesh/event-propagation flakes may appear locally or in CI. If CI reds on that class, rerun without code/harness/env changes and record the failed attempt plus rerun result. Do not "fix" by changing harness, wrapper, daemon invocation, build invocation, or environment.
- No wrappers/shims for readiness evidence.
- CI PR #5 is the green of record; local checks are supporting evidence only.

## Stop conditions

Stop and report if:

- deleting a guard exposes a state the Slice 1 choke-point invariant does not catch;
- remove/ban paths consult creator identity at uncited sites beyond line drift;
- last-admin TreeKEM ban returns 424 before invariant 409;
- work appears to require changing `.gsd/gate.sh`, CI workflow, test harness, daemon wrapper, build invocation, or environment setup;
- production serialization, role bytes, hashing, signing, commit format, or storage format needs changing;
- the scope expands into invites, leave/disband, KeyPackage distribution, or Phase 2/3 behavior.

## Required output

Return:

- files changed;
- commits created;
- current-head grep evidence for creator comparisons;
- local verification commands and results;
- PR #5 CI status;
- checkpoint update status;
- blockers or risks.
