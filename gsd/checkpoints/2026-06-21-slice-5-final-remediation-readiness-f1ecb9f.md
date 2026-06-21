# Checkpoint — Slice 5 final remediation readiness at `f1ecb9f`

- Date: 2026-06-21
- Slice/question: ADR-0016 Phase 1 Slice 5 — final TreeKEM durable-install race remediation
- Prepared by: OpenCode orchestrator
- Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
- Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `f1ecb9f2d2719f4afbab72223cc7abe570db8570`
- Parent for final remediation: `1fa5f23bd833a4c71e231efc7f19f4cef63ff13e`
- PR / CI arbiter: PR #5, <https://github.com/JimCollinson/x0x/pull/5>
- Status: **Ready for Jim checkpoint with caveats** — prior adversarial HIGH closed; raw PR #5 CI remains red only under the documented startup-timeout carve-out; no PR/upstream action without Jim confirmation.
- Meaningful work-unit? Yes — non-trivial Rust group-authority/TreeKEM persistence behaviour in an upstream/shared repo.
- Review cadence: final clean-context + adversarial + craft completed at exact head `f1ecb9f`.
- Unreviewed backlog: none for this remediation checkpoint.

## What happened

- Remediated the `1fa5f23` adversarial HIGH with `f1ecb9f fix(adr-0016-phase-1): harden treekem durable install`.
- The remediation closes the late terminal-withdrawal race by repairing durable withdrawn/keyless `named_groups.json`, removing TreeKEM snapshot/journal persistence, and bailing before stale non-withdrawn `named_groups.json` can be renamed into place.
- Card-import terminality now takes the per-group membership lock before retaining a withdrawn shell.
- Added a disk-level regression for the post-JSON-capture/pre-rename lost race.
- Reran CI failed jobs. Multi-Agent Integration passed on rerun. Test Suite attempt 4 remains raw red only for the known daemon startup-timeout signature.
- Final clean-context, adversarial, and craft gates were rerun at `f1ecb9f`.

## Files changed / scope

Final remediation diff `1fa5f23..f1ecb9f`:

- `src/bin/x0xd.rs` only — `209 insertions(+), 7 deletions(-)`.

Key anchors:

- `src/bin/x0xd.rs:11977-11996` — `repair_withdrawn_named_groups_json_and_wipe_key_material(...)` preserves the withdrawn/keyless shell and wipes TreeKEM persistence.
- `src/bin/x0xd.rs:14571-14573` — group-card import takes `group_membership_lock(...)` before terminal withdrawn handling.
- `src/bin/x0xd.rs:18481-18494` — late terminality repair runs after snapshot/journal writes and before `named_groups.json` rename.
- `src/bin/x0xd.rs:21833-21915` — `treekem_atomic_persist_lost_race_withdrawn_repairs_named_groups` regression asserts rejected install, no in-memory install, no snapshot/journal, and durable withdrawn/keyless state.

No final-remediation changes found to `.gsd/gate.sh`, CI workflows, `tests/harness/**`, daemon wrappers, build invocation, Cargo files, public wire structs, hash/state-commit algorithms, or storage-format/version constants.

## Evidence

### Local checks

Previously recorded at `f1ecb9f`:

- `cargo fmt --all` — PASS.
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS.
- `cargo check --workspace --all-targets` — PASS.
- `cargo test --all-features --bin x0xd treekem_atomic_persist_lost_race_withdrawn_repairs_named_groups -- --nocapture` — PASS, 1/1.
- `cargo test --all-features --bin x0xd member_joined_treekem -- --nocapture` — PASS, 2/2.
- `cargo test --all-features --bin x0xd lost_race -- --nocapture` — PASS, 19/19.
- `cargo test --all-features --bin x0xd withdrawn -- --nocapture` — PASS, 16/16.
- `cargo test --all-features --bin x0xd treekem -- --nocapture` — PASS, 32/32.
- `git diff --check` — PASS.

Additional final-gate checks run by reviewers/orchestrator:

- `cargo fmt --all -- --check` — PASS.
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS.
- `cargo check --workspace --all-targets` — PASS.
- `cargo test --all-features --bin x0xd treekem_atomic_persist_lost_race_withdrawn_repairs_named_groups` — PASS, 1/1.
- `cargo test --all-features --bin x0xd treekem` — PASS, 32/32.
- `cargo test --all-features --bin x0xd withdrawn` — PASS, 16/16.
- `cargo test --all-features --bin x0xd lost_race` — PASS, 19/19.
- `cargo test --all-features --bin x0xd member_joined_treekem` — PASS, 2/2.
- `cargo nextest run --all-features -E 'test(leave) or test(disband) or test(withdraw)'` — PASS, 23/23.
- `cargo test --all-features --test peer_lifecycle_integration direct_send_with_require_ack_round_trips_to_live_peer -- --nocapture` — PASS, 1/1.
- `cargo test --all-features --test peer_lifecycle_integration direct_send_without_require_ack_omits_ack_block -- --nocapture` — PASS, 1/1.

### CI arbiter / green of record

Location: PR #5 checks, <https://github.com/JimCollinson/x0x/pull/5>.

Head OID: `f1ecb9f2d2719f4afbab72223cc7abe570db8570`.

Passing checks after reruns:

- Build run `27909785933`: Validate release metadata; Build linux-x64-gnu; Build linux-x64-musl; Build linux-arm64-gnu; Build macos-x64; Build macos-arm64; Build windows-x64 — PASS.
- Security Audit run `27909785922`: Cargo Audit; Panic Scanner — PASS.
- CI run `27909785930`, attempt 4: Format Check; Clippy Lint; Documentation; API + GUI Parity Gate; Coverage Gate — PASS.
- Integration & Soak Tests run `27909785917`, attempt 3: API Coverage Guard; Property Tests; Multi-Agent Integration job `82593738021` — PASS.
- Soak Test — skipped.

