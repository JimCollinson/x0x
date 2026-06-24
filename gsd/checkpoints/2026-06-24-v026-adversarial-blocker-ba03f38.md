# GSD Checkpoint — ADR-0016 Phase 1 v0.26 final gauntlet blocker at ba03f38

Date: 2026-06-24
Project: x0x
Slice/question: ADR-0016 Phase 1 v0.26 final readiness after terminality remediation
Prepared by: orchestrator
Agents/tools used: cleancontext, adversarial

## Status

Blocked — adversarial review returned NOT-READY with two HIGH findings.

Meaningful work-unit? Yes — non-trivial group-authority/security behavior intended for upstream PR.
Review cadence: final integrated gauntlet in progress; clean-context ran with concerns, adversarial failed, so Craft Review was not run.
Unreviewed backlog if deferred: Craft Review and final clean-context pass are pending until the blockers are fixed.

Note: `planning/STATE.md` does not exist in this repo/worktree; this checkpoint is the current state handoff for the blocker.

## What happened

- Final remediation candidate head was pushed to the fork-only CI mirror branch as `ba03f3844e1c7fe7e35619516e0870ef2949a8e5`.
- PR #6 CI became green at `ba03f38`: all substantive checks passed; `Soak Test` skipped by workflow schedule.
- Clean-context rerun returned `Concerns`:
  - no behaviour blocker found in the final terminality changes;
  - formal planning current-state was stale for exact head `ba03f38`;
  - local daemon-backed nextest still hits the known macOS mesh precondition in `named_group_join_metadata_event`;
  - `named_group_integration` default invocation runs zero tests because its daemon tests are ignored.
- Adversarial rerun returned `NOT-READY`:
  - HIGH: raw `validate_apply` still permits admin-authorized live→withdrawn without explicit terminal context;
  - HIGH: signed role-update apply accepts reserved `Moderator`/`Guest`, contradicting ADR/spec non-assignability;
  - MEDIUM: successful public `seal_withdrawal` leaves key material in the `GroupInfo` object, relying on server cleanup rather than the primitive's own terminalization.

## Evidence

CI arbiter / green of record:

- Location: PR #6, <https://github.com/JimCollinson/x0x/pull/6>
- Status: green at head `ba03f3844e1c7fe7e35619516e0870ef2949a8e5` for all substantive checks; `Soak Test` skipped by workflow schedule.
- Note: CI green is real but insufficient; the current suite does not cover raw `validate_apply` default-terminality semantics or reserved-role signed-apply rejection.

Local fast gate / `.gsd/gate.sh`:

- Installed? N/A — no `.gsd/gate.sh` exists in the implementation worktree.
- Clean-context/adversarial local checks included:
  - `cargo fmt --all -- --check` — pass
  - `cargo clippy --all-features --all-targets -- -D warnings` — pass
  - `cargo check --workspace --all-targets` — pass
  - `cargo nextest run --all-features --test named_group_state_commit` — pass
  - `cargo test --all-features metadata_` — pass
  - `cargo nextest run --all-features --test membership_authority` — pass
  - `cargo nextest run --all-features --test proptest_groups` — pass
  - `cargo nextest run --all-features --test last_admin_invariant` — pass
  - `cargo nextest run --all-features --test owner_retirement` — pass
  - `cargo test --all-features exec::service::tests` — pass
  - `cargo nextest run --all-features --test named_group_join_metadata_event` — failed locally before assertions with `has zero peers after 30s — mesh is disconnected`, matching the documented macOS daemon mesh precondition caveat; not used as readiness evidence.

Files changed/artifacts produced this step:

- Planning: this checkpoint.

## Honesty rules check

- No-harness-modification: Pass. No `.github`, `.gsd`, `tests/harness`, dependency, networking, bootstrap, or presence changes in the implementation branch.
- Baseline-diff for evidence: Concern documented. Local daemon mesh failure was not used as readiness evidence; PR #6 CI is green, and the local failure matches the known pre-assertion mesh signature documented in `gsd/ci-arbiter.md`.
- Evidence reproducible-from-branch: Concern. CI evidence is reproducible from PR #6 head; formal GSD spec/checkpoint evidence lives in the planning worktree, not the implementation branch.
- Local vs CI consistency: CI is green; local daemon-backed suite remains weaker and explicitly not the green of record.

## Review findings

Clean-context test:

- Reviewer/tool: cleancontext
- Result: Concerns
- Findings:
  - Behaviour is mostly discoverable from docs/source/tests; no behaviour blocker found.
  - Previous stale role-rule wording is fixed in `docs/design/named-groups-full-model.md`.
  - `audit.jsonl` dirtiness appears remediated; `audit.jsonl` is not tracked and was not recreated by `exec::service` tests.
  - Planning current-state was stale for exact final head `ba03f38`; this checkpoint records the current state.
  - Local daemon-backed tests still have known macOS mesh precondition failures; CI is the green of record.

