# ADR-0016 Phase 1 Slice 6 — remediation verification b16c34c

Date: 2026-06-22
Role: Verifier
Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
Head verified: `b16c34c62e98933716fda1b1434d8961f6ec168d`
Review range: `84b17c4662508a946aca739674941960faa52a5f..b16c34c62e98933716fda1b1434d8961f6ec168d`

## Goal checked

Slice 6 requires property-level proof that across generated commit/action sequences, on both delivery paths, no non-withdrawn state with zero active admins is reachable, with legacy `Owner` counted as admin.

## Sources inspected

- `gsd/plan/phase-1-plan.md` lines 177-194
- `gsd/spec/phase-1-authority-alignment.md` lines 88-95
- build worktree `AGENTS.md`, `CLAUDE.md`, `tests/CLAUDE.md`
- `tests/proptest_groups.rs`
- `src/groups/state_commit.rs`
- `src/groups/member.rs`
- `src/groups/mod.rs` last-admin helpers / commit choke-points

## Artifact verification

- Status: VERIFIED.
- Existence: `tests/proptest_groups.rs` exists and contains the Slice 6 property suite.
- Substantive: implementation is non-stub; added generated action/state machinery, independent oracle helpers, REST-like and gossip-apply executors, and dedicated properties.
- Wired: `cargo test --all-features --test proptest_groups` runs 10 tests; `cargo nextest run --all-features -E 'test(last_admin)'` discovers/runs 33 matching tests including the new property tests.

## Goal-backward checks

- Generator/action enum covers add, remove, ban, set-role, self-leave, policy update, and withdrawal.
- Role generation covers Admin/Member plus reserved lower Moderator/Guest changes; initial rosters include active legacy Owner/Admin at the boundary plus Admin/Member/Owner entries across Active/Pending/Removed/Banned states.
- Legacy Owner counts as admin in the independent oracle and in dedicated owner boundary properties.
- REST-like path uses user-facing pre-check helpers for last-admin remove/ban/demote/self-leave attempts and then verifies the authoritative `seal_commit` choke-point rejects the same zero-admin mutation.
- Gossip path crafts signed commits, validates through `validate_apply`, mutates only a clone, and relies on `finalize_applied_commit` as the apply-side choke-point.
- Accepted actions/sequences assert `withdrawn || independent_has_active_admin(...)` after every step on both paths.
- Rejected actions assert the original `GroupInfo` JSON snapshot is unchanged.
- Withdrawal from sole-admin states is tested for both Admin and legacy Owner sole-admin cases on REST-like and gossip paths.
- Oracle independence verified: the test oracle uses explicit `matches!` on `GroupMemberState::Active` and `GroupRole::{Admin, Owner}`; no production `GroupMember::is_active()` or `GroupRole::at_least()` call appears in `tests/proptest_groups.rs`.
- Scope respected: remediation diff changes only `tests/proptest_groups.rs`; no gate, CI, harness, production, or build mechanism changes found.

## Case count / seed policy

- Generated sequence case count is `LAST_ADMIN_SEQUENCE_CASES = 128`.
- The sequence property block uses `proptest::test_runner::Config { cases: LAST_ADMIN_SEQUENCE_CASES, ..Default::default() }`.
- Seed policy is proptest default/random seed behavior; no fixed seed is set. Reproduction remains via proptest's normal failure persistence/replay output if a counterexample is found.

## Commands run by verifier

All commands were run in `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build` at `b16c34c62e98933716fda1b1434d8961f6ec168d`.

1. `git status --short && git rev-parse HEAD && git diff --stat 84b17c4662508a946aca739674941960faa52a5f..b16c34c62e98933716fda1b1434d8961f6ec168d && git diff --name-only 84b17c4662508a946aca739674941960faa52a5f..b16c34c62e98933716fda1b1434d8961f6ec168d`
   - Result: clean worktree; head `b16c34c62e98933716fda1b1434d8961f6ec168d`; changed file only `tests/proptest_groups.rs`.
2. `cargo fmt --all && cargo clippy --all-features --all-targets -- -D warnings && cargo check --workspace --all-targets`
   - Result: passed.
3. `cargo nextest run --all-features -E 'test(last_admin)' && cargo test --all-features --test proptest_groups`
   - Result: nextest `33 passed`; proptest integration test `10 passed`.
4. `git status --short`
   - Result: clean worktree after verifier commands.

## Caveats

- I did not run the full workspace nextest suite; Slice 6 required the targeted `test(last_admin)` nextest command plus mandatory Rust gates, which passed.
- CI green of record was checked after this verification by the orchestrator; PR #5 raw red was accepted under the internal startup-timeout carve-out below.

## Post-verification CI and review evidence

- Pushed head: `b16c34c62e98933716fda1b1434d8961f6ec168d` to Jim fork branch `feat/adr-0016-phase-1-authority-alignment`.
- Local pre-push tripwire: passed (`cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`).
- PR #5 CI arbiter: raw red, internally accepted under `gsd/ci-arbiter.md` daemon-startup timeout carve-out; do not describe as raw green.
  - Build run `27958185346`: all build jobs passed.
  - Security Audit run `27958185321`: Cargo Audit and Panic Scanner passed.
  - CI run `27958185435`: all jobs passed except Test Suite job `82732110386`, which failed only at `tests/harness/src/cluster.rs:68:17` with `x0xd pair-alice-43939 did not become healthy within 90s` in `forged_member_joined_admin_role_or_secret_is_rejected`.
  - Integration & Soak run `27958185266`: API Coverage Guard and Property Tests passed; Soak skipped; Multi-Agent Integration job `82732110231` failed only at `tests/harness/src/cluster.rs:68:17` with `x0xd pair-alice-19527 did not become healthy within 90s` in `named_group_admin_disband_propagates_to_peer_after_creator_delete_409`.
  - Isolation count: two startup-health timeouts total (`<= 3`). Diff guard: Slice 6 changed only `tests/proptest_groups.rs`; no startup, health, networking, presence, harness, daemon, CI, gate, or build-invocation code changed.
- Code review after remediation: passed.
- Goal verification: passed.
- Adversarial review: READY-WITH-NITS. No CRITICAL/HIGH blockers. Carried LOW/NITs: the generated REST zero-admin property proves choke-point rejection and no mutation, but its name overstates explicit pre-check proof; the generated-sequence accepted-count guard is seeded by a valid initial policy update, with deterministic accepted-action coverage compensating.
- Craft Review: pass; no CONFORMANCE/SIMPLICITY/NIT findings.

## Status

passed — Slice 6 remediation satisfies the stated goal and acceptance criterion 2, with local verification, PR #5 CI classification, and required reviews recorded above.
