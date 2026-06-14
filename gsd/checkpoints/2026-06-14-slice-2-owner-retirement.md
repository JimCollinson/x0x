# Checkpoint — Slice 2: owner retirement / flatten role model (ADR-0016 Phase 1)

- Date: 2026-06-14
- Slice: 2 of 7 (`gsd/plan/phase-1-plan.md`, dispatch note `gsd/packets/2026-06-14-slice-2-owner-retirement.md`)
- Setup prerequisite: completed first via `gsd/packets/2026-06-14-x0x-gsd-perproject-setup.md`
- Feature branch: `feat/adr-0016-phase-1-authority-alignment`
- Base entering slice: Slice 1 commit `903cf8d`
- Slice commits under verification:
  - `4da985e` — `feat(adr-0016-phase-1): retire owner-only group authority`
  - `6db3143` — `test(adr-0016): update last-admin REST fixture for admin genesis`
  - `b9f6b37` — `test(adr-0016-phase-1): cover legacy owner chain replay`
- CI arbiter / green of record: draft mirror PR #5, <https://github.com/JimCollinson/x0x/pull/5>, head `b9f6b37`
- Status: **Slice complete, including finish-off legacy Owner roster-stability and current-code replay coverage; CI green of record passed after rerunning a named-group mesh/event-propagation failure. Ready for Jim review / Slice 3 dispatch.**

## Evidence item 1: receive-path creator-comparison grep

Slice 2 required deleting the `PolicyUpdated` receive-path creator gate and recording sibling literal creator comparisons. Grep after implementation found these remaining production comparisons:

- `authorized_treekem_membership_event_for_queue`:
  - `MemberAdded`: `sender_hex == creator_hex && actor == sender_hex` — membership add queue gate, Slice 3 scope.
  - `MemberRemoved`: creator remove or self-leave gate — Slice 3 / Slice 5 scope.
- `MemberAdded` apply arm: `sender_hex != creator_hex || actor != sender_hex` — membership add authority, Slice 3 scope.
- `MemberRemoved` apply arm: `creator_auth` / `self_leave` path — Slice 3 / Slice 5 scope.
- `GroupCardPublished`: `sender_hex != creator_hex` — card publication / provenance path, not role-authority change for Slice 2.
- Join-result handling: `sender_hex != creator_hex` — routing/provenance path, not role-authority change for Slice 2.
- Tests contain creator comparisons in fixtures only.

No additional authority mechanism beyond the spec's two known classes (literal creator checks plus role layer) was found. No stop condition fired.

## What changed

- `src/groups/state_commit.rs`
  - Removed `ActionKind::OwnerOnly`.
  - `AdminOrHigher` is now the minimum role for former owner-only state commits.
  - Comments updated to the flat Admin model; legacy `Owner` still satisfies via `GroupRole::at_least(Admin)`.
- `src/groups/member.rs`
  - Added `GroupMember::new_admin` for new genesis seeds.
  - Added `GroupRole::assignable_from_name` for assignment-only parsing.
  - Preserved `GroupRole::from_name`, serde names, ranking, and `as_u8` values for legacy deserialization/hash stability.
  - Exact R5 strings are exposed through testable library code:
    - `'owner' is a legacy role and cannot be assigned; valid roles: 'admin', 'member'`
    - `role '<name>' is reserved and cannot be assigned; valid roles: 'admin', 'member'`
- `src/groups/mod.rs`
  - New group genesis seeds creator as active `Admin`, never `Owner`.
  - v1 migration seeds creator as active `Admin`.
- `src/bin/x0xd.rs`
  - `require_owner` removed; policy update and terminal group withdrawal use `require_admin_or_above`.
  - `PolicyUpdated` receive-path creator gate removed; apply relies on signed commit + `AdminOrHigher`.
  - `GroupDeleted` apply relies on signed commit + `AdminOrHigher`; `actor` must match `commit.committed_by`.
  - REST `update_member_role` no longer uses the two-tier role matrix; any active Admin-or-higher can change any active member's role, subject to the Slice 1 last-admin invariant.
  - Role assignment rejects non-assignable roles via `GroupRole::assignable_from_name`.
  - Production genesis/card-import sites switched to `new_admin`.
- Tests
  - Added `tests/owner_retirement.rs` normal-gate coverage for Admin genesis, R5 exact strings, promoted-admin policy/role/end-group via apply path, legacy `Owner` compatibility, and owner→admin normalization.
  - Added finish-off legacy `Owner` roster-stability/current-code replay coverage that hard-codes the legacy JSON BLAKE3 and roster root stability values, then authors and replays Owner-containing commits with current code. This does **not** claim a genuine pre-Slice-2 historical commit fixture replay.
  - Updated state-commit/last-admin fixtures for Admin genesis while keeping explicit legacy-Owner fixtures where history preservation is under test.
  - Updated the Slice 1 ignored REST maintainer-gate fixture to expect the rejected demote to leave the creator as `admin` after Slice 2's genesis change.

