# GSD Checkpoint — ADR-0016 Phase 1 v0.26 final gauntlet blocker

Date: 2026-06-24
Project: x0x
Slice/question: ADR-0016 Phase 1 v0.26 final readiness after withdrawal-authority remediation
Prepared by: orchestrator
Agents/tools used: cleancontext, adversarial

## Status

Blocked — adversarial review returned NOT-READY with a HIGH terminal-withdrawal-path bypass.

Meaningful work-unit? Yes — non-trivial group-authority/security behavior intended for upstream PR.
Review cadence: final integrated gauntlet in progress; adversarial failed, so Craft Review was not run.
Unreviewed backlog if deferred: Craft Review and final clean-context rerun are pending until the blocker is fixed.

Note: `planning/STATE.md` does not exist in this repo/worktree; this checkpoint is the current state handoff for the blocker.

## What happened

- Implementation docs cleanup was committed on `feat/adr-0016-phase-1-on-v0.26` as `4468921 docs(groups): update role rules for ADR-0016`.
- PR #6 body was updated to name current remediation/doc head `4468921` and remain fork-only/do-not-merge.
- PR #6 CI was rerun at final candidate head `446892123ba06267941be71bb819d649c725de38`; all substantive checks passed and `Soak Test` was skipped by workflow schedule.
- Planning verification evidence was committed/pushed as `c084581 chore(gsd): record withdrawal authority verification`.
- Clean-context rerun returned `Concerns`, not `Pass`:
  - stale role-rule wording remains in `docs/design/named-groups-full-model.md:619-634`;
  - full local nextest on macOS still hits the known pre-assertion daemon mesh failure;
  - full nextest mutates tracked `audit.jsonl` locally.
- Adversarial rerun returned `NOT-READY`:
  - the previous non-admin `MemberRemoved` self-leave `withdrawn=true` bypass is closed;
  - but admin-authorized non-`GroupDeleted` metadata events can still carry `withdrawn=true`, marking the group withdrawn without going through the explicit delete terminalization/key-wipe path;
  - `seal_withdrawal` also mutates `GroupInfo.withdrawn` before authorization, so a failed non-admin call leaves local state corrupted.

## Evidence

CI arbiter / green of record:

- Location: PR #6, <https://github.com/JimCollinson/x0x/pull/6>
- Status: green at head `446892123ba06267941be71bb819d649c725de38` for all substantive checks; `Soak Test` skipped by workflow schedule.
- Note: CI green is real but insufficient; the current suite does not cover admin-signed non-delete events carrying `withdrawn=true` or the `seal_withdrawal` failure-atomicity bug.

Local fast gate / `.gsd/gate.sh`:

- Installed? N/A — no `.gsd/gate.sh` exists in the implementation worktree.
- Clean-context/adversarial local checks included `cargo fmt --all -- --check`, `cargo clippy --all-features --all-targets -- -D warnings`, and `cargo check --workspace --all-targets`, all passing.
- Local full `cargo nextest run --all-features --workspace` failed before assertions with `has zero peers after 30s — mesh is disconnected`, matching the documented macOS daemon mesh precondition caveat; this local full-suite result was not used as readiness evidence.

Files changed/artifacts produced this step:

- Implementation: `docs/design/named-groups-full-model.md` at `4468921`.
- Planning: `gsd/evidence/2026-06-24-adr-0016-withdrawal-authority-remediation-verification.md`, this checkpoint.

## Honesty rules check

- No-harness-modification: Pass. No `.github`, `.gsd`, `tests/harness`, dependency, networking, bootstrap, or presence changes in the implementation branch.
- Baseline-diff for evidence: Concern documented. Local full-suite failure was not used as readiness evidence; PR #6 CI is green, and the local failure matches the known pre-assertion mesh signature already documented in `gsd/ci-arbiter.md`.
- Evidence reproducible-from-branch: Concern. CI evidence is reproducible from PR #6 head; local full nextest on this Mac is not clean because of the daemon mesh precondition and `audit.jsonl` side effect.
- Local vs CI consistency: CI is green; local full nextest is weaker and explicitly not the green of record.

## Review findings

Clean-context test:

