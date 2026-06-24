# ADR-0016 withdrawal-authority remediation verification

Date: 2026-06-24
Verifier: gpt-5.5 verifier
Implementation worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-v026`
Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
Implementation code head verified: `7cd8fefc378c64d2969f38ae9cff609c53c254ec`
Final candidate head after docs-only hygiene cleanup: `446892123ba06267941be71bb819d649c725de38`
PR: <https://github.com/JimCollinson/x0x/pull/6>

## Result

Passed: 9/9 requested acceptance items verified.

The previous adversarial blocker (`a56f4ce`) was that a non-admin member could smuggle `withdrawn=true` through `MemberRemoved` self-leave and terminally withdraw the group. At `7cd8fef`, live-to-withdrawn commits now require `ActionKind::AdminOrHigher` plus an active Admin/legacy Owner signer at the central `validate_apply` choke point, and direct authoring of withdrawn commits through `seal_commit` is guarded. The later final candidate head `4468921` adds documentation-only cleanup to remove stale role-rule wording from `docs/design/named-groups-full-model.md`.

## Sources read

- `gsd/checkpoints/2026-06-24-v026-adversarial-blocker-a56f4ce.md`
- `gsd/spec/phase-1-authority-alignment.md`
- `gsd/plan/phase-1-plan.md`
- `gsd/plan/phase-1-pr-notes.md`
- `gsd/README.md`
- `docs/adr/0016-role-based-group-authority-flat-admin.md`
- `src/groups/state_commit.rs`
- `src/groups/mod.rs`
- `src/server/mod.rs`
- `tests/membership_authority.rs`
- `tests/proptest_groups.rs`
- `tests/last_admin_invariant.rs`
- `tests/named_group_integration.rs`
- `.github/workflows/{ci,integration,security,build}.yml`

## Commands/checks run or inspected

- `git status --short && git rev-parse --abbrev-ref HEAD && git rev-parse HEAD` in both worktrees: clean, implementation at `7cd8fef`, planning at `b0626ac` before writing this evidence note.
- `git show --stat --oneline --decorate --no-renames a56f4ce..7cd8fef`: remediation commit touched only `src/groups/mod.rs`, `src/groups/state_commit.rs`, and tests.
- `git diff --name-status a56f4ce 7cd8fef`: same file set; no Cargo, workflow, docs, storage, or dependency files changed by the remediation.
- `git diff --exit-code a56f4ce 7cd8fef -- Cargo.toml Cargo.lock .github .cargo`: no output, no dependency/workflow changes.
- `gh pr view 6 --repo JimCollinson/x0x --json ...`: PR #6 final candidate head is `4468921`, draft/open, base `main`.
- `gh pr checks 6 --repo JimCollinson/x0x --watch --interval 30`: all substantive checks passed at `4468921`; `Soak Test` skipped by schedule.
- `cargo nextest run --all-features -E 'test(membership_authority_non_admin_self_leave_with_withdrawn_true_rejected_at_apply_choke_point) | test(membership_authority_direct_seal_withdrawn_state_requires_admin_or_legacy_owner) | test(membership_authority_group_deleted_withdrawal_commit_applies_under_admin_authority) | test(withdrawal_flag_authority_matches_transition_oracle)'`: 4/4 passed.
- `cargo nextest run --all-features -E 'test(membership_authority_plain_member_self_leave_converges) | test(membership_authority_non_last_admin_self_leave_converges)'`: 2/2 passed.
- `cargo nextest run --all-features -E 'test(membership_authority_creator_self_leave_converges_when_another_admin_remains) | test(membership_authority_legacy_owner_self_leave_converges_when_admin_remains) | test(withdrawn_card_non_admin_cannot_terminally_mark_keyed_live_group) | test(withdrawn_card_admin_cannot_terminally_mark_keyed_live_group_without_signed_commit) | test(withdrawn_card_can_supersede_keyless_discovery_stub_without_roster_admin)'`: 5/5 passed.
- `cargo nextest run --all-features -E 'test(last_admin_gossip_apply_allows_zero_admin_withdrawal_commit)'`: 1/1 passed.
- `cargo fmt --all -- --check`: passed. I used check mode to avoid writing files during verification.
- `cargo clippy --all-features --all-targets -- -D warnings`: passed.
- `cargo check --workspace --all-targets`: passed.
- `git status --short` after local commands: implementation worktree still clean.

## Requirements matrix

| # | Requirement | Status | Evidence |
|---|---|---|---|
| 1 | Non-admin self-leave with `withdrawn=true` is rejected. | Pass | `src/groups/state_commit.rs:629-648` rejects live-to-withdrawn unless `ActionKind::AdminOrHigher` and signer is active Admin/legacy Owner. `src/server/mod.rs:8778-8829` routes self-leave `MemberRemoved` through `ActionKind::MemberSelf`, so the central check rejects a non-admin withdrawn self-leave before mutation. Regression `membership_authority_non_admin_self_leave_with_withdrawn_true_rejected_at_apply_choke_point` passed locally. |
| 2 | Ordinary self-leave does not mark group withdrawn. | Pass | `src/server/mod.rs:12983-13042` and `12420-12500` author `MemberRemoved` self-leave events through `seal_commit`, without setting `withdrawn`. `tests/membership_authority.rs:267-300` asserts self-leave commits and applied state are not withdrawn. Plain member and non-last-admin self-leave tests passed locally. |
| 3 | Admin delete/withdraw still works. | Pass | `src/server/mod.rs:12888-12981` requires Admin-or-higher, calls `seal_withdrawal`, emits `GroupDeleted`, and retains withdrawn tombstone/wipes key material. `tests/membership_authority.rs:658-676` proves promoted-admin GroupDeleted withdrawal applies under signed admin authority. Focused test passed locally. |
| 4 | Rule is central and wired: metadata commits, `GroupInfo::apply_commit`, server `apply_stateful_event_to_group`, and direct `seal_commit` hit the check. | Pass | Central check lives in `validate_apply` (`src/groups/state_commit.rs:675-740`). `GroupInfo::apply_commit` calls it (`src/groups/mod.rs:602-615`). Server helper calls it (`src/server/mod.rs:7526-7545`) and all metadata-event arms with `commit` route through that helper: MemberAdded, MemberRemoved, GroupDeleted, PolicyUpdated, MemberRoleUpdated, MemberBanned, MemberUnbanned, JoinRequestCreated/Approved/Rejected/Cancelled, GroupMetadataUpdated. Direct authoring is guarded in `GroupInfo::seal_commit` (`src/groups/mod.rs:490-508`) and `seal_withdrawal` delegates to it (`src/groups/mod.rs:584-590`). |
| 5 | Rule gates live -> withdrawn transition only; already-withdrawn carry-forward and un-withdraw rejection semantics are preserved. | Pass | `validate_live_to_withdrawn_authority` returns early when `current_withdrawn || !commit.withdrawn`; un-withdraw remains rejected by `ctx.current_withdrawn && !commit.withdrawn` terminality (`src/groups/state_commit.rs:629-648`, `712-724`). Property `withdrawal_flag_authority_matches_transition_oracle` models current-withdrawn/commit-withdrawn/action-kind combinations and passed locally. |
| 6 | Card-import path cannot become a non-admin sibling disband route for live/keyed groups; keyless stub behavior remains existing discovery behavior. | Pass | `withdrawn_card_can_terminally_mark_local_group` allows withdrawn cards only when they supersede and the local group is not protected/keyed (`src/server/mod.rs:12152-12159`), and import uses that guard (`src/server/mod.rs:14814-14859`). Unit tests prove non-admin and roster-admin withdrawn cards cannot terminate live/keyed groups, while keyless discovery stubs can still be tombstoned (`src/server/mod.rs:22715-22825`); focused tests passed locally. Ignored daemon integration tests in `tests/named_group_integration.rs:1520-1692` cover the same behavior at API level and are included in PR #6 Multi-Agent Integration. |
| 7 | Legacy Owner/Admin withdrawal replay still works. | Pass | Owner remains Admin-equivalent via `role.at_least(Admin)` in the new helper and authority check. `membership_authority_direct_seal_withdrawn_state_requires_admin_or_legacy_owner` proves legacy Owner withdrawal seals; `last_admin_gossip_apply_allows_zero_admin_withdrawal_commit` proves a legacy Owner/admin-authorized withdrawal applies through `ActionKind::AdminOrHigher`. Both passed locally. |
| 8 | No wire/hash/signing/storage/dependency/scope change. | Pass | Remediation diff from blocker touched only `src/groups/mod.rs`, `src/groups/state_commit.rs`, and tests. `GroupStateCommit` fields/signable bytes remain unchanged (`src/groups/state_commit.rs:353-430`). No Cargo/Cargo.lock/.github/.cargo changes. The code adds validation helpers and tests; no serialization, storage format, dependency, or workflow edits found. |
| 9 | Required checks and CI evidence sufficient under repo/GSD rules. | Pass | Local focused regressions passed. Local `fmt --check`, `clippy --all-features --all-targets -D warnings`, and `cargo check --workspace --all-targets` passed for the remediation head. PR #6 at final candidate head `4468921` has Format Check, Clippy Lint, Test Suite, Property Tests, Multi-Agent Integration, API Coverage Guard, Coverage Gate, Documentation, Security Audit, and build matrix all successful. Soak Test is skipped by workflow because it only runs on schedule, matching the provided caveat. |

## Evidence notes / caveats

- I did not run the full workspace nextest locally; PR #6 CI is the green of record and ran the workspace Test Suite plus the ignored named-group integration suite in the Multi-Agent Integration job. Local verification focused on the blocker/regression paths and mandatory quick gates.
- The post-remediation `4468921` commit is documentation-only hygiene: it updates `docs/design/named-groups-full-model.md` to match the accepted ADR-0016 role rules and does not change Rust code or shipped behavior.
- The central rule is authority-based, not event-kind-exclusive: any live-to-withdrawn commit accepted by `validate_apply` must be applied as `AdminOrHigher` by an active Admin/legacy Owner. The production explicit delete path emits `GroupDeleted`; non-admin `MemberRemoved` self-leave is blocked. If the intended future invariant is stricter (“only GroupDeleted may ever carry `withdrawn=true`”), that is a separate tightening not required by the acceptance list verified here.
- `cargo fmt --all` was not run in mutating mode during verification to avoid modifying the worktree; `cargo fmt --all -- --check` and CI Format Check both passed.

## Required remediation

None for the requested withdrawal-authority remediation.

Recommended next step: proceed to the next GSD gate/checkpoint. For PR/merge readiness, keep the stated GSD caveat: clean-context/adversarial/craft gates remain the orchestrator’s responsibility for the final PR candidate.