## Scope / drift check

- Upstream/fork main at dispatch: `189b89c`; no material upstream drift.
- Slice 2 code sites were present with only Slice 1 line drift.
- No serde role names changed.
- No role byte / `as_u8` values changed.
- No hashing/signing/commit-format/state-hash semantics changed.
- No `.gsd` files were added to the feature branch.
- No hook, gate, CI workflow, test harness, daemon wrapper, build invocation, or environment setup was changed during Slice 2.
- Finish-off commit `b9f6b37` was test-only and did not modify production serialization, role-byte values, hashing, signing, commit format, harness/build/env, or `.gsd` files.

## Setup prerequisite evidence

One-time per-project setup completed before Slice 2:

- Planning commit: `77fe562` — `chore(gsd): add per-project gate and ci arbiter`
- Files committed to planning branch only:
  - `gsd/gate.sh`
  - `gsd/ci-arbiter.md`
- Canonical/deployed gate checksum: `f458b20e602049addd9889ce524fac08ad0fb6021025326624f37f5fea934534`
- Deployed local gate: feature worktree `.gsd/gate.sh`, locally excluded through `.git/info/exclude` (`.gsd/`).
- Hook installed at `/Users/jimcollinson/code/x0x/.git/hooks/pre-push`.
- Gate contents:

```sh
GSD_GATE_BRANCHES="feat/* exp/*"
GSD_GATE_COMMANDS=(
  "cargo fmt --all -- --check"
  "cargo clippy --all-targets --all-features -- -D warnings"
)
```

The hook ran on both Slice 2 pushes and passed before pushing to Jim's fork. CI remained the green of record.

## Verification evidence

### Local mandatory Rust checks

Run in feature worktree after implementation, again after the follow-up test fixture change where applicable, and again after finish-off test commit `b9f6b37`:

| Command | Result |
|---|---|
| `cargo fmt --all` | PASS |
| `cargo clippy --all-features --all-targets -- -D warnings` | PASS |
| `cargo check --workspace --all-targets` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS |

### Local Slice 2 tests

| Command | Result |
|---|---|
| `cargo nextest run --all-features -E 'test(owner_retirement) or test(role_assignment) or test(role_from_name) or test(new_admin) or test(test_group_info_new_seeds_admin) or test(test_migrate_from_v1) or test(test_caller_role)'` | PASS — 14/14 |
| `cargo nextest run --all-features -E 'test(role) or test(owner) or test(genesis)'` | FAIL locally only: 32 passed, 1 failed, 2 not run; failure was pre-existing macOS mesh setup failure `named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected` (`zero peers after 30s`) already baseline-reproduced in `gsd/evidence/2026-06-13-slice-1-local-gate.md`. |
| `cargo nextest run --all-features --workspace` | FAIL locally only: first failure `comprehensive_integration::test_agent_creation_performance` at 168 ms vs `<100ms`; isolated re-run passed. This exact timing-flake class was baseline-classified in Slice 1 evidence. |
| `cargo nextest run --all-features -E 'test(test_agent_creation_performance)'` | PASS — 1/1 |
| `cargo nextest run --all-features --workspace --no-fail-fast` | FAIL locally only: 1726 passed, 5 failed, 161 skipped; all 5 failures were the known `named_group_join_metadata_event` pair-mesh setup failures (`zero peers after 30s`) already baseline-reproduced at clean `189b89c`. |

### Maintainer-gate / CI-specific ignored suite

First CI run at `4da985e` failed `Multi-Agent Integration` because the Slice 1 ignored REST fixture still expected the creator's role to remain `owner` after a rejected last-admin demotion. Slice 2 correctly changes genesis to `admin`, so the roster was unchanged but the fixture expectation was stale. Fixed in separate commit `6db3143` (no amend after CI rejection).

Follow-up local evidence:

| Command | Result |
|---|---|
| `cargo nextest run --all-features --test named_group_integration -E 'test(last_admin_rest_self_demote_returns_409_exact_string)' --run-ignored all` | PASS — 1/1 |
| `cargo nextest run --all-features --test named_group_integration --run-ignored ignored-only` | Local FAIL after 4 passes on `named_group_creator_delete_propagates_to_peer` with the same known macOS pair-mesh setup failure (`zero peers after 30s`). CI Linux run passed after the fixture fix. |