- Reviewer/tool: cleancontext
- Result: Concerns
- Findings:
  - Stale role-rule doc gap remains: `docs/design/named-groups-full-model.md:619-634` still describes pre-ADR-0016 Owner/Admin/Moderator powers that conflict with the accepted ADR and the updated local rule text at `docs/design/named-groups-full-model.md:426-430`.
  - Local full-suite reproducibility remains weak on this Mac due known daemon mesh precondition.
  - Full nextest mutates tracked `audit.jsonl`; clean-context restored it after inspection.
  - PR #6 stale-body concern is resolved.

Adversarial review:

- Reviewer/tool: adversarial
- Required? Yes — meaningful upstream/security-relevant work.
- Result: Blockers / NOT-READY
- Findings:
  - HIGH — live→withdrawn is still accepted through non-`GroupDeleted` admin event arms, bypassing the explicit delete terminalization path.
    - `src/groups/state_commit.rs:629-648`: live→withdrawn is accepted for any `ActionKind::AdminOrHigher` plus active Admin/legacy Owner.
    - `src/server/mod.rs:8778-8829`: `MemberRemoved` admin-removal path can select `AdminOrHigher` and apply the supplied commit without rejecting `commit.withdrawn`.
    - `src/groups/mod.rs:630-644`: `finalize_applied_commit` copies `commit.withdrawn` into the state.
    - `src/server/mod.rs:8905-8957`, `12347-12370`, `12919-12952`: contrast — explicit `GroupDeleted`/delete path runs terminal cleanup/tombstone/key-wipe handling.
  - MEDIUM — `seal_withdrawal` mutates `GroupInfo.withdrawn` before authorization; failed non-admin call leaves corrupted local state.
    - `src/groups/mod.rs:584-590`: sets `self.withdrawn = true` before calling `seal_commit`.
    - `src/groups/mod.rs:499-507`: authorization check happens later inside `seal_commit`.
    - Existing tests set `withdrawn = true` before direct-seal checks and do not assert rollback on failed `seal_withdrawal`.
- Previous CRITICAL retry: exact non-admin self-leave bypass is closed, but broader non-delete withdrawal remains unresolved.

Craft Review:

- Reviewer/tool: Not run
- Required? Yes after adversarial clears
- If Not run: deferred because adversarial found a HIGH blocker; running Craft Review before remediation would waste review.

## Drift / scope concerns

- Do not open or mark ready for upstream PR.
- PR #6 CI mirror can remain draft/open for internal CI only.
- The remediation should stay within ADR-0016 Phase 1 authority/terminality and docs cleanup; no wire/hash/signing/storage/dependency/harness/CI changes.

## Open questions / decisions for Jim

None required if ADR-0016 intent stands: only the explicit admin delete/terminal withdrawal path should be able to live→withdraw a group and run terminal cleanup.

PR / upstream action gate:

- PR ready to raise? No.
- Jim confirmed upstream PR may be opened? No / N/A.
- Draft PR title/description prepared: planning draft exists, but not maintainer-ready until blockers are fixed and reviews rerun.

## Recommended next step

Create a remediation slice:

1. Make live→withdrawn event-kind explicit: reject `commit.withdrawn` on every non-`GroupDeleted` metadata arm while current state is live, or route any accepted live→withdrawn transition through the same terminal tombstone/key-wipe cleanup path. Prefer explicit `GroupDeleted` gating to match the product/API contract.
2. Add regression tests for at least:
   - admin `MemberRemoved` carrying `withdrawn=true` is rejected or terminalized through the proper delete path;
   - one non-membership admin arm carrying `withdrawn=true` is rejected;
   - legitimate admin `GroupDeleted` / delete-withdraw still works.
3. Make `seal_withdrawal` failure-atomic: authorize before setting `self.withdrawn` or roll back on error. Add a non-admin `seal_withdrawal` test that returns `Unauthorized` and leaves `withdrawn == false`.
4. Clean up stale role-rule text in `docs/design/named-groups-full-model.md:619-634`.
5. Re-run mandatory Rust gates: `cargo fmt --all`, `cargo clippy --all-features --all-targets -- -D warnings`, `cargo check --workspace --all-targets`, plus focused authority/leave/withdraw tests.
6. Push to PR #6, require fresh CI green.
7. Rerun code review, verifier, clean-context, adversarial, then Craft Review.

## Handoff note

The branch is **not ready** despite green PR #6 CI. The first adversarial blocker (non-admin self-leave withdrawal) was fixed, but the second adversarial pass found a broader non-`GroupDeleted` live→withdrawn bypass and a failure-atomicity bug in `seal_withdrawal`. Fix those before any final checkpoint or upstream PR gate.
