# Verification — ADR-0016 Phase 1 Slice 5 final remediation (`f1ecb9f`)

- Date: 2026-06-21
- Role: verifier
- Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
- Branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `f1ecb9f2d2719f4afbab72223cc7abe570db8570`
- Parent inspected: `1fa5f23bd833a4c71e231efc7f19f4cef63ff13e`
- Prior blocker source: `gsd/checkpoints/2026-06-21-slice-5-final-remediation-blocker-1fa5f23.md`
- Verdict: `passed`

## Goal-backward result

The final adversarial HIGH is closed for the inspected remediation commit. The TreeKEM durable install path now repairs a late same-stable withdrawal before the stale `named_groups.json` rename can occur, wipes TreeKEM snapshot/journal files, aborts the install, and the caller does not install in-memory TreeKEM state. The withdrawn-card import terminality path now participates in the per-group membership serialization lock used by TreeKEM metadata applies.

## Verified goals

1. **Late same-stable terminal withdrawal cannot leave non-withdrawn durable install state.**
   - `src/bin/x0xd.rs:18444-18451` performs the existing pre-check using the stable id copied from the candidate `GroupInfo`.
   - `src/bin/x0xd.rs:18452-18458` captures the candidate non-withdrawn JSON.
   - `src/bin/x0xd.rs:18481-18491` now runs `repair_withdrawn_named_groups_json_and_wipe_key_material(...)` after journal/snapshot writes and before `write_named_groups_json_atomic(...)`; if withdrawal is present, it bails instead of renaming the stale JSON.
   - `src/bin/x0xd.rs:11977-11996` implements repair by detecting same-stable withdrawn records from current in-memory named groups, removing TreeKEM persistence for the group id, and atomically writing the repaired withdrawn `named_groups.json`.

2. **Card-import withdrawal cannot race around TreeKEM install due missing per-group lock.**
   - `src/bin/x0xd.rs:14571-14573` now acquires `group_membership_lock(&state, &group_id)` before the withdrawn-card terminality branch.
   - `src/bin/x0xd.rs:8260-8275` normalizes lock keys through existing named-group records to the stable group id, so a card keyed by stable id and a TreeKEM apply keyed by MLS/storage id converge on the same mutex when the local group exists.
   - `src/bin/x0xd.rs:8398-8399`, `src/bin/x0xd.rs:8594-8601`, and `src/bin/x0xd.rs:9302-9309` show joined TreeKEM installs are reached while the metadata apply holds that per-group membership guard.

3. **Regression is disk-level and proves durable withdrawal/keyless/no-install outcomes.**
   - `src/bin/x0xd.rs:21833-21915` adds `treekem_atomic_persist_lost_race_withdrawn_repairs_named_groups`.
   - The test injects withdrawal after stale JSON capture via `force_atomic_persist_post_json_withdrawn_ids(...)` at `src/bin/x0xd.rs:21867` and the hook at `src/bin/x0xd.rs:18460-18466`.
   - It asserts rejected install/no in-memory TreeKEM state at `src/bin/x0xd.rs:21877-21883`.
   - It asserts snapshot and journal are removed at `src/bin/x0xd.rs:21885-21891`.
   - It reloads durable `named_groups.json` from disk and asserts withdrawn, keyless, and without stale roster advance at `src/bin/x0xd.rs:21893-21908`.

4. **No forbidden scope/mechanism or format changes found.**
   - Diff `1fa5f23..f1ecb9f` touches only `src/bin/x0xd.rs`.
   - No changes found to `.gsd/gate.sh`, CI workflows, test harness, daemon/service wrappers, build invocation, Cargo files, public wire structs, storage schema/version constants, or hash/state-commit algorithms.
   - Added test-only hooks are `#[cfg(test)]` and runtime changes are limited to terminality repair/locking in existing persistence/import paths.

## Checks run by verifier

```sh
cargo test --all-features --bin x0xd treekem_atomic_persist_lost_race_withdrawn_repairs_named_groups
```

Result: PASS, 1/1.

Known orchestrator evidence also reports mandatory Rust checks and focused suites passing.

## Gaps / risks

- No blocker found for the stated HIGH.
- Minor residual race class not blocking this goal: repair uses the current in-memory named-group map as the source of truth. That matches the targeted same-process race and card-import path under review; it is not a new storage or wire format concern.

## Recommended next gate

Proceed to final integrated GSD gates for PR readiness: clean-context/adversarial/craft rerun at `f1ecb9f`, then CI arbiter classification/green-of-record before marking Slice 5 done or asking Jim about upstream PR action.
