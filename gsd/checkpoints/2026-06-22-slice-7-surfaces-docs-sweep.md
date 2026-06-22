# ADR-0016 Phase 1 Slice 7 — surfaces/docs sweep

Date: 2026-06-22
Role: Implementer
Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
Base head at dispatch: `b16c34c62e98933716fda1b1434d8961f6ec168d`

## Goal

Finalize ADR-0016 Phase 1 Slice 7: no owner-required user/admin surface remains, legacy `owner` remains readable/admin-equivalent, docs say plainly that Admin is group root, and the provisional group-ending verb remains isolated as `disband` pending maintainer answer on #107.

## Sources read

- Build worktree `AGENTS.md`, `CLAUDE.md`, `tests/CLAUDE.md`
- `gsd/plan/phase-1-plan.md` lines 198-216
- `gsd/spec/phase-1-authority-alignment.md`
- `docs/adr/0016-role-based-group-authority-flat-admin.md`
- `gsd/ci-arbiter.md`

## Surfaces checked and dispositioned

| Surface / finding | Disposition | Notes |
|---|---:|---|
| `docs/api-reference.md` known finding: `AdminOnly` write-access said only `Admin`/`Owner` may send | clean | Already read as active admins with legacy `Owner` counted admin-equivalent. |
| `docs/api-reference.md` known finding: `POST /groups/:id/state/seal` documented `(owner/admin)` | clean | Already documented as any admin. |
| `docs/api-reference.md` known finding: `POST /groups/:id/state/withdraw` documented `(owner)` | clean | Already documented as `x0x group disband` / any admin. |
| `docs/api-reference.md` known finding: add/remove rows described as Creator-authored | clean | Already documented as Admin-authored member add/removal. |
| `docs/api-reference.md` role-assignment depth | fixed | Added policy/role/ban/request rows plus a `Roles` explainer: Admin is root; `admin`/`member` assignable; legacy `owner` readable/admin-equivalent, not assignable. |
| `docs/api-reference.md` `/mls` invariant | fixed | Added maintainer-follow-up invariant: legacy/raw `/mls/groups` must not expose usable key material or reactivate a withdrawn named group; named-group tombstone/terminality marker remains authoritative. No behavior implemented. |
| `src/api/mod.rs` endpoint descriptions | fixed | Changed `group policy` description from owner-only to admin+. Updated generated `docs/design/api-manifest.json` to match the registry. |
| `src/bin/x0x.rs` CLI help | fixed | `group policy` help now says admin+; `group set-role --help` lists only `admin` and `member` with one-line meanings and states legacy `owner` is readable/admin-equivalent but not assignable; `group disband` help says terminality marker, not shell. |
| `src/cli/commands/group.rs` | clean | Existing `DISBAND_VERB` remains the isolated provisional verb constant; no stale user-facing owner/withdraw copy found in this module. |
| GUI surface `src/gui/x0x-gui.html` | fixed | Deliberate in-scope R9 behavioral surface fix: policy, rename, state seal/disband, roster role/ban controls now gate on `isAdminOrAbove` instead of owner-only; legacy owner still renders/readable and counts as admin-equivalent for control display. Button/copy uses `Disband` instead of bare withdraw. |
| `src/bin/gui_coverage.rs` | clean | No stale authority/withdraw copy found; coverage checker passed at 98.3% with threshold 95. |
| `src/bin/x0xd.rs` withdrawn retained-record terminology | fixed | Renamed `retain_withdrawn_group_shell` to `retain_withdrawn_group_tombstone` and changed comments/test messages from shell to tombstone/terminality marker. No behavior changed. |
| `tests/named_group_integration.rs` withdrawn retained-record wording | fixed | Changed shell wording to tombstone/terminality marker in comments/assertion messages. |
| `tests/proptest_groups.rs` Slice-6 nit | fixed | Added explicit production pre-check assertion before the seal/choke-point assertion in `last_admin_rest_generated_zero_admin_attempts_hit_precheck_and_seal_chokepoint`. |
| `tests/parity_cli.rs` CLI role help coverage | fixed | Added `group_set_role_help_lists_only_assignable_roles`. |
| `docs/api.md` | fixed | Updated add/remove/policy/disband rows and added Admin-root role-assignment paragraph. |
| `docs/primers/groups.md` | fixed | Updated owner/creator/withdraw wording, Admin-root warning, AdminOnly write row, disband/tombstone language, and current-limits copy. |
| `README.md` | fixed | Updated current named-group note, Admin-root warning, state-chain signer language, and disband command comment. |
| `docs/conceptual-guide-for-humans.md` | fixed | Replaced owner terminal-withdrawal wording with Admin-root/disband language. |
| Historical/reference docs and proof reports containing older owner/creator wording | escalated / out of scope | Grep still finds historical accepted ADRs, proof reports, and long-form design notes with old owner language. These were not edited because accepted ADR text/proof artifacts are historical and the slice explicitly preferred docs/PR notes over ADR edits. |

## Deliberate GUI behavioral surface fix

