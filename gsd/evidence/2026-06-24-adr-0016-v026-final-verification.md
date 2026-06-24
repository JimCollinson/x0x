# Verification — ADR-0016 Phase 1 v0.26 final

Date: 2026-06-24
Verifier: gpt-5.5
Implementation worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-v026`
Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
Implementation head verified: `a56f4ceac4d2e38eafb113f725b33636dfae8aa0`
Base/fork main for PR #6: `71a92f23234190b934cd616c76370084181aa1c6` (same tree as upstream `v0.26.0` `a6fce96b341a52b84026546413d246dda44ce713`)

## Result

passed — 8/8 requested goals verified. No required remediation found.

**Post-verification note:** final adversarial review later found a CRITICAL withdrawn self-leave authority bypass at the same implementation head. This verification result is superseded for readiness until that blocker is remediated and re-reviewed. See `gsd/checkpoints/2026-06-24-v026-adversarial-blocker-a56f4ce.md`.

## Sources read

- `docs/adr/0016-role-based-group-authority-flat-admin.md`
- `gsd/spec/phase-1-authority-alignment.md`
- `gsd/plan/phase-1-plan.md`
- `gsd/ci-arbiter.md`
- `gsd/plan/phase-1-pr-notes.md`
- `gsd/plan/phase-1-pr-description.md`
- `gsd/checkpoints/2026-06-22-phase-1-feature-complete-1c3f17a.md`
- `gsd/checkpoints/2026-06-22-final-polish-330965f.md`
- `src/groups/state_commit.rs`, `src/groups/member.rs`, `src/groups/mod.rs`, `src/groups/invite.rs`
- `src/server/mod.rs`, `src/bin/x0x.rs`, `src/cli/commands/group.rs`, `src/api/mod.rs`, `src/gui/x0x-gui.html`
- `README.md`, `docs/api-reference.md`, `docs/api.md`, `docs/primers/groups.md`, `docs/conceptual-guide-for-humans.md`
- Tests: `tests/last_admin_invariant.rs`, `tests/membership_authority.rs`, `tests/owner_retirement.rs`, `tests/invite_authority.rs`, `tests/proptest_groups.rs`, `tests/named_group_integration.rs`, `tests/named_group_join_metadata_event.rs`, `tests/parity_cli.rs`
- GitHub PR #6 metadata/check rollup via `gh pr view/checks`

## Commands/checks inspected or run

- `git status --short --branch` in implementation and planning worktrees — both clean before verification note.
- `git log --oneline -10` and `git diff --stat 71a92f2...HEAD`.
- `git diff --name-only 71a92f2...HEAD`.
- `git merge-base HEAD upstream/main`, `git rev-parse upstream/main`, `git rev-parse HEAD`, and tree comparison for fork-main/upstream v0.26.
- `gh pr checks 6 --repo JimCollinson/x0x --watch=false` — all substantive checks passed; Soak Test skipped.
- `gh pr view 6 --repo JimCollinson/x0x --json headRefOid,baseRefOid,statusCheckRollup,url,isDraft,state` — PR #6 head is `a56f4ce`, base is `71a92f2`, draft/open.
- `git diff --name-only 71a92f2...HEAD -- .github .gsd tests/harness Cargo.toml Cargo.lock src/network.rs src/bootstrap.rs src/presence.rs src/network src/bootstrap src/presence` — no output.
- `git diff --unified=0 71a92f2...HEAD -- src/server/mod.rs | rg ...` — server hunks are named-group/API/helper/test areas; no harness/CI/dependency/network/presence/bootstrap file changes.
- Local targeted tests:
  - `cargo test --all-features --test owner_retirement --test membership_authority --test invite_authority --test parity_cli` — 41/41 passed.
  - `cargo nextest run --all-features -E 'test(last_admin)'` — 36/36 passed, 2017 skipped by filter.

## Requirements matrix

| # | Requirement | Status | Evidence |
|---|---|---|---|
| 1 | Creator/Owner authority gates removed/replaced by committed-roster Admin role for Phase 1 admin acts. | pass | `ActionKind::OwnerOnly` and `require_owner` are absent from Rust code. `validate_apply` authorizes `AdminOrHigher` via `commit.committed_by` role lookup on `ctx.members_v2`; REST paths use `require_admin_or_above`; receive paths for add/remove/ban/policy/role/group-delete use `AdminOrHigher` signed-commit validation. Searches found no active group `creator_auth`, ownership-transfer stub, cannot-ban-owner, or cannot-remove-creator guard in code. |
| 2 | Last-admin invariant applies on REST and apply paths, with withdrawn terminal state exempt. | pass | `enforce_last_admin_invariant` checks post-mutation active Admin/Owner count and immediately exempts `withdrawn`; `seal_commit` enforces on authoring/REST paths; `finalize_applied_commit` enforces after post-mutation hash match on apply paths; REST prechecks return the exact 409 strings. Local `last_admin` nextest filter passed 36 tests covering REST/precheck, seal, gossip apply, property sequences, and withdrawal. |
| 3 | Legacy Owner compatibility retained; new assignment restricted to admin/member; reserved role authoring/apply split documented and covered. | pass | `GroupRole::Owner` remains parseable/serde-compatible and admin-equivalent via rank/`at_least`; `as_u8` and role-byte mapping remain stable. New group genesis uses `new_admin`; `new_owner` remains for legacy fixtures. `assignable_from_name` accepts only `admin`/`member` with exact ADR errors. Apply path rejects `Owner` role updates but accepts `Moderator`/`Guest` valid signed commits for convergence; this is documented in PR notes/description and covered by `membership_authority_signed_role_update_apply_accepts_current_and_legacy_labels`. |
| 4 | Invite/membership authority allows non-creator Admin where in scope; deferred delegated ban/KeyPackage and card-bound discovery/request-access limitations openly documented. | pass | `create_group_invite`, add/remove/ban, policy, role, request review, and delete paths require Admin role, not creator. Invite join derives creator provenance from base-state roster, not unsigned `invite.inviter`. Tests cover non-creator Admin invite, add/remove/ban, policy/role/delete, and daemon invite convergence. Delegated TreeKEM ban missing-KeyPackage remains a 424/FailedDependency boundary; PR notes document delegated ban/KeyPackage Phase 2 and card-bound discovery/request-access Phase 2 deferral. |
| 5 | Leave vs explicit delete/terminal withdrawal semantics match docs/CLI/API; REST DELETE self-leave; `x0x group delete` / POST state/withdraw terminal Admin action; no wire/hash/signing/storage/dependency changes. | pass | Routes wire `DELETE /groups/:id` to `leave_group` and `POST /groups/:id/state/withdraw` to `withdraw_group_state`; CLI primary `group delete` posts to `/state/withdraw`, with hidden `state-withdraw` alias. Docs/API/GUI describe leave vs delete and Admin authority. Tests cover creator DELETE as self-leave/409 when last admin, promoted Admin terminal delete propagation, CLI parsing/help. No `Cargo.toml`/`Cargo.lock`, role byte, serde role names, commit signable bytes, or commit hash format changes found. |
| 6 | Sender-binding disposition documented and not a hidden creator gate. | pass | Planning PR description explicitly says receive-path creator gates were removed, pre-existing `actor == sender_hex` binding was left as anti-spoofing/delivery-model binding, and `GroupDeleted` validates `actor == commit.committed_by` plus signed Admin-authorized withdrawal. Code sender checks are `actor == sender_hex` / expected-inviter checks, not creator comparisons; authority still runs through signed commit + Admin role. |
| 7 | CI/check evidence sufficient under `gsd/ci-arbiter.md`. | pass | PR #6 is draft fork-only CI mirror at head `a56f4ce`; check rollup has Format, Clippy, Test Suite, Property Tests, Multi-Agent Integration, API/GUI parity, API coverage, docs, coverage, builds, audit, panic scanner, release metadata all passing. Soak Test is skipped by workflow. Local targeted verification tests passed. No carve-out is needed for PR #6 because substantive checks are green. |
| 8 | Scope boundaries respected: no Phase 2/3, networking/transport/server lifecycle/harness changes. | pass | Changed files are docs/API/CLI/GUI/group authority/server named-group logic/tests only. No `.github`, `.gsd`, `tests/harness`, `Cargo.toml`, `Cargo.lock`, `src/network*`, `src/bootstrap*`, or `src/presence*` diffs. Phase 2 KeyPackage/card-bound authority and Phase 3 deterministic committer/race handling remain documented deferrals. |

## Evidence notes and caveats

- PR #6 check evidence is stronger than the earlier PR #5 raw-red/startup-timeout carve-out: PR #6 substantive checks are green at `a56f4ce`; only Soak Test is skipped by workflow.
- I did not run the full workspace `cargo nextest run --all-features --workspace` locally. I relied on PR #6 CI as green of record and ran focused local tests against the requested authority/leave/invariant surfaces.
- The actual draft CI mirror PR body is minimal and still mentions an older head (`776d2c8`); I treated `gsd/plan/phase-1-pr-description.md` and `phase-1-pr-notes.md` as the maintainer-description source because PR #6 is explicitly fork-only/CI-only.
- I did not find a final `a56f4ce` checkpoint file in planning docs. This is process hygiene, not a functional goal gap, given PR #6 green and direct source/test verification here.
- Residual owner-era wording exists in historical ADRs/proof reports/KV-store docs and file-permission contexts. Current named-group docs/CLI/API/GUI surfaces inspected for Phase 1 use Admin/legacy-owner language correctly.

## Required remediation

None.

Recommended hygiene before external handoff, not required for goal achievement: update the CI mirror PR body/final handoff checkpoint to mention `a56f4ce` and carry the final PR-description text from planning docs.