Adversarial review:

- Reviewer/tool: adversarial (`openai/gpt-5.5`; implementer model/provider not provided, so cross-model independence cannot be proven beyond fresh-context review)
- Required? Yes — meaningful upstream/security-relevant work.
- Result: Blockers / NOT-READY
- Findings:
  - HIGH — raw `validate_apply` still permits live→withdrawn without explicit terminal context.
    - `src/groups/state_commit.rs:629-642`: `ActionKind::AdminOrHigher` plus active Admin/legacy Owner returns `Ok(())` for live→withdrawn.
    - `src/groups/state_commit.rs:668-670`: docs present `validate_apply` as the pre-mutation validation gate.
    - Risk: a raw caller can treat `Ok(())` as permission to mutate and skip explicit `GroupDeleted` terminal tombstone/key-wipe handling.
  - HIGH — signed role-update apply accepts reserved `Moderator`/`Guest`, contradicting accepted ADR/spec non-assignability.
    - `docs/adr/0016-role-based-group-authority-flat-admin.md:139-142`: reserved roles stay non-assignable.
    - `gsd/spec/phase-1-authority-alignment.md:43-44`: role assignment accepts exactly `admin` and `member`.
    - `src/server/mod.rs:9084-9090`: signed gossip apply rejects only `Owner`.
    - `tests/membership_authority.rs:735-740`: test deliberately asserts signed `Moderator`/`Guest` role updates apply.
  - MEDIUM — public `seal_withdrawal` leaves key material in `GroupInfo` after successful local terminal seal.
    - `src/groups/mod.rs:600-603`: sets `self.withdrawn = true` and returns the commit without clearing `shared_secret`.
    - `src/server/mod.rs:12991-12995`: daemon delete endpoint clears key material later.
    - Risk: non-server callers can hold `withdrawn == true` and key material unless the contract says sealing only and caller must separately terminalize.

Craft Review:

- Reviewer/tool: Not run
- Required? Yes after adversarial clears.
- If Not run: deferred because adversarial found HIGH blockers; running Craft Review before remediation would waste review.

## Drift / scope concerns

- Do not open or mark ready for upstream PR.
- PR #6 CI mirror can remain draft/open for internal CI only.
- Remediation should stay within ADR-0016 Phase 1 authority/terminality and docs/test alignment; no wire/hash/signing/storage/dependency/harness/CI changes.
- Accepted ADR-0016 must not be edited. If reserved-role signed-apply behaviour is intended as a legacy-replay carve-out, stop for an explicit ADR/spec addendum rather than silently diverging.

## Open questions / decisions for Jim

- None required for the raw terminality blocker: default/raw validation should reject live→withdrawn unless explicit terminal context is used.
- Possible decision if maintainers intentionally want signed `Moderator`/`Guest` replay to remain accepted for historical chains: that needs an explicit ADR/spec carve-out that distinguishes legacy replay from new head commits. Current ADR/spec text says non-assignable.

PR / upstream action gate:

- PR ready to raise? No.
- Jim confirmed upstream PR may be opened? No / N/A.
- Draft PR title/description prepared: planning draft exists, but not maintainer-ready until blockers are fixed and reviews rerun.

## Recommended next step

Create another remediation slice:

1. Make live→withdrawn terminality explicit in `validate_apply`: reject live→withdrawn by default and add a terminal-only context/action used solely by the `GroupDeleted` path, or equivalent API split. Add a direct regression test that `validate_apply` rejects live→withdrawn without terminal context.
2. Align signed role-update apply with ADR/spec: reject `Moderator`/`Guest` for new role-update commits, or stop for Jim/maintainer decision and ADR/spec addendum if legacy replay requires a distinct carve-out.
3. Decide/fix `seal_withdrawal` key-material cleanup semantics: either clear `shared_secret` on successful seal or make the seal-only contract explicit and require terminal local cleanup through a separate API. Prefer clearing for the safety invariant. Add a regression test.
4. Re-run mandatory Rust gates: `cargo fmt --all`, `cargo clippy --all-features --all-targets -- -D warnings`, `cargo check --workspace --all-targets`, plus focused authority/terminality/role tests.
5. Push to PR #6, require fresh CI green.
6. Rerun code review, verifier, clean-context, adversarial, then Craft Review.

## Handoff note

The branch is **not ready** despite green PR #6 CI. The prior non-`GroupDeleted` server apply bypass and `seal_withdrawal` failed-call mutation were improved, but final adversarial review found a lower-level raw validation terminality bypass plus an ADR/spec divergence in signed reserved-role apply. Fix these before any final checkpoint or upstream PR gate.