Jim explicitly approved the GUI owner-gating swap for this slice. The GUI now exposes policy/state/role management controls to `admin` and legacy `owner`, not only `owner`. This is a behavioral surface alignment only; daemon/API authority remains enforced server-side. Legacy `owner` remains rendered as `owner` and is treated as admin-equivalent for display/control.

## PR-note bullets to carry forward

- Admin is root for a group: a hostile/compromised Admin can admit/remove members, rekey, change policy, assign roles, and disband the group; keep the admin set small and do not map softer app roles onto x0x Admin.
- `x0x group set-role` accepts only `admin` and `member`; legacy `owner` renders/readable as admin-equivalent for old groups but is not assignable. `moderator`/`guest` remain reserved and non-assignable.
- The group-ending user-facing verb is still provisional `disband` pending maintainer answer on #107; it is isolated for a pre-PR one-line swap if David chooses a different verb.
- Withdrawn named groups retain a keyless tombstone/terminality marker, not a "shell"; MLS/TreeKEM/GSS key material is wiped and the retained record blocks stale-card reanimation.
- Maintainer follow-up invariant: no legacy/raw `/mls/groups` endpoint may expose usable key material or reactivate a withdrawn named group. This slice records the invariant; it does not implement new `/mls` behavior.
- Local full nextest on this macOS host still hits the pre-existing daemon-startup timeout in `forged_member_joined_admin_role_or_secret_is_rejected`; baseline `b16c34c` reproduces the same failure.

## Verification evidence

Commands run in build worktree unless noted:

1. `cargo fmt --all`
   - Result: passed.
2. `cargo clippy --all-features --all-targets -- -D warnings`
   - Result: passed.
3. `cargo check --workspace --all-targets`
   - Result: passed.
4. `cargo nextest run --all-features --workspace`
   - First run failed because `docs/design/api-manifest.json` was stale after the `src/api/mod.rs` description change; manifest was updated.
   - Subsequent full runs failed only at `tests/harness/src/cluster.rs:68:17` with `x0xd pair-alice-<port> did not become healthy within 90s` in `x0x::named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected`; observed 1764 passed, 1 failed, 164 skipped, 7 not run (one run also reported one leaky passing test).
5. Baseline-diff reproduction:
   - Created detached baseline worktree at `/var/folders/f_/j942sskj6nx67b6gk3rqgsqm0000gn/T/opencode/x0x-base-b16c34c-slice7` for `b16c34c62e98933716fda1b1434d8961f6ec168d`.
   - Ran `cargo test --all-features --test named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected`.
   - Result: same failure at `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-59337 did not become healthy within 90s`.
6. Focused current-branch reproduction:
   - `cargo test --all-features --test named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected`
   - Result: same daemon startup timeout at `tests/harness/src/cluster.rs:68:17`.
7. GUI coverage:
   - `cargo run --bin gui-coverage -- --threshold 95`
   - Result: passed; 115/117 counted endpoints covered, 98.3% coverage, threshold 95.0%. Initial attempted command `cargo run --bin gui_coverage -- --threshold 95` failed because the bin target is named `gui-coverage`, not `gui_coverage`.
8. Focused tests:
   - `cargo test --all-features --test proptest_groups last_admin_rest_generated_zero_admin_attempts_hit_precheck_and_seal_chokepoint` — passed.
   - `cargo test --all-features --test parity_cli group_set_role_help_lists_only_assignable_roles` — passed.
   - `cargo test --all-features --test api_manifest manifest_matches_registry` — passed.

## Honesty / mechanism check

- No `.gsd/gate.sh`, CI workflow, test harness, service/daemon wrapper, build invocation, environment setup, `Cargo.toml`, or `Cargo.lock` changes.
- No PR action and no upstream push.
- No accepted ADR text edited.
- The only non-green required evidence is the full-workspace nextest daemon-startup timeout; it was reproduced on the dispatch base commit and is not introduced by Slice 7.

## Status

Implementation complete with one baseline-reproduced local nextest blocker. All slice changes are surgical and all introduced/focused checks pass; CI green of record still requires the existing mirror PR flow if/when the branch is pushed.

## Post-review wording remediation — 2026-06-22

- Code review wording findings remediated in build commit `1c3f17a58c94c04f4099014bfad121f04dc1b904`.
- GUI `nagWithdrawState` feedback now uses disband copy: `Group disbanded — public card superseded.` / `Disband failed.`
- `docs/api-reference.md` now describes `GET /groups/:id/requests` as `admin-only`, not `admin-authored`.
- Verification rerun in the build worktree: `cargo fmt --all`; `cargo clippy --all-features --all-targets -- -D warnings`; `cargo check --workspace --all-targets`; `cargo run --bin gui-coverage -- --threshold 95`; `cargo test --all-features --test parity_cli group_set_role_help_lists_only_assignable_roles` — all passed.
- No `.gsd/gate.sh`, CI workflow, test harness, service/daemon wrapper, build invocation, environment setup, `Cargo.toml`, or `Cargo.lock` changes; no push or PR action.
