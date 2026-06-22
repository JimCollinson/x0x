# GSD Checkpoint — ADR-0016 Phase 1 feature-complete at 1c3f17a

Date: 2026-06-22
Project: x0x
Slice/question: ADR-0016 Phase 1 — final Slice 7 and integrated phase readiness
Prepared by: orchestrator
Agents/tools used: `codereviewer`, `verifier`, `adversarial`, `craft`, `cleancontext`, local cargo/gh

## Status

Done for Phase 1 feature implementation / not PR-opened.

Meaningful work-unit? Yes — non-trivial shared/upstream Rust/docs/API/GUI work affecting named-group authority semantics.
Review cadence: per-unit + final integrated gauntlet completed; adversarial HIGH finding explicitly deferred by Jim; Craft CONFORMANCE findings explicitly justified/corrected in planning notes; clean-context returned Concerns but no blocker.
Unreviewed backlog if deferred: card-bound discovery/request-access authority seeding deferred to Phase 2 / maintainer follow-up; no Phase 1 fix.

Note: PR creation remains Jim's explicit gate. This checkpoint does not open or approve an upstream PR.

## What happened

- Slice 7 implementation was already committed and pushed to Jim's fork at build head `1c3f17a58c94c04f4099014bfad121f04dc1b904`.
- Slice 7 code review and verifier passed.
- PR #5 raw CI completed red in two daemon suites, both with startup-health-timeout signatures. Jim accepted the internal carve-out after base-vs-branch reproduction showed base/equivalent and branch both fail before assertions at `tests/harness/src/cluster.rs:68:17` and branch is no worse.
- Final adversarial review returned `NOT-READY` due two HIGH findings. Jim explicitly dispositioned them:
  - card-bound discovery/request-access limitation: defer to Phase 2 / maintainer follow-up, surface in PR description, do not fix in Phase 1;
  - CI carve-out ambiguity: accept after reproduction; `async fn main` hunk is inert AppState field initialization, not startup/health/network/bootstrap behavior.
- Final Craft Review returned Concerns with two CONFORMANCE findings; both are dispositioned below.
- Final clean-context returned Concerns but judged Phase 1 feature-complete under recorded deferrals and CI carve-out.

## Evidence

CI arbiter / green of record:

- Location: draft mirror PR #5, <https://github.com/JimCollinson/x0x/pull/5>
- Status: raw red, internally accepted under `gsd/ci-arbiter.md` startup-timeout carve-out plus Jim's 2026-06-22 integrated-branch acceptance.
- Passing checks included Format Check, Clippy Lint, Documentation, Property Tests, API + GUI Parity Gate, API Coverage Guard, builds, Cargo Audit, Coverage Gate, and release metadata validation.
- Red checks:
  - CI run `27968200087`, job `82767397265` (`Test Suite`): `x0x::named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected` failed at `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-60704 did not become healthy within 90s`; 1682 passed, 1 failed, 164 skipped, 88 not run due fail-fast.
  - Integration & Soak run `27968201873`, job `82767406138` (`Multi-Agent Integration`): `x0x::named_group_integration named_group_admin_disband_propagates_to_peer_after_creator_delete_409` failed at `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-24987 did not become healthy within 90s`; 2 passed, 1 failed, 24 not run due fail-fast.

Local fast gate / `.gsd/gate.sh`:

- Installed? pre-push hook installed in build worktree earlier; `.gsd/gate.sh` not present in this worktree snapshot.
- Pre-push to Jim fork at `1c3f17a` passed local hook commands: `cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`.

Files changed/artifacts produced:

- Build branch head: `1c3f17a58c94c04f4099014bfad121f04dc1b904`.
- Planning artifacts updated:
  - `gsd/plan/phase-1-pr-notes.md`
  - `gsd/checkpoints/2026-06-22-slice-7-surfaces-docs-sweep.md`
  - `gsd/checkpoints/2026-06-22-slice-7-verification-1c3f17a.md`
  - `gsd/checkpoints/2026-06-22-slice-7-adversarial-blocker-1c3f17a.md`
  - this checkpoint.

Checks run / results:

- Slice 7 verifier commands passed: `cargo fmt --all`; `cargo clippy --all-features --all-targets -- -D warnings`; `cargo check --workspace --all-targets`; `cargo run --bin gui-coverage -- --threshold 95`; focused `parity_cli`, `api_manifest`, and `proptest_groups` tests.
- Clean-context reran:
  - `cargo fmt --all -- --check` — passed.
  - `cargo clippy --all-targets --all-features -- -D warnings` — passed.
  - `cargo check --workspace --all-targets` — passed.
  - `cargo nextest run --all-features -E 'test(last_admin)'` — 33 passed.
  - `cargo test --all-features --test owner_retirement` — 8 passed.
  - `cargo test --all-features --test membership_authority` — 23 passed.
  - `cargo test --all-features --test invite_authority` — 3 passed.
  - `cargo nextest run --all-features -E 'test(member) and (test(add) or test(remove) or test(ban))'` — 25 passed.
  - `cargo nextest run --all-features -E 'test(leave) or test(disband) or test(withdraw)'` — 24 passed.
  - `cargo nextest run --all-features -E '(test(role) or test(owner) or test(genesis)) & !binary(named_group_join_metadata_event)'` — 41 passed.
  - `cargo test --all-features --test parity_cli` — 7 passed.
  - `cargo test --all-features --test api_manifest manifest_matches_registry` — 1 passed.
  - `cargo run --bin gui-coverage -- --threshold 95` — passed, 115/117 counted endpoints, 98.3%.
  - CLI help/routes smoke checks for `group set-role`, `group disband`, `group`, `group state-withdraw`, and `routes` — passed/discoverable.
