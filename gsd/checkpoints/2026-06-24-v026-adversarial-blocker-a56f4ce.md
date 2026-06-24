# GSD Checkpoint — ADR-0016 Phase 1 v0.26 adversarial blocker

Date: 2026-06-24
Project: x0x
Slice/question: ADR-0016 Phase 1 v0.26 final readiness
Prepared by: orchestrator
Agents/tools used: codereviewer, verifier, cleancontext, adversarial

## Status

Blocked — adversarial review returned NOT-READY with a CRITICAL authority bypass.

Meaningful work-unit? Yes — non-trivial group-authority/security behavior intended for upstream PR.
Review cadence: final integrated gauntlet in progress; adversarial failed, so Craft Review was not run.
Unreviewed backlog if deferred: Craft Review/final clean-context rerun are pending until the blocker is fixed.

## What happened

- Implementation docs cleanup was committed on `feat/adr-0016-phase-1-on-v0.26` as `a56f4ce docs(groups): align authority wording`.
- Planning cleanup/evidence commits were pushed on `gsd/adr-0016-planning`.
- PR #6 was green of record at implementation head `a56f4ceac4d2e38eafb113f725b33636dfae8aa0`.
- Code review rerun passed after the review-generated `audit.jsonl` dirtiness was cleared.
- Verifier passed 8/8 and evidence was recorded, but this readiness evidence is now superseded by the adversarial blocker below.
- Clean-context found concerns, not blockers: local full nextest on this Mac hit the known daemon mesh precondition, historical checkpoints may mislead until a final checkpoint exists, and PR-description sender-binding wording was too broad. The sender-binding wording was fixed in planning commit `3717c13`.
- Final adversarial review found a CRITICAL bypass: a non-admin member self-leave can carry a signed `withdrawn=true` commit through the `MemberRemoved` path, terminally withdrawing/deleting the group without using the explicit admin delete path.

## Evidence

CI arbiter / green of record:

- Location: PR #6, <https://github.com/JimCollinson/x0x/pull/6>
- Status: green at head `a56f4ceac4d2e38eafb113f725b33636dfae8aa0`.
- Note: CI green is real but insufficient; the current suite does not cover the adversarial withdrawn self-leave event shape.

Local fast gate / `.gsd/gate.sh`:

- Installed? N/A — no `.gsd/gate.sh` exists in the implementation worktree.
- Commands run by clean-context/verifier included fmt/clippy/check and targeted tests; local full `cargo nextest run --all-features --workspace` failed before assertions with `has zero peers after 30s — mesh is disconnected`, matching the documented macOS daemon mesh precondition caveat.

Files changed/artifacts produced this session:

- Implementation: `docs/design/named-groups-full-model.md`
- Planning: `gsd/README.md`, `gsd/ci-arbiter.md`, `gsd/spec/phase-1-authority-alignment.md`, `gsd/plan/phase-1-plan.md`, `gsd/plan/phase-1-plan-addendum-2-slice-3r-remediation.md`, `gsd/plan/phase-1-pr-notes.md`, `gsd/plan/phase-1-pr-description.md`, `gsd/evidence/2026-06-24-adr-0016-v026-final-verification.md`, this checkpoint.

## Honesty rules check

- No-harness-modification: Pass. No `.github`, `.gsd`, `tests/harness`, dependency, networking, bootstrap, or presence changes in the implementation branch.
- Baseline-diff for evidence: Concern documented. Local full-suite failure was not used as readiness evidence; PR #6 CI is green, and the local failure matches the known pre-assertion mesh signature already documented in `gsd/ci-arbiter.md`.
- Evidence reproducible-from-branch: Concern. CI evidence is reproducible from PR #6 head; local full nextest on this Mac is not clean because of the daemon mesh precondition.
- Local vs CI consistency: CI is green; local full nextest is weaker and explicitly not the green of record.

## Review findings

Clean-context test:

