# GSD Work Packet

Date: 2026-06-14
Requested agent/tool: OpenCode
Role requested: Implementer
Project: x0x
Repo/path: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
Planning path: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`

## Goal

Implement ADR-0016 Phase 1 Slice 4: invites per issuer and creator provenance.

Any Admin may issue invites. Invite issue/consume/track runs on the issuing daemon. A joiner's recorded `GroupInfo.creator` must come from genesis/base state history, never unsigned invite metadata. Inviter remains the routing target for join-result polling.

## Read first

- `gsd/plan/phase-1-plan.md` — Universal slice preamble and Slice 4 section.
- `gsd/spec/phase-1-authority-alignment.md` — R7/R8 and acceptance criteria.
- `gsd/checkpoints/2026-06-12-slice-1-last-admin-invariant.md`
- `gsd/checkpoints/2026-06-14-slice-2-owner-retirement.md`
- `gsd/checkpoints/2026-06-14-slice-3-membership-authority.md`
- `src/bin/x0xd.rs`
- `tests/membership_authority.rs`
- relevant named-group invite/join integration tests

## Current source of truth

- Feature branch: `feat/adr-0016-phase-1-authority-alignment`
- Current feature head: `6ebac93`
- PR #5 CI green of record: <https://github.com/JimCollinson/x0x/pull/5>
- Slice 3 checkpoint: complete and green.

The plan's pinned `x0xd.rs` line numbers were taken at `189b89c` and predate Slices 1-3. Re-confirm the invite gate and provenance/routing sites at current head before editing; do not rely on old line numbers.

## Prework

- Follow the plan's universal slice preamble.
- Sync fork/main as required by the plan.
- Rebase only if required by the plan/current branch state.
- Inspect current invite issue/consume/join-result paths before patching.
- Record current-head grep evidence for remaining `creator` comparisons in authority paths.
- Treat remaining leave/delete creator semantics as Slice 5 out of scope.

## Approved slice

Slice 4: Invites per-issuer + creator provenance (R7, R8).

## Scope

- Delete the invite creator gate in `x0xd.rs` around current-head `10596`; role lookup is the survivor.
- Any active Admin-or-higher may issue invites.
- Preserve the existing per-issuer issue/consume/track mechanic; only remove creator-locking.
- Correct joiner-side `GroupInfo.creator` provenance so it derives from seeded base state / genesis, not unsigned `invite.inviter`.
- Keep inviter identity as the join-result polling/routing target.
- Keep creator/provenance and inviter/routing variables distinct in code and tests.
- Re-confirm the existing consume-side `MemberJoined` path already checks that the inviter is Admin-or-higher. Do not duplicate or contradict that check when removing the issue-side creator gate.
- Keep gate-runnable assertions gate-runnable. Do not move provenance or issue-side authorization checks behind ignored daemon tests if they can be tested at the group-state/helper seam.
- Run and record the closing no-creator-authority sweep for Slices 2-4.
- Classify remaining `creator` comparisons as provenance/routing, Slice 5 scope, or drift.

## Tests This Slice Adds Or Updates

Fast-gate / normal nextest coverage:

- Promoted non-creator Admin passes invite-issue authorization.
- Plain `member` cannot issue an invite.
- Invite issue-side creator gate is gone; role lookup is the authority.
- When inviter != creator, provenance derivation keeps `GroupInfo.creator` equal to genesis/base-state creator, not unsigned `invite.inviter`.
- Creator-issued invite path still passes the same fast authorization/provenance checks.
- Existing consume-side `MemberJoined` inviter-admin check is verified and not duplicated or contradicted.

Maintainer-gate / real daemon coverage:

- Promoted non-creator Admin issues an invite through the daemon.
- Joiner consumes that invite against the issuing daemon and joins.
- Join-result polling still routes to inviter when inviter != creator.
- Creator-issued invites still work end-to-end.

Checkpoint must state plainly which assertions were covered by fast-gate tests and which remain maintainer-gate daemon/mesh assertions.

## Out of scope

- Invite wire-format changes.
- KeyPackage distribution improvements, Phase 2.
- Rekey/committer behavior, Phase 3.
- Leave/end group endpoint split, Slice 5.
- GUI/CLI invite surfaces, Slice 7.
- Serde names, role bytes, hashing, signing, commit format, storage format.
- GSD gate, CI workflow, test harness, daemon wrappers, build invocation, environment setup.
- PR creation.

## Verification required

Run in exact mandatory Rust order after code changes:

- `cargo fmt --all`
- `cargo clippy --all-features --all-targets -- -D warnings`
- `cargo check --workspace --all-targets`

Then targeted tests:

- `cargo nextest run --all-features -E 'test(invite)'`
- Adjust targeted filter if needed and record exact command.

Before readiness:

- Push only to Jim's fork.
- Confirm PR #5 CI green of record.
- Update Slice 4 checkpoint in planning branch.

## Environment / CI heads-up

- Preserve the repo's `time` 0.3.47 pin.
- Known macOS mesh/event-propagation flakes may appear locally or in CI. If CI reds on that class, rerun without code/harness/env changes and record the failed attempt plus rerun result.
- Do not change harness, wrapper, daemon invocation, build invocation, or environment.
- No wrappers/shims for readiness evidence.
- CI PR #5 is the green of record; local checks are supporting evidence only.

## Stop conditions

Stop and report if:

- invite flow proves structurally keyed to the creating daemon beyond the deleted gate, such that per-issuer operation needs redesign rather than gate removal;
- fixing provenance would require changing what the invite carries on the wire;
- proving provenance correctness appears to require a flaky two-daemon mesh test rather than a deterministic helper/state-level test;
- removing the issue-side gate reveals the consume-side inviter-admin check is absent or semantically different from the plan's assumption;
- any remaining creator comparison appears to be authority rather than provenance/routing or Slice 5 scope;
- work appears to require changing `.gsd/gate.sh`, CI workflow, test harness, daemon wrapper, build invocation, or environment setup;
- production serialization, role bytes, hashing, signing, commit format, invite wire format, or storage format needs changing;
- scope expands into leave/disband, KeyPackage distribution, Phase 2/3 behavior, GUI/CLI invite surfaces, or PR creation.

## Required output

Return:

- files changed;
- commits created;
- current-head grep evidence for creator comparisons and dispositions;
- local verification commands and results;
- PR #5 CI status;
- checkpoint update status;
- blockers or risks.
