# GSD Work Packet — Slice 1: last-admin invariant (ADR-0016 Phase 1)

(Repo-filed copy. The dispatched version is prefixed with the GSD cloud bootstrap block; content otherwise identical.)

Date: 2026-06-12
Prepared by: Claude (Cowork planning session), approved by Jim Collinson
Requested agent/tool: Claude Code (cloud/web session) on `JimCollinson/x0x`
Role requested: **Implementer — exactly this one slice.** Do not start any other slice.
Review mode: N/A (the gauntlet runs after all slices, before the PR)

## Project / workspace

Project: x0x ADR-0016 Phase 1 — authority alignment (flat Admin/Member, retiring Owner). One PR, seven slices; this is **Slice 1 of 7**.
Repo: fork `JimCollinson/x0x` (origin). Upstream: `https://github.com/saorsa-labs/x0x` — **READ-ONLY: never push there, ever.**
Work branch: `feat/adr-0016-phase-1-authority-alignment` (exists; clean at upstream main `189b89c` as of packet time).
Planning branch (GSD home, never merged, never PR'd): `gsd/adr-0016-planning`.
Current source of truth, in order: (1) upstream `docs/adr/0016-role-based-group-authority-flat-admin.md` (Accepted); (2) `gsd/spec/phase-1-authority-alignment.md` on the planning branch; (3) `gsd/plan/phase-1-plan.md` on the planning branch.

## Goal

Implement the last-admin invariant (spec R2): no commit may leave a live (non-withdrawn) group with zero active admins, enforced at the `validate_apply` choke-point on every delivery path (REST and gossip-apply), with friendly REST pre-check errors. The net goes up before any later slice removes the old protections.

## Read first (planning-branch files via `git show gsd/adr-0016-planning:<path>` or a second worktree)

1. `gsd/README.md` (planning branch) — the binding rules of this repo's GSD structure.
2. `gsd/spec/phase-1-authority-alignment.md` (planning branch) — §R2 and §3 are your contract; read the whole spec for context.
3. `gsd/plan/phase-1-plan.md` (planning branch) — the **"Universal slice preamble"** and **"Slice 1"** sections are binding: in-scope, out-of-scope, required tests, verification commands, done-when, stop-if. This packet summarizes them; the plan governs on any difference.
4. `docs/adr/0016-role-based-group-authority-flat-admin.md` (in-tree) — the contract above the spec.
5. Repo root `CLAUDE.md` / `AGENTS.md` — house engineering rules.

## Stage

Implementation (Slice 1 of the approved plan).

## Approved slice

Slice 1 exactly as defined in `gsd/plan/phase-1-plan.md` (plan approved by Jim 2026-06-12). Key points:

- **Preamble first:** sync fork `main` with upstream; rebase the feature branch if upstream moved; **re-verify every cited code site** (line numbers are pinned to `189b89c`; record drift in your checkpoint; stop if a cited mechanism has materially changed).
- New check at the `validate_apply` choke-point (`src/groups/state_commit.rs`, fn at ~521): reject any commit whose **post-mutation, non-withdrawn** state has zero active members of rank ≥ Admin. Legacy `Owner` counts (use `role.at_least(Admin)`). Withdrawn state exempt. Use `ApplyError::Invariant`.
- **The post-mutation roster subtlety:** the commit carries only `roster_root` (a hash); `ApplyContext.members_v2` is the parent state. Evaluate the invariant over the proposed post-mutation roster computed by the applier, via **one shared helper used identically by both delivery paths**. New `validate_apply` argument vs adjacent mandatory check is your latitude; same choke-point semantics on all paths is not.
- REST friendly pre-checks on remove-member, ban, and `update_member_role` (demotion): 409 with EXACTLY `{"error":"a group must always have at least one admin; make another member an admin first"}`. (The "before leaving" variant belongs to Slice 5 — not yours.)
- Tests per the plan's Slice 1 list, named with a `last_admin` prefix: demote/remove/ban-last-admin rejected; withdrawal-from-sole-admin accepted; sole legacy Owner→admin normalization accepted; Owner→member rejected; legacy Owner counted in mixed rosters; gossip-path rejection of a crafted zero-admin commit (the choke-point itself, not just pre-checks); exact-string REST assertions; proposed-roster-hashes-to-`roster_root` test.

## Scope summary

In: everything above. Out (other slices — do NOT touch): deleting any creator gate or owner-target guard; genesis seeding; `DELETE /groups/:id` semantics; the property-test suite; user-facing docs. The invariant must be behavior-neutral for currently-reachable states — if an existing test trips it, investigate that test's roster construction before changing anything.

## Constraints / forbidden actions

- NEVER push to upstream. All pushes to origin (the fork) only.
- No PR creation — always Jim's explicit gate.
- Work only on `feat/adr-0016-phase-1-authority-alignment`. No GSD files on the feature branch.
- Conventional Commits. No production `unwrap`/`expect`/`panic` in new/touched code.
- §3 error strings verbatim; status codes follow repo precedent (record deviations, don't improvise strings).
- No changes to hashing, signing, commit format, or validation-pipeline ordering beyond inserting this one check.

## Verification required (capture outputs as evidence)

```
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features --workspace
cargo nextest run --all-features -E 'test(last_admin)'
```

The `#[ignore]`d daemon-API suite and multi-daemon convergence tests are Jim's local gate — do not run them here; note your exposure to them in the checkpoint.

## Stop conditions (from the plan, binding)

Stop and report — do not improvise — if:

- the check cannot be implemented without altering hashing, signing, the commit format, or pipeline ordering beyond inserting this one step;
- any delivery path cannot be given the post-mutation roster without restructuring beyond the choke-point;
- re-verification reveals a delivery path that applies commits without passing the choke-point (spec-level finding for Jim);
- a cited code site has materially changed upstream (not just line drift);
- anything requires judgment beyond this packet + the plan's Slice 1 section.

## Required output

1. Commit and push the slice to the feature branch (origin).
2. **Checkpoint:** commit a checkpoint note to `gsd/checkpoints/2026-06-12-slice-1-last-admin-invariant.md` on the planning branch (what changed, verification evidence with command outputs, drift found vs the pinned line numbers, deviations, risks, recommended next step).
3. Final report in-session: role performed; sources read; changes; exact test results; blockers/risks/open questions; whether the plan remains valid; recommended next checkpoint.
