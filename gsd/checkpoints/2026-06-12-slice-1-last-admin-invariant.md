# Checkpoint — Slice 1: last-admin invariant (ADR-0016 Phase 1)

- Date: 2026-06-12
- Slice: 1 of 7 (`gsd/plan/phase-1-plan.md`, packet `gsd/packets/2026-06-12-slice-1-last-admin-invariant.md`, addendum 1 bound)
- Branch: `feat/adr-0016-phase-1-authority-alignment` (cut at upstream `189b89c`; upstream main verified == `189b89c` at session time — no rebase needed, zero line drift)
- Implementer: gsd-build agent (Claude Code local dispatch). Orchestrator independently re-ran all gates and reviewed the full diff before this checkpoint.
- Status: **Slice complete; all gates green on this machine; awaiting Jim's review before Slice 2.**

## Evidence item 1 (binding, addendum 1): authority/apply-path map

Every REST handler and gossip-apply arm that mutates group membership, roles, policy, or lifecycle was enumerated at `189b89c`. Conclusion: **state-mutating group commits exist on exactly two production seams, both now guarded by the one shared helper** `enforce_last_admin_invariant` (`src/groups/state_commit.rs`):

- **Authoring seam** — `GroupInfo::seal_commit` (`src/groups/mod.rs`, first statement, before any chain-field mutation). `GroupStateCommit::sign` has exactly one production caller (`seal_commit`, mod.rs:449), so no authoring path can bypass it. `seal_withdrawal` sets `withdrawn = true` before delegating → group-ending commits exempt (the ADR's exit valve).
- **Apply seam** — `GroupInfo::finalize_applied_commit` (`src/groups/mod.rs`, immediately after the state-hash equality check, so the roster validated is provably the roster the signed commit's `roster_root` committed to). Reached by every commit-applying path: the daemon gossip pipeline `apply_stateful_event_to_group` (x0xd.rs:7365: `validate_apply` → mutate-on-clone → `finalize_applied_commit` at 7384) and the library `GroupInfo::apply_commit` (mod.rs:500).

Gossip-apply arms (x0xd.rs), all delivered through `apply_stateful_event_to_group`: MemberAdded (match 8211 / apply 8244), MemberRemoved incl. self-leave (8429/8470), GroupDeleted (8560/8584; withdrawn → exempt), PolicyUpdated (8615/8630; roster unchanged), MemberRoleUpdated (8657/8700), MemberBanned (8713/8747), MemberUnbanned (8823/8840), JoinRequestCreated (8866/8902; pending entries are roster-root-neutral), JoinRequestApproved (8954/9003), JoinRequestRejected (9141/9163) and JoinRequestCancelled (9183/9202) (join_requests only), GroupMetadataUpdated (9239/9257; roster unchanged). Non-commit arms: GroupCardPublished (card cache only), SecureShareDelivered (secret/epoch material only), MemberJoined (third parties deliberately defer to the inviter's signed MemberAdded, comment x0xd.rs:9490–9495; inviter-side it authors via seal at 9592/9629 → covered by the authoring seam).

REST handlers all author via `seal_commit`/`seal_withdrawal` → covered by the apply-path seam; §3 pre-checks added where an act can zero the admin set: remove_named_group_member (~11353), remove_treekem_named_group_member (~11594), update_member_role (~12307), ban_group_member (~12379), ban_treekem_group_member (~12534). Handlers that cannot zero the admin set (add, unban→Member, join-request lifecycle, metadata/policy, reseal) verified and left untouched. create_named_group genesis still seeds Owner (Slice 2 changes this, not ours).

Non-commit mutation paths noted for completeness (trust-anchor/seeding, not commit application): invite/card-import seeding (x0xd.rs 678, 13590, 13631), `drop_local_named_group_state` (local teardown), `migrate_from_v1` (seeds creator as Owner).

**No path applies commits without passing the choke-point. No stop condition fired.**

## What changed

- `src/groups/state_commit.rs`: new shared `pub fn enforce_last_admin_invariant(&BTreeMap<String, GroupMember>, withdrawn) -> Result<(), ApplyError>` — withdrawn exempt; counts active members with `role.at_least(GroupRole::Admin)` (legacy Owner rank 4 > Admin rank 3 → alias automatic); rejects with existing `ApplyError::Invariant`. 7 choke-point unit tests (`last_admin_*`).
- `src/groups/mod.rs`: helper re-exported; called at top of `seal_commit` and in `finalize_applied_commit` after the hash check.
- `src/bin/x0xd.rs`: `LAST_ADMIN_PRECHECK_ERROR` const (§3 string verbatim); `last_admin_precheck` helper (clones the group, applies the intended mutation, runs the shared check → 409); five call sites placed after all existing guards and immediately before mutation; 3 unit tests asserting exact string + status + pass/exempt cases.
- `tests/last_admin_invariant.rs` (new): 11 integration tests — authoring-seam rejections (demote/remove/ban sole admin), withdrawal exemption, owner→admin normalization pass, legacy-Owner-counts mixed roster, and apply-side via the daemon's exact sequence: adversarially crafted (correctly signed, chain-valid) zero-admin commit rejected at the choke-point itself; sole-Owner→member commit rejected; zero-admin withdrawal accepted; proposed-roster-hashes-to-`roster_root` seam test; behavior-neutral member-removal convergence.
- `tests/named_group_integration.rs`: one new `#[ignore]`d daemon-backed REST test (`last_admin_rest_self_demote_returns_409_exact_string`) for the maintainer-side gate.

**Design choice (packet latitude):** adjacent mandatory check via one shared helper at the two library seams, NOT a new `validate_apply` argument. At both `validate_apply` call sites the post-mutation roster does not yet exist when it runs (mutation follows validation), so a new argument would have forced mutate-before-validate reordering — a forbidden pipeline perturbation. `finalize_applied_commit` is invoked immediately after `validate_apply` on both apply paths and cannot be skipped; `seal_commit` is the single authoring funnel — identical choke-point semantics on all paths.

**Behavior-neutrality (verified analytically and by the suite):** today every live group's creator is an active Owner (the un-deleted "cannot remove creator"/"cannot ban owner" guards keep it so), therefore the invariant is unreachable through every currently-live flow except sole-owner self-demotion via `update_member_role` — which now correctly returns the §3 409 from the pre-check before any mutation (R2's mandated consequence: the last admin cannot self-demote).

## Evidence item 2: verification (orchestrator-run, this machine)

Commit under verification: `903cf8d` on `feat/adr-0016-phase-1-authority-alignment` (parent `189b89c`), exactly 5 files / 646 insertions / 2 deletions.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all-targets --all-features -- -D warnings` | PASS — `Finished dev profile`, zero warnings |
| `cargo nextest run --all-features --workspace --no-fail-fast` | **1716 passed, 5 failed (pre-existing environmental, see below), 161 skipped** (the `#[ignore]`d daemon-API/e2e suites — Jim's gate, per plan) |
| `cargo nextest run --all-features -E 'test(last_admin)'` | **18/18 PASS** (7 choke-point unit + 11 integration; full list below) |

`last_admin` suite (all PASS): unit — demote/remove/ban sole admin rejected, withdrawal exempt, owner self-demote-to-admin accepted, owner demote-to-member rejected, legacy owner counts as admin; integration — seal rejects demote/remove/ban of sole admin, seal allows withdrawal + owner normalization, seal counts legacy owner in mixed roster, gossip-apply rejects crafted zero-admin commit + owner-demoted-to-member, gossip-apply allows zero-admin withdrawal + member removal with admin remaining, proposed-roster-hashes-to-`roster_root`.

**Pre-existing failure, triaged and exonerated:** `named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected` fails on this machine at the harness level — `[cluster] FATAL: pair-alice has zero peers after 30s — mesh is disconnected` (tests/harness/src/cluster.rs:430), i.e. the two-daemon mDNS mesh never forms; the test dies in setup before any group logic runs. Verified to fail **identically at clean upstream `189b89c` with the slice diff absent** (detached-HEAD run, same FATAL). The file has prior "CI flake hardening" commits (`96aa28d` widened its waits 15s→30s) and upstream's `.config/nextest.toml` already serializes this binary into a `quic-localhost` max-threads=1 group citing ant-quic dual-stack loopback issues on macOS — a known-fragile family. All 5 fail even fully serialized and in isolation on this machine (whole-file baseline run at detached `189b89c`: 1 passed / 5 failed — byte-identical to the run with the slice). Environmental, not a regression; re-run on the maintainer machine at the local gate.

## Drift vs the spec's pinned citations

Zero. Worktree at `189b89c` == current upstream main; every Slice-1-relevant citation re-verified byte-identical (validate_apply at state_commit.rs:521; ApplyError::Invariant 444–446; at_least/ranks member.rs:19–49; the creator gates, owner-target guards, require_* helpers, role matrix, new_owner sites — all confirmed for later slices' benefit).

## Deviations (recorded, not improvised)

1. **Error body shape:** repo precedent (`api_error`) emits `{"ok": false, "error": "..."}`. The §3 string is carried verbatim in `"error"`; the body has the additional `"ok": false` field. Status 409 (`StatusCode::CONFLICT`) matches both §3 and repo precedent.
2. **The §3 exact-string REST assertion is not executable inside slice gates — by repo design.** Two compounding upstream conventions: (a) `Cargo.toml` declares `test = false` for the `x0xd` bin, so its `#[cfg(test)] mod tests` — including our 3 `last_admin_precheck_*` unit tests AND the pre-existing upstream tests already in that mod — is never compiled or run by any gate (pre-existing wart, flagged for upstream triage); (b) `tests/named_group_integration.rs` is contractually "all tests `#[ignore]` — require a running x0xd daemon", i.e. part of the maintainer-side gate the plan forbids slices from running. Net effect: the exact string + 409 + roster-untouched + normalization-passes contract is pinned at runtime ONLY by the new `#[ignore]`d test `last_admin_rest_self_demote_returns_409_exact_string` — **please exercise it at the maintainer gate** (`cargo nextest run --all-features -E 'test(last_admin)' --run-ignored all` with a built daemon). The 3 dead-by-config unit tests were kept to match the file's existing convention; Jim may prefer moving `LAST_ADMIN_PRECHECK_ERROR` into the library in a later slice to make the string assertable in normal gates — orchestrator's open question #1.
2. **Lockfile pin (environment, not the diff):** `Cargo.lock` is gitignored; fresh resolution picks `time 0.3.48`, which fails rustc 1.95.0's coherence check (E0119 via `rcgen`/`tracing-subscriber`). Pinned locally with `cargo update -p time --precise 0.3.47`. **Any freshly-resolving environment (incl. CI) will hit this until the ecosystem ships a fix — flagging for Jim.**

## Risks / notes for later slices

- **Slice 3 packet note:** on the TreeKEM ban path the new invariant 409 fires before the 424 missing-KeyPackage check (auth/invariant before dependency errors). Unreachable until Slice 3 deletes the owner guards; pin the desired precedence in that packet's tests.
- **Slices 3/5 packet note:** several handlers mutate `GroupInfo` in place and then seal (no rollback if seal fails). Unreachable-mutate-then-fail today (see behavior-neutrality), but once guards come down, last-admin rejections on un-pre-checked paths could leave local state mutated without a sealed commit. The §3 pre-checks added this slice run BEFORE mutation precisely for this reason; Slices 3 and 5 must keep that ordering on every path they touch (or clone-first like the TreeKEM remove path).
- `update_member_role` takes no group membership lock (pre-existing); the pre-check is UX, the choke-point is the net.
- Exposure to the maintainer-side gate: one new `#[ignore]`d daemon-backed test (compiles under clippy; not executed here); the pre-existing `#[ignore]`d suites untouched and not run, per the packet. Plus the environmental cluster-test failure above — re-run `named_group_join_metadata_event` on the maintainer machine.
- **Repo hygiene (upstream, out of scope, for triage):** daemon-backed test runs append denial events to the **tracked** root file `audit.jsonl`, dirtying the working tree on every local test run (it nearly leaked into this slice's commit; caught and restored to the `189b89c` byte state). Suggest upstream gitignore + fixture relocation.

## Session/process notes

- Mid-slice the machine ran out of disk (nextest build filled it); recovered by deleting two stale regenerable `target/` dirs (`/private/tmp/x0x-control-main/target`, `.claude/worktrees/great-murdock-75031d/target`). The implementer agent stopped cleanly per GSD instead of improvising; the orchestrator re-ran all gates afterwards.
- Safety: the local clone's `origin` remote points at upstream saorsa-labs/x0x — its **push URL has been disabled** (`git remote set-url --push origin DISABLED-…`) for the duration of this work; pushes go to the `fork` remote (JimCollinson/x0x) only. Revert with `git remote set-url --push origin https://github.com/saorsa-labs/x0x.git`.
- The main worktree (`/Users/jimcollinson/code/x0x`, branch `main`) holds pre-existing **staged uncommitted changes** (deleting ADR-0015 + ADR governance tooling, kv edits) that predate this session. Untouched; surfaced to Jim.

## Plan validity & recommended next step

Plan remains valid; no stop condition fired; the seam design matched the spec's expectation. **Recommended next:** Jim reviews this checkpoint and the slice diff; on approval, dispatch Slice 2 (owner retirement) with the two packet notes above carried forward.
