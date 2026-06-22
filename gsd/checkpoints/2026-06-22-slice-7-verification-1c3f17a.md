# ADR-0016 Phase 1 Slice 7 — verification 1c3f17a

Date: 2026-06-22
Role: Verifier
Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
Head verified: `1c3f17a58c94c04f4099014bfad121f04dc1b904`
Review range: `b16c34c62e98933716fda1b1434d8961f6ec168d..1c3f17a58c94c04f4099014bfad121f04dc1b904`

## Goal checked

Slice 7 requires the final R9/R10/§3 sweep: no user/admin surface requires an `owner` to exist, legacy `owner` remains readable/admin-equivalent but non-assignable, docs plainly warn that Admin is root for the group, the group-ending verb is consistently `disband` with withdrawal kept as protocol/internal wording, and the sweep record is complete.

## Sources inspected

- `gsd/plan/phase-1-plan.md` lines 198-216.
- `gsd/spec/phase-1-authority-alignment.md`, especially R9, R10, §3.3, and acceptance criteria.
- `gsd/checkpoints/2026-06-22-slice-7-surfaces-docs-sweep.md`.
- Build diff and changed files at `1c3f17a`.
- `docs/api-reference.md`, `docs/api.md`, `README.md`, `docs/primers/groups.md`, `docs/conceptual-guide-for-humans.md`.
- `src/bin/x0x.rs`, `src/cli/commands/group.rs`, `src/api/mod.rs`, `src/gui/x0x-gui.html`, `src/bin/gui_coverage.rs`.
- `src/bin/x0xd.rs`, `tests/named_group_integration.rs`, `tests/parity_cli.rs`, `tests/proptest_groups.rs`.

## Artifact verification

- Status: VERIFIED.
- Existence: all named surfaces exist; sweep record exists at the requested path.
- Substantive: changes are real docs/help/GUI/test updates, not placeholders or stubs.
- Wired:
  - CLI help is wired through clap in `src/bin/x0x.rs` and covered by `tests/parity_cli.rs::group_set_role_help_lists_only_assignable_roles`.
  - API registry wording is wired through `src/api/mod.rs` and `docs/design/api-manifest.json`; `api_manifest` focused test passes.
  - GUI admin controls are wired through `nagRenderAdmin` using `isAdminOrAbove` for policy/state/roster/request controls.
  - `gui-coverage` runs against the changed GUI and registry.

## Goal-backward checks

- R9 surface sweep: verified GUI, `src/cli`, `src/api`, `src/bin/gui_coverage.rs`, and `docs/api-reference.md` were checked in the sweep record. Current head confirms the four known `docs/api-reference.md` findings are dispositioned:
  - `AdminOnly` now says only active admins may send, with legacy `Owner` counted admin-equivalent.
  - `POST /groups/:id/state/seal` says any admin.
  - `POST /groups/:id/state/withdraw` is `x0x group disband` / any admin.
  - add/remove rows are Admin-authored, not Creator-authored.
- GUI owner-gating fix: `src/gui/x0x-gui.html` now computes `isAdminOrAbove = myRole === 'owner' || myRole === 'admin'` and uses it for policy, rename, state seal/disband, roster role/ban, and request controls. Stale "Only the owner" / owner-only GUI copy was removed.
- R10 docs/help: Admin-root warning appears in repo docs; `x0x group set-role` help lists only `admin` and `member`, explains Admin power, and says legacy `owner` renders/read as admin-equivalent but cannot be assigned. Reserved `moderator`/`guest` are not listed as assignable in the help assertion.
- §3.3 language: user-facing CLI/GUI/api-reference surfaces use `disband` for the group-ending act; remaining withdrawal/withdrawn terms are paired with `disband` or refer to protocol fields/terminal state.
- Terminology: retained withdrawn record wording is now tombstone/terminality marker. The `retain_withdrawn_group_shell` helper was renamed to `retain_withdrawn_group_tombstone`; remaining `shell` matches inspected are unrelated exec/security/UI shell contexts, not the retained withdrawn record.
- `/mls` invariant: `docs/api-reference.md` records it as an "Operational invariant (maintainer follow-up)" and explicitly says it is not a new low-level MLS helper API; no behavior overclaim found.
- Slice-6 nit bundled: `tests/proptest_groups.rs::last_admin_rest_generated_zero_admin_attempts_hit_precheck_and_seal_chokepoint` now explicitly asserts the production pre-check error before exercising the seal/choke-point invariant rejection.
- Sweep record completeness: every required surface has a clean/fixed/escalated disposition, includes the deliberate GUI behavior fix, and carries PR-note bullets for Admin-root, assignable roles, provisional `disband`, tombstone/terminality marker, `/mls` follow-up, and the baseline daemon-startup timeout.
- Scope/mechanism: diff changed docs, GUI, API/CLI wording, one x0xd tombstone helper rename/call-site rename, and tests. No `.gsd/gate.sh`, CI workflow, test harness, service/daemon wrapper, build invocation, `Cargo.toml`, or `Cargo.lock` changes were found.

## Commands run by verifier

All commands were run in `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build` unless otherwise noted.

1. `git status --short && git rev-parse HEAD && git diff --name-status b16c34c62e98933716fda1b1434d8961f6ec168d..1c3f17a58c94c04f4099014bfad121f04dc1b904`
   - Result: clean build worktree; head `1c3f17a58c94c04f4099014bfad121f04dc1b904`; changed files are the expected docs/source/tests only.
2. `cargo fmt --all && cargo clippy --all-features --all-targets -- -D warnings && cargo check --workspace --all-targets && cargo run --bin gui-coverage -- --threshold 95 && cargo test --all-features --test parity_cli group_set_role_help_lists_only_assignable_roles && cargo test --all-features --test proptest_groups last_admin_rest_generated_zero_admin_attempts_hit_precheck_and_seal_chokepoint && cargo test --all-features --test api_manifest manifest_matches_registry`
   - Result: passed. GUI coverage: 115/117 counted endpoints, 98.3%, threshold 95.0%.
3. Current branch focused reproduction: `cargo test --all-features --test named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected`
   - Result: failed only at `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-28774 did not become healthy within 90s`.
4. Baseline reproduction in `/var/folders/f_/j942sskj6nx67b6gk3rqgsqm0000gn/T/opencode/x0x-base-b16c34c-slice7` after confirming head `b16c34c62e98933716fda1b1434d8961f6ec168d`: `cargo test --all-features --test named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected`
   - Result: same failure signature at `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-32172 did not become healthy within 90s`.
5. `git status --short`
   - Result: clean build worktree after verification commands.

## Caveats

- I did not rerun full workspace nextest. I did rerun the cited failing test on current head and the dispatch baseline; both fail with the same pre-assertion daemon-health timeout, supporting the sweep record's baseline-diff claim.
- CI mirror PR status was not checked in this verifier pass; local evidence is strong but CI remains the formal green of record if the branch is pushed.

## Status

passed — Slice 7 achieved its goal. No gaps found.