- Known local broad filters can still trigger daemon startup timeout before assertions; this is covered only by the recorded carve-out and must not be called a raw green.

Base-vs-branch reproduction for CI carve-out:

- Base `189b89c`, `cargo test --all-features --test named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected`: failed before assertions at `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-43735 did not become healthy within 90s`; 0 passed, 1 failed, 5 filtered out, 90.31s.
- Branch `1c3f17a`, same command: failed before assertions at `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-47536 did not become healthy within 90s`; 0 passed, 1 failed, 7 filtered out, 90.25s.
- Branch `1c3f17a`, `cargo test --all-features --test named_group_integration named_group_admin_disband_propagates_to_peer_after_creator_delete_409 -- --ignored`: failed before assertions at `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-26163 did not become healthy within 90s`; 0 passed, 1 failed, 26 filtered out, 90.22s.
- Base `189b89c` equivalent predecessor `cargo test --all-features --test named_group_integration named_group_creator_delete_propagates_to_peer -- --ignored`: failed before assertions at `tests/harness/src/cluster.rs:68:17`, `x0xd pair-alice-22805 did not become healthy within 90s`; 0 passed, 1 failed, 22 filtered out, 90.19s.

## Honesty rules check

- No-harness-modification: Pass.
  - No `.gsd/gate.sh`, CI workflow, test harness, service/daemon wrapper, build invocation, environment setup, `Cargo.toml`, or `Cargo.lock` changes in Slice 7. The integrated `async fn main` hunk called out by adversarial is `expected_join_result_inviters: StdMutex::new(HashMap::new())`, an AppState bookkeeping field initialization, not startup/health/network/bootstrap behavior.
- Baseline-diff for evidence: Pass.
  - CI red classified only after base/equivalent vs branch reproduction showed the same pre-assertion startup-health timeout and branch no worse.
- Evidence reproducible-from-branch: Concern / Pass with caveat.
  - Build branch evidence is reproducible from branch. Planning/CI carve-out records currently live in the separate planning worktree and need to be handed to reviewers or committed/pushed on the planning branch.
- Local vs CI consistency: Pass with caveat.
  - Local focused tests pass; daemon startup-health timeouts reproduce locally and in CI. Raw CI is red and must not be called green.

## Review findings

Clean-context test:

- Reviewer/tool: `cleancontext` agent.
- Result: Concerns, no blocker.
- Findings:
  - Phase 1 feature-complete under recorded Phase 2 deferrals and CI startup-timeout carve-out.
  - Raw PR #5 CI is red; do not present as raw green.
  - CI acceptance depends on planning records outside the build branch.
  - Card-bound request-access/discovery limitation remains real but visible as Phase 2.

Adversarial review:

- Reviewer/tool: `adversarial` agent (`gpt-5.5`; implementer model/provider not provided, so cross-provider independence could not be proven).
- Required? Yes.
- Result: Blockers found, then dispositioned by Jim.
- Findings:
  - HIGH card-bound discovery/request-access limitation: Jim deferred to Phase 2 / maintainer follow-up; PR notes updated; no Phase 1 fix.
  - HIGH integrated CI raw red/carve-out ambiguity: Jim accepted carve-out after base-vs-branch reproduction; raw CI remains red.
  - MEDIUM signed metadata apply paths still gate on transport sender, not just signed commit authority: carried as review concern/future work, not blocking Phase 1 after Jim's dispositions.
  - LOW residual owner roster wording: dispositioned via Craft Review as historical/versioned limitation wording.

Craft Review:

- Reviewer/tool: `craft` agent.
- Verdict: Concerns.
- CONFORMANCE findings and dispositions:
  - Current docs contain owner-era wording in versioned x0x 0.21.0 TreeKEM limitation notes. Disposition: justified as historical/versioned wording, not current owner authority requirement.
  - Provisional `disband` verb is not literally isolated as a one-line swap. Disposition: PR notes corrected to say a maintainer-requested verb change requires a small multi-surface sweep; no code change unless #107 returns a different verb.
- SIMPLICITY / NIT findings carried: none beyond the above.

## Drift / scope concerns

- Card-bound discovery/request-access authority seeding is deliberately deferred to Phase 2 / maintainer follow-up. This should be placed in the PR description's deferred-to-Phase-2 section with delegated ban / KeyPackage distribution.
- `disband` is provisional but consistent. If David answers #107 with another verb before PR, do a small CLI/API/docs/GUI sweep and rerun appropriate checks.
- Planning artifacts are not all committed/pushed at this checkpoint; do not assume a build-branch-only reviewer has them.

## Open questions / decisions for Jim

PR / upstream action gate:

- PR ready to raise? Not automatically. Feature implementation is complete, but Jim's explicit PR action decision is still required; upstream PR must carry the deferral and raw-red carve-out wording.
- Jim confirmed PR may be opened? No.
- Draft PR title/description prepared: running PR notes exist in `gsd/plan/phase-1-pr-notes.md`; not converted into final PR body here.

## Recommended next step

If Jim wants to proceed toward upstream PR: prepare final PR description from `gsd/plan/phase-1-pr-notes.md`, explicitly including Phase 2 deferrals and raw-red/internal-carve-out CI status, then ask Jim for the exact PR-open approval. Do not open PR without that approval.

## Handoff note

Phase 1 build branch is feature-complete at `1c3f17a` under recorded deferrals/carve-out. Raw PR #5 CI is red but internally accepted; do not overstate. The card-bound discovery/request-access limitation is real and intentionally deferred to Phase 2. The provisional `disband` verb is consistent but not one-line-swappable. Planning artifacts must be preserved/pushed before external handoff.
