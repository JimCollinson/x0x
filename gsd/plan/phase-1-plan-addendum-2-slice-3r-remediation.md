# Plan addendum 2 — Slice 3R retro blocker remediation

- Date: 2026-06-14
- Status: Approved by Jim; binding before Slice 4. Amends `phase-1-plan.md` execution order after Slice 3.
- Origin: Slice 1-3 retro adversarial + craft review (`gsd/checkpoints/2026-06-14-slice-1-3-retro-review.md`) found blockers in the Slice 1-3 foundation.

## Objective

Close the blocking Slice 1-3 retro review findings before Slice 4 starts. This is not Slice 4. It is a focused remediation slice over the foundation Slices 1-3.

## Blocking findings to fix

1. **Last-admin self-leave corruption path.** After Slice 3, a non-creator admin can become the sole admin. The current non-creator `DELETE /groups/:id` self-leave path mutates live `GroupInfo` before `seal_commit`; if `seal_commit` rejects, in-memory state can be left as a live zero-admin group.
2. **Reserved-role signed apply gap.** REST assignment accepts only `admin` / `member`, but gossip `MemberRoleUpdated` apply rejects `Owner` while still accepting `Moderator` / `Guest`.

## Scope

- Patch only the concrete blockers above.
- Add tests proving the blockers are closed.
- Correct the two retro-identified evidence overclaims so they do not ride into the PR:
  - Slice 2 checkpoint: replace “byte-for-byte legacy Owner chain replay” wording with a narrower claim: fixed legacy Owner roster serialization/root stability plus current-code replay over Owner-containing rosters. Do not claim a genuine pre-Slice-2 historical chain fixture unless one is added.
  - Slice 3 checkpoint: replace “REST coverage” wording where it implies actual daemon-handler coverage with “REST-semantics/helper coverage”; true daemon HTTP coverage remains maintainer-gate / ignored-suite exposure unless explicitly added.
- Commit currently untracked/modified planning artifacts so the source of truth is reproducible from the planning branch.

## Reserved-role replay principle

Before changing apply behavior, actively confirm whether legacy `Moderator` / `Guest` role-update commits could exist under pre-ADR-0016 shipped code.

Principle:

- reject new reserved-role assignments;
- keep historical replay valid.

If historical `Moderator` / `Guest` role-update commits could have been produced by shipped code, stop and surface instead of breaking replay. If they were never practically assignable, record the evidence in the remediation checkpoint and reject them on the apply path.

## Out of scope

- Slice 4 invite changes.
- Full Slice 5 leave/disband endpoint split.
- KeyPackage distribution, Phase 2.
- Deterministic committer / rekey, Phase 3.
- Broad actor/sender/committer normalization across all gossip arms, unless a tiny local role-update cleanup is necessary for the reserved-role fix.
- Rewriting old checkpoints beyond correcting the named overclaim wording; remediation evidence belongs in a new Slice 3R checkpoint.

## Implementation sequence

1. Planning hygiene:
   - Commit the cadence amendment, Slice 3 packet, Slice 4 packet, Slice 3R packet, this addendum, and retro checkpoint to `gsd/adr-0016-planning`.
2. Fix last-admin self-leave:
   - Audit `DELETE /groups/:id` paths at current head.
   - For non-creator self-leave, avoid mutating live `GroupInfo` before the last-admin check can reject.
   - Preferred fix: clone-first plus explicit last-admin precheck before mutation/side effects.
   - It is acceptable to introduce the Slice 5 “before leaving” 409 string early, but do not implement the full Slice 5 leave/disband split.
   - Preserve existing creator-delete behavior for Slice 5.
3. Fix reserved-role gossip apply:
   - Confirm whether Moderator/Guest role-update commits could have been produced by shipped code.
   - If safe, reject `Owner`, `Moderator`, and `Guest` on `MemberRoleUpdated` apply path.
   - Keep REST assignment behavior unchanged.
   - Preserve legacy stored roster parsing and legacy Owner entries.
4. Tests:
   - Fast-gate tests: non-creator last-admin self-leave rejection does not mutate original group state; exact “before leaving” error string if surfaced through a testable helper; signed/gossip-style `MemberRoleUpdated` rejects `Owner`, `Moderator`, and `Guest`; `Admin` / `Member` role updates still apply; Owner-to-admin normalization remains valid.
   - Maintainer-gate / ignored daemon test only if feasible without broad harness work; do not build a flaky mesh test for a deterministic state-level property.
5. Verification:
   - Run mandatory Rust checks in exact order: `cargo fmt --all`; `cargo clippy --all-features --all-targets -- -D warnings`; `cargo check --workspace --all-targets`.
   - Run targeted remediation tests and relevant existing Slice 1-3 tests.
   - Push only to Jim’s fork and confirm PR #5 CI.
6. Review:
   - Run adversarial confirmation on the remediation delta.
   - Run craft confirmation on the remediation delta.
   - Record both in `gsd/checkpoints/2026-06-14-slice-3r-retro-remediation.md`.

## Done when

- Both HIGH adversarial findings are fixed or explicitly accepted by Jim.
- The craft CONFORMANCE finding is fixed or explicitly accepted by Jim.
- Required checks pass.
- PR #5 CI is green at the remediation head.
- Remediation checkpoint is filed.
- Slice 4 is unblocked.