- Reviewer/tool: cleancontext
- Result: Concerns
- Findings:
  - Local full `nextest` is not reproducibly green on this Mac due known mesh precondition.
  - Historical checkpoints contain old PR #5 / old-head context; this checkpoint records the current blocker.
  - Sender-binding PR-description wording was too broad; fixed in `gsd/plan/phase-1-pr-description.md`.

Code review:

- Reviewer/tool: codereviewer
- Result: Pass after rerun
- Findings: prior LOW dirty `audit.jsonl` process issue closed; no remaining findings.

Verifier:

- Reviewer/tool: verifier
- Result: Passed before adversarial review
- Disposition: superseded for readiness by the adversarial CRITICAL finding below.

Adversarial review:

- Reviewer/tool: adversarial
- Required? Yes — meaningful upstream/security-relevant work.
- Result: Blockers / NOT-READY
- Finding: CRITICAL — `MemberRemoved` self-leave can carry `withdrawn=true`, letting any active member terminally withdraw/delete the group.
- Evidence anchors from adversarial review:
  - `src/server/mod.rs:8790-8801`: `MemberRemoved` accepts self-leave with `self_leave_auth` and selects `ActionKind::MemberSelf`.
  - `src/server/mod.rs:8821-8830`: applies the supplied commit through `apply_stateful_event_to_group` while mutating only `remove_member`.
  - `src/groups/state_commit.rs:684-689`: `ActionKind::MemberSelf` requires only an active signer.
  - `src/groups/mod.rs:621-635`: `finalize_applied_commit` copies `commit.withdrawn` and then last-admin invariant is exempt when withdrawn.
  - `src/groups/state_commit.rs:592-598`: withdrawn states bypass the last-admin invariant.
  - `src/server/mod.rs:8905-8932`: contrast — `GroupDeleted` requires `commit.withdrawn` and applies as `AdminOrHigher`; `MemberRemoved` lacks `commit.withdrawn` rejection.
- Why this blocks: Phase 1 contract separates member self-leave from explicit any-admin group delete/terminal withdrawal. This event shape lets a plain member smuggle terminal withdrawal through the self-leave path.

Craft Review:

- Reviewer/tool: Not run
- Required? Yes after adversarial clears
- If Not run: deferred because adversarial found a CRITICAL blocker; running Craft Review before remediation would waste review.

## Drift / scope concerns

- Do not open or mark ready for upstream PR.
- PR #6 CI mirror can remain draft/open for internal CI only; its body is stale and should be updated before any maintainer-facing action, but PR creation upstream remains Jim-gated.

## Open questions / decisions for Jim

None needed to remediate the blocker if ADR-0016 intent stands: member self-leave must not be able to set `withdrawn=true`; explicit group delete/withdraw remains admin-authorized.

PR / upstream action gate:

- PR ready to raise? No.
- Jim confirmed upstream PR may be opened? No / N/A.
- Draft PR title/description prepared: planning draft exists, but not maintainer-ready until blocker fixed and reviews rerun.

## Recommended next step

Create a remediation slice:

1. Reject `commit.withdrawn` on `MemberRemoved` self-leave / all non-`GroupDeleted` event arms, or otherwise require `withdrawn=true` to imply `AdminOrHigher` and the explicit terminal delete path.
2. Add regression tests:
   - non-admin self-leave with `withdrawn=true` is rejected;
   - ordinary self-leave does not mark group withdrawn;
   - admin `GroupDeleted` / delete-withdraw still works.
3. Re-run mandatory Rust gates: `cargo fmt --all`, `cargo clippy --all-features --all-targets -- -D warnings`, `cargo check --workspace --all-targets`, plus focused authority/leave/withdraw tests.
4. Push to PR #6, require fresh CI green.
5. Rerun code review, verifier, clean-context, adversarial, and Craft Review.

## Handoff note

Read this checkpoint before trusting `gsd/evidence/2026-06-24-adr-0016-v026-final-verification.md`: the verifier passed before adversarial found a CRITICAL authority bypass. The branch is **not ready** until the withdrawn self-leave bypass is fixed and the gauntlet reruns.
