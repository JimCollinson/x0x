# Checkpoint — Slice 5 local remediation follow-up (uncommitted)

- Date: 2026-06-20
- Slice/question: ADR-0016 Phase 1 Slice 5 — adversarial blocker remediation after `fd01679`
- Prepared by: OpenCode orchestrator
- Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
- Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `fd01679e73dd4b012882d6825f0ce869463df079`
- Current status: **Blocked before checkpoint/PR-ready — remediation is local and uncommitted; no CI green-of-record for this exact diff.**
- Meaningful work-unit? Yes — non-trivial Rust API/group-authority/security behavior in upstream-bound repo.
- Review cadence: per-unit/integrated gauntlet run; no Jim waiver/deferral.

## What changed locally

Uncommitted build-worktree diff currently changes only:

- `src/bin/x0xd.rs`
- `docs/api-reference.md`

Local remediation implemented:

1. Added post-crypto withdrawn terminality rechecks before returning crypto effects from:
   - secure GSS encrypt/decrypt;
   - TreeKEM encrypt/decrypt;
   - secure group reseal;
   - secure open-envelope adversarial endpoint;
   - legacy `/mls/groups/:id/encrypt` and `/mls/groups/:id/decrypt`.
2. Added real handler call-site tests for lost-race output suppression. These prove handler response paths drop ciphertext/plaintext/opened secret/secret envelope when post-crypto recheck observes withdrawal. They are not a full real concurrent signed-disband integration proof.
3. Changed card-only withdrawn `GroupCard` import semantics:
   - card-only withdrawal cannot terminally mark/wipe a live/keyed same-stable local group;
   - signed terminal `GroupStateCommit` via metadata/direct disband remains the live-keyed terminality path;
   - keyless discovery stubs/listings can still be superseded by withdrawn cards;
   - stale withdrawn/keyless same-stable stubs do not poison live keyed secure/open-envelope responses.
4. Updated `docs/api-reference.md` to match the live-keyed terminality rule and ADR-0016 admin-authority wording.

## Evidence

CI arbiter / green of record:

- Location: PR #5, <https://github.com/JimCollinson/x0x/pull/5>
- Status for exact current diff: **Not available.** Current remediation is uncommitted/unpushed; PR #5 still points at `fd01679...` and does not validate this diff.
- Current PR #5 CI status observed by clean-context/adversarial: raw red Test Suite at `fd01679...` with startup-timeout failure in `named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected`.
- Under GSD, this blocks readiness until the exact diff is committed/pushed and CI is green, or Jim approves a documented carve-out with base-vs-branch proof.

Local checks run after latest Rust changes:

- `cargo fmt --all` — PASS
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS
- `cargo check --workspace --all-targets` — PASS
- `cargo test --all-features --bin x0xd lost_race` — PASS, 14/14
- `cargo test --all-features --bin x0xd withdrawn` — PASS, 14/14
- `cargo nextest run --all-features -E 'test(leave) or test(disband) or test(withdraw)'` — PASS, 23/23
- `git diff --check` — PASS after docs updates

Scope/honesty:

- No changes to `tests/harness/**`, CI workflows, `.gsd/gate.sh`, daemon wrappers, build invocation, environment setup, Cargo files, or scripts.
- No committed `.gsd/gate.sh` exists in the build worktree.
- Evidence is local only and not reproducible from a clean branch checkout until the remediation is committed.

## Review findings

Code review:

- Reviewer/tool: `codereviewer`
- Result: **Passed** after iterative fixes.
- Notable resolved findings:
  - legacy `/mls/groups/:id/encrypt|decrypt` now post-crypto recheck withdrawn terminality;
  - stale withdrawn same-stable stubs no longer poison live keyed alias responses;
  - same-stable keyed alias protection scans aliases, including stale withdrawn/keyless exact records.

Verifier:

- Reviewer/tool: `verifier`
- Result: **Passed** after final open-envelope stale-stub fix.
- Verified:
  - all named/legacy crypto-effect success paths recheck before returning effect material;
  - handler call-site lost-race tests exist;
  - card-only import cannot wipe live keyed same-stable groups;
  - stale withdrawn stubs do not poison live keyed secure/open-envelope paths;
  - diff scope is limited to `src/bin/x0xd.rs` and `docs/api-reference.md`.

Craft Review:

- Reviewer/tool: `craft`
- Verdict: **Pass with minor advisories; no CONFORMANCE findings.**
- SIMPLICITY advisory: test-only forced-withdrawal hook is a pragmatic seam; consider a short explanatory comment if touching again.
- NIT: optional response-format indentation tidy in legacy MLS JSON blocks; `cargo fmt` passes.

Clean-context test:

- Reviewer/tool: `cleancontext`
- Result: **Blocked**
- Blockers:
  1. PR #5 CI arbiter is red at committed head `fd01679...`.
  2. Current remediation is uncommitted/unpushed, so CI does not validate it.
- Clean-context local acceptance checks passed, but it reproduced the current CI startup-timeout failure locally and did not have base-commit proof to classify it as pre-existing/environmental.

Adversarial review:

- Reviewer/tool: `adversarial`, `openai/gpt-5.5`
- Result: **NOT-READY**
- Blocking findings:
  1. **HIGH:** current remediation is uncommitted; clean branch checkout does not contain it; no CI green for exact diff.
  2. **HIGH:** CI green-of-record absent/red; red startup-timeout failure not base-classified in this checkpoint.
- Resolved docs drift:
  - card-only withdrawal paragraph now requires signed terminal `GroupStateCommit` for live keyed terminality;
  - named group add/remove and state-seal docs now use admin-authority wording;
  - `AdminOnly` public-message wording now treats legacy `Owner` as Admin-equivalent rather than a current rank.
- Test-quality note carried: lost-race tests are meaningful handler response-shape regressions, but they are synthetic hook tests, not full real signed-disband concurrency/persistence proofs. Do not overclaim them.

## Honesty rules check

- No-harness-modification: PASS.
- Baseline-diff for evidence: CONCERN/BLOCKER — current CI red cannot be dismissed without base proof; exact remediation has not run in CI.
- Evidence reproducible-from-branch: BLOCKER — current remediation and this checkpoint are uncommitted.
- Local vs CI consistency: BLOCKER — local focused checks pass; CI arbiter remains red/stale for this exact diff.

## Status

Slice 5 remediation is **locally implemented and locally reviewed**, but **not ready for Jim checkpoint as Done / not PR-ready** under GSD because adversarial and clean-context both block on evidence integrity and CI.

## Recommended next step

Jim decision needed:

1. Approve committing/pushing the exact remediation to Jim's fork branch so PR #5 CI can run; or
2. Keep it local and stop here.

After push, rerun/record:

- CI arbiter for the pushed commit;
- clean-context on the pushed branch;
- adversarial recheck with CI evidence;
- update this checkpoint with final CI and review results.

Do not call Slice 5 done, do not mark PR #5 ready, and do not open any upstream PR until the current diff is committed/pushed, CI is green or explicitly classified with approved evidence, and adversarial/clean-context blockers are cleared.