### Finish-off legacy Owner roster-stability / current-code replay evidence

Finish-off commit `b9f6b37` added only `tests/owner_retirement.rs` coverage. Local evidence in feature worktree:

| Command | Result |
|---|---|
| `cargo nextest run --all-features --test owner_retirement -E 'test(owner_retirement_legacy_owner_chain_replays_byte_for_byte)'` | PASS — 1/1. Name predates the retro calibration; the test proves fixed legacy Owner roster serialization/root stability plus current-code replay over Owner-containing rosters, not a checked-in pre-Slice-2 historical commit fixture. |
| `cargo nextest run --all-features -E 'test(owner_retirement)'` | PASS — 8/8 |

### CI arbiter / green of record

Draft mirror PR #5, <https://github.com/JimCollinson/x0x/pull/5>, head `b9f6b37`:

- Build: all platform build jobs SUCCESS (`linux-x64-gnu`, `linux-x64-musl`, `linux-arm64-gnu`, `macos-x64`, `macos-arm64`, `windows-x64`).
- CI: Format Check SUCCESS; Clippy Lint SUCCESS; Test Suite SUCCESS; Coverage Gate SUCCESS; Documentation SUCCESS; API + GUI Parity Gate SUCCESS.
- Integration & Soak Tests: API Coverage Guard SUCCESS; Property Tests SUCCESS; Multi-Agent Integration SUCCESS; Soak Test SKIPPED by workflow.
- Security Audit: Cargo Audit SUCCESS; Panic Scanner SUCCESS.
- Initial Test Suite attempt at `b9f6b37` failed before completion in `named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected` (`alice never reported rejecting forged admin MemberJoined`). The new finish-off test had already passed in that run. The failed CI job was rerun without code, harness, environment, or workflow changes; rerun Test Suite job `81287790733` passed in 8m33s.

CI is the green of record; local full-suite failures were not used to claim readiness and were tied to baseline evidence.

## Honesty rules check

- No-harness-modification: PASS. No test harness, wrapper, daemon invocation, build, gate, hook, or CI workflow was changed during Slice 2.
- Baseline-diff for evidence: PASS. Local failures used as caveats are the exact macOS timing/mesh classes baseline-reproduced in `gsd/evidence/2026-06-13-slice-1-local-gate.md`. The one CI failure at `b9f6b37` was not dismissed as proof of readiness; the branch was not considered ready until PR #5's rerun CI was green. The failed attempt is recorded as a failed attempt, and the new finish-off test passed in that failed run.
- Evidence reproducible-from-branch: PASS. Readiness evidence is from committed feature branch `b9f6b37` plus CI PR #5. Local `.gsd/gate.sh` is a clone-local tripwire only and is not needed for CI.
- Local vs CI consistency: expected difference for known macOS mesh/timing cases. CI Linux runner is green of record after rerun at `b9f6b37`.

## Deviations / notes

- `POST /groups/:id/state/withdraw` now requires Admin-or-higher, as an owner-only act converted by Slice 2. `DELETE /groups/:id` semantics were intentionally left unchanged for Slice 5.
- R5 exact-string checks are gate-runnable through `GroupRole::assignable_from_name` library/unit tests and normal integration tests, not through dead `x0xd` bin unit tests.
- The role assignment helper intentionally accepts exactly lowercase `admin` and `member`; other spellings are rejected as reserved/non-assignable vocabulary. `GroupRole::from_name` remains case-insensitive for stored legacy names.
- Existing comments and tests still mention Owner where testing legacy behavior or out-of-scope KV ownership semantics; no `ActionKind::OwnerOnly` or `require_owner` remains.

## Files changed on feature branch

- `src/bin/x0xd.rs`
- `src/groups/member.rs`
- `src/groups/mod.rs`
- `src/groups/state_commit.rs`
- `tests/last_admin_invariant.rs`
- `tests/named_group_state_commit.rs`
- `tests/owner_retirement.rs`
- `tests/named_group_integration.rs` (follow-up fixture fix)

## Recommended next step

Review Slice 2 diff and this checkpoint. If accepted, dispatch Slice 3 — membership authority: add/remove/ban (`gsd/plan/phase-1-plan.md` Slice 3). Carry forward these notes:

- Remaining literal creator gates in add/remove paths are expected Slice 3 scope.
- Ban paths still have owner-target protections, expected Slice 3 scope.
- Keep the Slice 1 last-admin pre-check ordering before mutation when removing guards.
- Continue using PR #5 as CI green of record and do not modify the gate mid-slice.