Remaining raw red:

- CI / Test Suite run `27909785930`, attempt 4, job `82594805927` — raw red, accepted only under `gsd/ci-arbiter.md` startup-timeout carve-out.
- Command: `cargo nextest run --all-features --workspace -E '!binary(x0x_0041_synthetic_kill_restart)'`.
- Summary: `1758/1765 tests run: 1757 passed, 1 failed, 164 skipped`; `7/1765` not run due fail-fast.
- Failing test: `x0x::named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected`.
- Failure site: `tests/harness/src/cluster.rs:68:17`.
- Verbatim signature: `x0xd pair-alice-59474 did not become healthy within 90s`.

Arbiter classification:

- Signature: PASS — the only current failure is a daemon-startup health timeout at harness bring-up.
- Isolation: PASS — one timed-out test, below the `<= 3` threshold.
- Diff guard: PASS for final remediation — `1fa5f23..f1ecb9f` touches only `src/bin/x0xd.rs`, not `tests/harness/**`, CI, startup/health/network/bootstrap/presence files, daemon wrappers, or build invocation.
- Baseline record: existing checkpoint evidence recorded the same focused startup-timeout signature on base `189b89c0aadb25a1458752fdec040d01df9d2d66` (`x0xd pair-alice-55461 did not become healthy within 90s`) and current `1fa5f23` (`x0xd pair-alice-62669 did not become healthy within 90s`). The final `f1ecb9f` delta is group persistence/import/test code and does not touch startup.

Earlier CI attempt 3 had non-startup direct-send failures:

- `direct_send_with_require_ack_round_trips_to_live_peer` failed with `IncompleteMessage` in attempt 2/3 history.
- `direct_send_without_require_ack_omits_ack_block` failed with `IncompleteMessage` in attempt 3.
- Both direct-send tests passed in CI attempt 4; both focused current local tests also passed. These earlier non-startup reds are not hidden and are **not** covered by the startup-timeout carve-out.

## Review findings

Clean-context test:

- Reviewer/tool: `cleancontext`.
- Result: **Concerns**, no blocker.
- Findings:
  - Final evidence/checkpoint artifacts are still untracked in the planning worktree, so the formal evidence is not fully branch-alone durable.
  - GitHub UI remains raw red; checkpoint must say “raw CI red, accepted under startup-timeout carve-out,” not “CI green.”
  - Final checkpoint must restate `f1ecb9f` CI run/job IDs, signature, and diff guard.

Adversarial review:

- Reviewer/tool: `adversarial`, `openai/gpt-5.5`.
- Result: **READY-WITH-NITS**.
- Independence note: same model/provider as orchestrator/adversarial available in this harness, so evidence is weaker than cross-model review.
- CRITICAL/HIGH findings: none.
- LOW findings:
  - CI is raw red and readiness depends on the internal startup-timeout carve-out; checkpoint must not overstate this as ordinary green.
  - Final planning evidence remains uncommitted/untracked in the planning worktree.
- Test-quality note: the added regression is honest for the prior blocker; it injects withdrawal after JSON capture and proves disk and memory outcomes. It uses a test hook rather than a real concurrent task interleaving, but targets the exact stale-JSON window.

Craft Review:

- Reviewer/tool: `craft`.
- Verdict: **Craft pass**.
- CONFORMANCE findings: none.
- SIMPLICITY findings: none.
- NIT findings: none.
- Notes: remediation is surgical, helper naming/error shape matches sibling persistence code, test-only hook mirrors existing patterns, and membership lock use matches neighboring mutation serialization.

## Honesty rules check

- No-harness-modification: PASS — final remediation touched only `src/bin/x0xd.rs`; no `tests/harness/**`, CI, `.gsd/gate.sh`, daemon wrapper, build invocation, environment setup, Cargo, wire/commit/hash/storage-format changes.
- Baseline-diff for evidence: PASS with caveat — current raw CI failure is classified only under the documented startup-timeout carve-out with base/current same-signature proof from prior checkpoint plus final diff guard. Earlier non-startup direct-send reds are recorded separately and cleared on rerun/focused local tests.
- Evidence reproducible-from-branch: CONCERN — code/tests/check commands reproduce from the build branch; final planning/review evidence is in planning-worktree artifacts and remains uncommitted unless Jim chooses to publish/commit it.
- Local vs CI consistency: PASS under the internal arbiter; do not call raw GitHub UI green.

## Drift / scope concerns

- No new scope creep found in the final remediation.
- Broader surface/docs/API language work remains Slice 7 scope.
- PR creation/upstream action remains Jim-gated.

## Open questions / decisions for Jim

- Whether/when to publish or commit the planning evidence artifacts. The final checkpoint exists in the planning worktree but is not committed by the orchestrator.
- Whether to proceed toward Slice 6/Slice 7 or pause for Jim review of Slice 5 evidence.
- No PR/upstream action has been taken; Jim confirmation is required for any PR action.

## Recommended next step

Treat Slice 5 final remediation as ready for Jim checkpoint with caveats:

1. Acknowledge PR #5 raw Test Suite red is accepted only under the startup-timeout carve-out, not ordinary CI green.
2. Publish/commit or otherwise preserve this final checkpoint/evidence before asking for a PR decision.
3. Continue to Slice 6 / Slice 7 only after Jim accepts this checkpoint, or stop here if Jim wants to review the full Slice 5 evidence first.

Do not open an upstream PR without Jim's explicit confirmation.
