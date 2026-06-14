# Verification — Slice 3R retro remediation (ADR-0016 Phase 1)

- Date: 2026-06-14
- Role: Verifier
- Feature branch/head verified: `feat/adr-0016-phase-1-authority-alignment` @ `779835028dae3324a20534f07f0402c47e6d6fe8`
- Slice delta: `6ebac93e423e3fab60f91481adad6a86fb212445..779835028dae3324a20534f07f0402c47e6d6fe8`
- Status: **passed functionally; process note below**

## Goal-backward result

Functional Slice 3R remediation goals are verified:

1. Non-creator last-admin non-TreeKEM self-leave is prechecked before mutation, returns the exact self-leave 409 string, and commits through clone-first authoring.
2. `leave_treekem_group` was audited: it already clones the stored `GroupInfo` before self-leave mutation and only removes local state after `seal_commit` succeeds; no scope-expanding TreeKEM change was needed.
3. R2 follows Jim's binding decision: apply rejects only `Owner` and accepts `Moderator`/`Guest`; authoring is gated to `admin`/`member`; the REST role-assignment handler is the only production author of `MemberRoleUpdated` found in the tree.
4. Scope stayed surgical: slice delta changed only `src/bin/x0xd.rs`, `src/groups/mod.rs`, `tests/last_admin_invariant.rs`, and `tests/membership_authority.rs`.
5. Local mandatory checks, focused tests, and PR #5 CI all passed. The broad-filter local failure is the already-baseline-classified macOS mesh setup failure.

Process note: the packet/plan asked for `gsd/checkpoints/2026-06-14-slice-3r-retro-remediation.md`, but that exact remediation checkpoint file was not present in the planning worktree during verification.

## Evidence anchors

### R1 — self-leave corruption fixed

- `src/bin/x0xd.rs:11863-11871` checks `last_admin_self_leave_precheck_error` before any mutation and returns `StatusCode::CONFLICT`.
- `src/bin/x0xd.rs:11872-11885` mutates a cloned `next`, seals, and assigns `*info = next` only after `seal_commit` succeeds.
- `src/groups/mod.rs:228-231` defines the exact self-leave string: `a group must always have at least one admin; make another member an admin before leaving`.
- `src/groups/mod.rs:253-270` evaluates self-leave on a clone.
- `tests/membership_authority.rs:399-417` proves the non-creator last-admin self-leave helper returns the exact conflict and leaves roster revision/state hash unchanged.

### R1 creator-delete preserved / TreeKEM disposition

- `src/bin/x0xd.rs:11846-11863` keeps the creator path on `seal_withdrawal` and `GroupDeleted`.
- `src/bin/x0xd.rs:11425-11441` clones TreeKEM state before the leave mutation.
- `src/bin/x0xd.rs:11459-11470` mutates/seals the clone.
- `src/bin/x0xd.rs:11472-11485` removes local TreeKEM state only after successful sealing.

### R2 — Jim binding implemented

- `src/bin/x0xd.rs:8669-8671` leaves Owner-on-apply rejection unchanged.
- `src/bin/x0xd.rs:8672-8680` applies all non-Owner target roles through the signed commit path, so Moderator/Guest are not newly rejected on apply.
- `src/bin/x0xd.rs:12183-12186` gates REST role authoring through `GroupRole::assignable_from_name`.
- `src/groups/member.rs:85-99` accepts only `admin` and `member`; rejects `owner`, `moderator`, and `guest` for assignments while parsing remains broader via `from_name`.
- `src/bin/x0xd.rs:12245-12252` is the production `MemberRoleUpdated` authoring site; grep found no other production constructor.
- `tests/membership_authority.rs:434-440` covers signed role-update apply for `Admin`, `Member`, `Moderator`, and `Guest`.
- `tests/owner_retirement.rs:190-212` covers assignment rejection for `owner`, `moderator`, and `guest`.
- `tests/last_admin_invariant.rs:229-254` covers last-admin invariant rejection for sole-admin demotion to reserved below-admin roles.

### Scope / validation

- Slice diff names: `src/bin/x0xd.rs`, `src/groups/mod.rs`, `tests/last_admin_invariant.rs`, `tests/membership_authority.rs`; no `.gsd/gate.sh`, CI workflow, test harness, daemon wrapper, build invocation, or environment setup changes.
- Verifier local checks passed: `cargo fmt --all -- --check`; `cargo clippy --all-features --all-targets -- -D warnings`; `cargo check --workspace --all-targets`.
- Verifier focused tests passed: `cargo nextest run --all-features --test membership_authority` (14/14); `cargo nextest run --all-features --test owner_retirement -E 'test(owner_retirement_role_assignment_accepts_only_admin_member_and_exact_errors)'` (1/1); `cargo nextest run --all-features --test last_admin_invariant` (12/12).
- Broad-filter local command `cargo nextest run --all-features -E 'test(last_admin) or test(role) or test(member)'` failed only in `named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected` with `[cluster] FATAL: ... zero peers after 30s`; baseline classification exists in `gsd/evidence/2026-06-13-slice-1-local-gate.md:25-31` and `:38-51`.
- PR #5 checks at head `779835028dae3324a20534f07f0402c47e6d6fe8` passed: Format Check, Clippy Lint, Test Suite, Coverage Gate, API/API-GUI, Property Tests, Multi-Agent Integration, audit, docs, and all build matrix jobs; Soak Test skipped by workflow.

## Recommendation

Proceed from a functional verification standpoint. If strict packet bookkeeping is required before Slice 4, file the missing Slice 3R remediation checkpoint or treat this verification note as the disposition record.
