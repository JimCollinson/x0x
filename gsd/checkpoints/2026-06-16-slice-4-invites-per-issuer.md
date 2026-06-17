# Checkpoint — Slice 4 invites per issuer + creator provenance (ADR-0016 Phase 1)

- Date: 2026-06-16
- Slice/question: Slice 4 — Invites per-issuer + creator provenance (R7/R8), plus code-review HIGH remediation
- Prepared by: OpenCode implementer
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `4fabfccada662d43719c7da71dd1d8818ccb5157`
- Status: **Blocked — adversarial HIGH state-hash/roster coherence finding**

## What changed

- Removed the invite issue-side creator-only gate and made active Admin-or-higher role lookup the issuing authority.
- Preserved per-issuer invite issue/consume/track routing: `invite.inviter` remains the join-result polling and delivery target.
- Added deterministic creator provenance derivation from invite base-state member snapshot so a joiner's `GroupInfo.creator` comes from seeded genesis/base state, never unsigned `invite.inviter` metadata.
- Added fast, gate-runnable coverage for promoted non-creator Admin issuing an invite, plain member rejection, creator-issued invite regression, and base-state creator provenance when inviter differs from creator.
- Remediated the code-review HIGH finding by seeding the invite base state for non-TreeKEM joins too, while keeping inviter routing distinct from creator provenance.

## Commits

- `680198b38c55c380bafc8adc3da1ac0a0b2f5607` — `feat(adr-0016-phase-1): allow admin-issued invites`
- `4fabfccada662d43719c7da71dd1d8818ccb5157` — `fix(adr-0016-phase-1): seed non-TreeKEM invite base state`

## Files changed on feature branch

- `src/bin/x0xd.rs`
- `src/groups/invite.rs`
- `tests/invite_authority.rs`

Remediation-only diff (`680198b..4fabfcc`) touched only:

- `src/bin/x0xd.rs`
- `src/groups/invite.rs`

## Code-review HIGH remediation

Reviewer finding: non-TreeKEM admin-issued invites likely did not converge for non-creator admins because `join_group_via_invite` copied `invite.base_members_v2` only for TreeKEM invites. Non-TreeKEM joins seeded a new local `GroupInfo`, added only the joiner, and recomputed state hash, so an inviter-authored `MemberAdded` could fail against a different base frontier.

Remediation performed:

- Confirmed the failure with a deterministic regression test before the fix: `PrevHashMismatch { expected: Some("3903091e38a5ec0d238efe24f0100b6713d88f12a9e9b7f1e1b510eccfe5cde6"), got: Some("60433951805f0afc0739d774700f87432cce377dc7ec2aeb99163ae46bc2778c") }`.
- Hoisted base-state seeding so all invite joins can seed from the invite's base roster/state hash, not only TreeKEM joins.
- Preserved TreeKEM-specific behavior where the invite carries no shared secret.
- For non-TreeKEM joins, kept local joiner roster seeding but did not recompute `state_hash` when the invite carries `base_state_hash`; the inviter-authored `MemberAdded` now validates against the inviter's base frontier.
- No invite wire-format, storage-format, hash, signing, test-harness, daemon-wrapper, CI, build, or environment changes were made.

## Local verification evidence

Mandatory Rust order after code changes:

- `cargo fmt --all` — PASS
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS
- `cargo check --workspace --all-targets` — PASS

Targeted and supporting checks:

- `cargo nextest run --all-features --all-targets -E 'test(non_treekem_admin_invite_joiner_validates_member_added_state_chain)'` — FAIL before remediation with the `PrevHashMismatch` above; PASS after remediation
- `cargo nextest run --all-features --test invite_authority` — PASS, 3/3
- `cargo nextest run --all-features -E 'test(invite) & !binary(named_group_join_metadata_event)'` — PASS, 23/23
- `cargo nextest run --all-features --all-targets -E 'test(treekem_invite_stub_matches_authority_base_hash)'` — PASS

Previously recorded Slice 4 supporting checks before remediation:

- `cargo nextest run --all-features -E 'test(invite)'` — PASS, 25 tests across 69 binaries
- `cargo nextest run --all-features -E 'test(creator_provenance) or test(invite_authority)'` — PASS, 6/6
- `git diff --check` — PASS

## PR #5 CI arbiter status

Green of record source: PR #5, <https://github.com/JimCollinson/x0x/pull/5>.

Head `4fabfccada662d43719c7da71dd1d8818ccb5157` was pushed to Jim's fork. PR #5 reports all checks green except the two daemon-startup timeout jobs below; under `gsd/ci-arbiter.md`, this is **green of record modulo known mesh flake** for the internal mirror PR.

Initial post-remediation CI invocation:

- `Test Suite`: run `27622938623`, job `81676594825` — FAIL
  - Failing test: `x0x::named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected`
  - Failure: `x0xd pair-alice-16409 did not become healthy within 90s`
  - Summary: `1747/1752 tests run: 1746 passed (1 slow), 1 failed, 161 skipped`; 5 not run due fail-fast.
- `Multi-Agent Integration`: run `27622939039`, job `81676596180` — FAIL
  - Failing test: `x0x::named_group_integration::named_group_creator_removal_propagates_to_removed_peer`
  - Failure: `x0xd pair-alice-17066 did not become healthy within 90s`
  - Summary: `6/24 tests run: 5 passed, 1 failed`; 18 not run due fail-fast.

Per the packet's CI heads-up, reran failed jobs only, without code, harness, gate, environment, or workflow changes:

- `gh run rerun 27622938623 --failed --repo JimCollinson/x0x`
- `gh run rerun 27622939039 --failed --repo JimCollinson/x0x`

Rerun results:

- `Test Suite`: run `27622938623`, rerun job `81680553773` — FAIL
  - Failing test: `x0x::named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected`
  - Failure: `x0xd pair-alice-54818 did not become healthy within 90s`
  - Summary: `1747/1752 tests run: 1746 passed (1 slow), 1 failed, 161 skipped`; 5 not run due fail-fast.
- `Multi-Agent Integration`: run `27622939039`, rerun job `81680551200` — FAIL
  - Failing test: `x0x::named_group_integration::named_group_creator_delete_propagates_to_peer`
  - Failure: `x0xd pair-bob-59792 did not become healthy within 90s`
  - Summary: `5/24 tests run: 4 passed, 1 failed`; 19 not run due fail-fast.

### Known-flake carve-out invocation

The latest invocation qualifies under the internal arbiter's signature / isolation / diff-guard rule:

- Signature: both red jobs failed only on daemon-startup health-timeouts before any test assertion.
  - `Test Suite`: run `27622938623`, rerun job `81680553773`; failing test `x0x::named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected`; verbatim line: `x0xd pair-alice-54818 did not become healthy within 90s`.
  - `Multi-Agent Integration`: run `27622939039`, rerun job `81680551200`; failing test `x0x::named_group_integration::named_group_creator_delete_propagates_to_peer`; verbatim line: `x0xd pair-bob-59792 did not become healthy within 90s`.
  - No assertion failure, diagnostic-counter mismatch, or timeout inside already-running test logic was reported. Tests not run due fail-fast are not counted as failures.
- Isolation: 2 timed-out tests across all jobs (`<= 3`); the rest of PR #5's reported checks were green or skipped-by-workflow (`Soak Test`).
- Diff guard: satisfied. Slice 4 changed only `src/bin/x0xd.rs`, `src/groups/invite.rs`, and `tests/invite_authority.rs`; remediation changed only `src/bin/x0xd.rs` and `src/groups/invite.rs`. It changed nothing under `tests/harness/`, nothing under `src/network*`, `src/bootstrap*`, or `src/presence*`, and no `src/bin/x0xd.rs` startup/health code (`fn main`, serve/startup sequence, `/health`, node/transport/bootstrap initialization).
- Upstream provenance: the latest failing tests exist at untouched base `189b89c`:
  - `tests/named_group_join_metadata_event.rs:556`
  - `tests/named_group_integration.rs:1296`

Determination: **green of record (modulo known mesh flake)** for the internal PR #5 arbiter. This records and reasons about the red CI jobs; it does not claim upstream CI passed, and the harness flake remains flagged to David.

## Closing creator-authority sweep

Current-head search of `src/bin/x0xd.rs` for remaining creator/inviter comparisons and uses:

- `sender_hex != creator_hex` in the public group-card receive path — creator/provenance/public-card path, not invite authority.
- API output fields rendering `"creator"` — metadata/output only.
- invite join path uses `invite.creator_agent_id_from_base_state()` for creator provenance and `invite.inviter` for routing/result delivery — intended Slice 4 split.
- leave/delete/disband creator checks and `treekem_leave_disposition` tests — Slice 5 scope.
- join-request notification/routing placeholder using `creator_hex` — routing placeholder, not invite issue authority.

No remaining invite issue-side creator authority gate was found.

## Coverage notes

Fast-gate / normal nextest coverage completed:

- promoted non-creator Admin can pass invite issue authorization at the helper/state seam;
- plain `member` cannot issue an invite;
- creator-issued invite path still passes;
- when inviter differs from creator, creator provenance derives from base-state creator rather than unsigned inviter;
- consume-side inviter-admin role check remains the authority at consume/apply;
- non-TreeKEM creator != inviter join/apply state-chain semantics now validate against the inviter's base frontier;
- TreeKEM invite stub authority-base hash still matches after the generalized seeding change.

Maintainer-gate daemon/mesh assertions remain affected by the pre-existing startup-health flake and are not claimed as cleanly exercised by the failing CI jobs:

- promoted non-creator Admin issuing through the real daemon;
- joiner consuming against issuing daemon;
- join-result polling routing to non-creator inviter;
- creator-issued invite end-to-end.

## Honesty rules check

- No-harness-modification: PASS — no changes to tests/harness, CI workflow, `.gsd/gate.sh`, daemon wrappers, build invocation, or environment.
- Baseline-diff for evidence: PASS with recorded carve-out — the CI failures are classified only under the approved internal arbiter rule because they match the daemon-startup timeout signature, are isolated to 2 tests, and the Slice 4 diff guard shows no startup/health/networking changes. Latest failing tests exist at base `189b89c`.
- Evidence reproducible-from-branch: PASS for local checks; PR #5 remains the green of record and is satisfied modulo the recorded known mesh flake under the internal arbiter.
- Local vs CI consistency: PASS with caveat — local checks pass; PR #5 is green of record modulo the recorded known mesh flake under the internal arbiter.

## Review gates

- Prior code review: `issues_found` with one HIGH finding on non-TreeKEM admin-issued invite convergence; remediated in `4fabfcc`.
- Repeat independent code review after remediation: `passed`.
  - Reviewer/tool: `codereviewer` subagent.
  - Commands run:
    - `cargo fmt --all -- --check` — PASS.
    - `cargo clippy --all-features --all-targets -- -D warnings` — PASS.
    - `cargo check --workspace --all-targets` — PASS.
    - `cargo nextest run --all-features --all-targets -E 'test(non_treekem_admin_invite_joiner_validates_member_added_state_chain)'` — PASS, 1/1.
    - `cargo nextest run --all-features --test invite_authority` — PASS, 3/3.
    - `cargo nextest run --all-features -E 'test(invite) & !binary(named_group_join_metadata_event)'` — PASS, 23/23.
    - `cargo nextest run --all-features --all-targets -E 'test(treekem_invite_stub_matches_authority_base_hash)'` — PASS, 1/1.
    - `gh pr checks 5 --repo JimCollinson/x0x` — all green except known red `Test Suite` + `Multi-Agent Integration`, classified under the internal arbiter carve-out.
  - Judgment: original HIGH was real; repro meaningful; remediation had no code-review blockers.
- Verifier: `passed` with CI carve-out caveat.
  - Reviewer/tool: `verifier` subagent.
  - Note: `gsd/checkpoints/2026-06-16-slice-4-verification.md`.
  - Result: 8/8 goals verified, but later adversarial review superseded this as final acceptance evidence.
- Clean-context test: Not run — deferred until behaviour is complete enough to exercise from repo/docs / PR-readiness.
- Adversarial review: **NOT-READY**.
  - Reviewer/tool: `adversarial` subagent, same reported model family (`openai/gpt-5.5`), so independence is weaker than cross-provider.
  - HIGH blocker: non-TreeKEM join stores a roster that does not match its `state_hash`. The adversarial reviewer cited `src/bin/x0xd.rs:10708-10710` preserving `base_state_hash`, `src/bin/x0xd.rs:10713-10724` adding the joiner to non-TreeKEM `members_v2`, and `src/bin/x0xd.rs:10733-10734` avoiding recompute when `base_state_hash` exists. Result: persisted `members_v2 = base roster + joiner` while `state_hash` commits to pre-join base roster, weakening state-chain coherence even though it fixes the original `PrevHashMismatch`.
  - MEDIUM: join-result “inviter” check compares sender to `MemberAdded.actor`, not an independently stored expected invite inviter; checkpoint wording overclaims inviter routing validation.
  - MEDIUM: legacy/missing-base invites now hard-fail; may be acceptable for R8, but compatibility decision is not explicit.
  - MEDIUM: fast invite authority tests mirror helper behavior rather than the real daemon handler.
  - Test-quality note: remediation test proves the original `PrevHashMismatch`, but also encodes the incoherent state-staging approach and does not prove full daemon convergence.
- Craft Review: Not run — adversarial HIGH blocks acceptance; run after remediation.

## Current gate status

Implementation and local verification are complete, and PR #5 internal CI arbiter is satisfied under the approved generalized known-flake carve-out, but Slice 4 is **not ready** because adversarial review found an unresolved HIGH state-hash/roster coherence issue. Slice 4 must not be marked Done until this is remediated and repeat review gates pass.

## Recommended next step

Remediate the state-hash/roster coherence issue: do not persist an active joiner in non-TreeKEM `members_v2` while retaining the pre-join base `state_hash`. Reassess the join-result sender check (`MemberAdded.actor` versus expected invite inviter) and explicitly document/decide legacy invite behavior. Then rerun mandatory checks, targeted tests, PR #5 arbiter, code review, verifier, adversarial, and Craft Review.

---

## Follow-up remediation — coherent stubs + rejoin REST view

- Date: 2026-06-16
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `67bda7513634a534991aaf7d96e4087c9be9e92b`
- Status: **Blocked — repeat code review returned MEDIUM issues; PR #5 not yet green-of-record**

### Additional commits

- `70db9af7cc1e1253358aaa360809fac58ee7e76a` — `fix(adr-0016-phase-1): keep invite join stubs coherent`
  - Modern non-TreeKEM invite stubs with `base_state_hash` stay exactly at the invite authority frontier.
  - Missing joiners are not inserted as active local members before the inviter-authored `MemberAdded` commit.
  - Regression strengthened for pre-commit exclusion, post-apply convergence, and state-hash/roster coherence.
- `67bda7513634a534991aaf7d96e4087c9be9e92b` — `fix(adr-0016-phase-1): refresh invite rejoin display metadata`
  - Fixes the real-daemon `named_group_rejoin_after_leave` regression without reintroducing active local add under a base hash.
  - When the invite authority base already contains the local joiner, only non-committed display/key-package metadata is refreshed for the local REST view; role/state and `state_hash` remain at the base frontier.

### Local verification evidence after `67bda75`

Mandatory Rust order:

- `cargo fmt --all` — PASS
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS
- `cargo check --workspace --all-targets` — PASS

Targeted checks:

- `cargo nextest run --all-features --all-targets -E 'test(non_treekem_invite_stub_refreshes_existing_joiner_display_without_rehash) or test(non_treekem_admin_invite_joiner_validates_member_added_state_chain) or test(treekem_invite_stub_matches_authority_base_hash)'` — PASS, 3/3
- `cargo nextest run --all-features --test invite_authority` — PASS, 3/3
- `cargo nextest run --all-features -E 'test(invite) & !binary(named_group_join_metadata_event)'` — PASS, 23/23
- `cargo nextest run --all-features --test named_group_integration --run-ignored ignored-only -E 'test(named_group_rejoin_after_leave)'` — PASS, 1/1

Additional attempted local check:

- `cargo nextest run --all-features --test named_group_integration --run-ignored ignored-only` — FAIL before completing due known daemon-startup health-timeout:
  - `named_group_creator_delete_propagates_to_peer`
  - `x0xd pair-alice-52254 did not become healthy within 90s`
- Single-test rerun of that timed-out test also failed with the same startup signature:
  - `x0xd pair-alice-54775 did not become healthy within 90s`
- This was not used as pass evidence.

### PR #5 CI status at checkpoint

Head `67bda7513634a534991aaf7d96e4087c9be9e92b` was pushed to Jim's fork. At the time of this checkpoint:

- `Test Suite`: run `27642281701`, job `81745156482` — FAIL with daemon-startup timeout:
  - failing test `x0x::named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected`
  - verbatim line: `x0xd pair-alice-32496 did not become healthy within 90s`
  - summary: `1747/1752 tests run: 1746 passed (1 slow), 1 failed, 161 skipped`
- `Property Tests` — PASS
- `Format Check`, `Clippy Lint`, `Coverage Gate`, `Documentation`, `API + GUI Parity Gate`, `API Coverage Guard`, `Cargo Audit`, `Panic Scanner`, release metadata, and completed build jobs — PASS
- `Multi-Agent Integration` and `Build windows-x64` were still pending when this checkpoint was recorded.

No CI carve-out determination is final yet because all jobs had not completed.

### Repeat code review after `67bda75`

Reviewer/tool: `codereviewer` subagent.

Result: `issues_found`.

Findings:

- Prior adversarial HIGH appears remediated. Reviewer found no evidence that `67bda75` grants active membership to a missing non-TreeKEM joiner or breaks state-chain hash coherence.
- MEDIUM: expected-inviter check still compares join-result sender to `MemberAdded.actor`; it does not compare against an independently stored expected inviter from the original invite/poll target.
- MEDIUM/coverage: legacy missing-base invite behavior is inconsistent in comments/code path. `invite_join_group_info` still contains a fallback branch for missing-base non-TreeKEM invites, but real `join_group_via_invite` calls `creator_agent_id_from_base_state()` first and appears to reject such invites before fallback can run. If intentional, document as compatibility decision; if not, fix.
- PROCESS/CI: PR #5 not green at review time; do not claim CI green.

### Current gate status

Slice 4 remains **not Done**. Do not proceed to verifier/adversarial/Craft until the code-review issues are fixed or explicitly accepted/deferred by Jim, and until PR #5 reaches a final arbiter determination.

---

## Follow-up remediation — grounded Slice 4 pass

- Date: 2026-06-17
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `95684702f8061e42b1b16684cb37f5582dbcee7b`
- Status: **implemented and committed locally; not pushed; CI/review gates still required on the new head**

### Additional commits

- `4287904ad5ad915c1d6c0b328056cbd5284eaf26` — `fix(adr-0016-phase-1): role-gate group card receipt`
  - Replaced the `GroupCardPublished` receive-path creator check with `info.caller_role(&sender_hex)` and `role.at_least(GroupRole::Admin)`.
  - Kept the existing `card.group_id == info.stable_group_id()` and `card.verify_signature()` checks.
  - Discovery/cache note: if a receiver's local roster has not yet converged to show the sender as Admin, this discovery-cache receive path may transiently reject that card and later re-accept once roster convergence catches up; it is not the signed state chain.
- `d40fb29d980ceed73903764dcf691b52d08f2691` — `refactor(adr-0016-phase-1): keep expected inviters on app state`
  - Moved expected join-result inviter storage from the process-global `LazyLock<StdMutex<HashMap<...>>>` onto `AppState` beside pending join/welcome state.
  - Preserved TTL pruning and clear-after-use; helper locks are synchronous and not held across `.await`.
  - Explicitly touched `AppState` construction in `fn main` to initialize the transient map.
- `95684702f8061e42b1b16684cb37f5582dbcee7b` — `test(adr-0016-phase-1): cover admin-issued invite e2e`
  - Added a real three-daemon REST/handler-path test: creator creates `public_open`, non-creator member is promoted to Admin, that Admin issues an invite, a separate joiner consumes it through `POST /groups/join`, and creator/admin/joiner are asserted to converge on role, `added_by`, `state_hash`, and `roster_root`.
  - The test also asserts observable routing/provenance split: invite `inviter` is the non-creator Admin, while group `creator` remains the historical creator.

### Creator provenance / authority sweep

`creator provenance is best-effort historical, derived from the base-state snapshot; it is not authority-bearing and is not a tamper-evident guarantee.`

Post-Item-1 source search found no remaining creator authority path except Slice-5-deferred DELETE leave-vs-disband behavior:

- API/list/detail rendering of `creator` is output/provenance only.
- Invite join uses `creator_agent_id_from_base_state()` for creator provenance and keeps inviter/routing separate.
- `treekem_leave_disposition` / `DELETE /groups/:id` still consult creator for leave-vs-disband semantics — explicit Slice 5 carry-note.
- Join-request `creator_hex` remains a reserved direct-notification routing placeholder, not authority.
- Tests use `creator` fixtures/provenance assertions.

No deliberate creator-only card/discovery intent was found in the read docs/comments; ADR-0016 explicitly calls receive-path creator checks bugs to delete, and the publish path is already any-admin.

### Local verification evidence after `9568470`

Mandatory Rust order after Rust changes:

- `cargo fmt --all` — PASS.
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS.
- `cargo check --workspace --all-targets` — PASS.

Focused checks:

- `cargo nextest run --all-features --test invite_authority` — PASS, 3/3.
- `cargo nextest run --all-features --all-targets -E 'test(non_treekem_invite_stub_refreshes_existing_joiner_display_without_rehash) or test(non_treekem_admin_invite_joiner_validates_member_added_state_chain) or test(treekem_invite_stub_matches_authority_base_hash) or test(join_result_requires_stored_expected_inviter) or test(creator_provenance_does_not_fall_back_to_unsigned_inviter)'` — PASS, 5/5.
- New e2e attempted: `cargo nextest run --all-features --test named_group_join_metadata_event -E 'test(non_creator_admin_invite_e2e_converges_through_real_daemons)'` — FAIL before test assertions with daemon startup health-timeout (`x0xd test-alice-37390 did not become healthy within 90s`). Immediate rerun failed with the same startup signature (`x0xd test-alice-22608 did not become healthy within 90s`).
- Non-`all-features` attempt of the new e2e also failed before assertions with the same startup signature (`x0xd test-alice-59955 did not become healthy within 90s`).
- Existing daemon e2e comparison `cargo nextest run --test named_group_join_metadata_event -E 'test(member_joined_event_propagates_to_inviter)'` also failed before assertions with the same startup signature (`x0xd pair-alice-57129 did not become healthy within 90s`). These startup-timeouts are the packet's carve-out class and were not used as functional pass evidence.

### Honesty / scope

- No changes to `tests/harness/**`, CI workflows, `.gsd/gate.sh`, daemon wrappers, build invocation, or environment setup.
- No serde role names, role bytes, hashing, signing, commit format, storage format, wire format, `roster_root`, or `state_hash` computation changes.
- No PR opened and no push performed.

### Current gate status

The remediation implementation is locally committed. The new head has not been pushed, so PR #5 CI is not the green of record for this head yet. Next orchestrator action should be push-to-fork/CI, then repeat verifier/adversarial/Craft gates as required.

---

## Remaining remediation delta — discovery scope + TreeKEM e2e

- Date: 2026-06-17
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `9901c9c2a4834ddf20065bd9cfab1d2730838b9d`
- Planning branch/head before this checkpoint update: `gsd/adr-0016-planning` @ `aea64953ff8f776aa43b44ae0a9f71a05844ef80`
- Status: **implemented locally; not pushed; CI/review gates still required on the new head**

### Additional commit

- `9901c9c2a4834ddf20065bd9cfab1d2730838b9d` — `test(adr-0016-phase-1): add TreeKEM admin invite e2e`
  - Refactors the existing real three-daemon non-creator-admin invite proof through a preset-aware helper.
  - Keeps the `public_open` / non-TreeKEM e2e coverage.
  - Adds a `private_secure` TreeKEM variant that exercises non-creator Admin invite issue, secure-plane join / Welcome handling, creator-provenance-vs-inviter split, and final state/roster/security-binding convergence across creator, Admin, and joiner.
  - Direct expected-inviter sender/actor rejection remains covered by the focused `join_result_requires_stored_expected_inviter` unit regression; it is not overclaimed from the daemon e2e alone.

### Item 1b — discovery claim scoped and flagged to David

- Confirmed scope: the known-local-group `GroupCardPublished` metadata-apply path now enforces active Admin authority before accepting the card into the receiver's local cache (`4287904`).
- Correction to prior over-broad wording: the global discovery listener, directory shard listener, and ListedToContacts direct-card listener are best-effort signed-hint / key-possession discovery caches, not current group-admin authority checks.
- For known local groups, these pre-existing David C.2/D.3 discovery receive paths can cache or override a signed discovery listing without confirming the signer is currently an active Admin. This is cosmetic discovery cache state only, not committed group state.
- Disposition: flag to David as a pre-existing observation. Slice 4 intentionally does **not** harden the discovery receive paths.
- Scope guard satisfied: no edits to the global discovery, shard discovery, or ListedToContacts receive logic; no edits to `src/groups/directory.rs`.

### Local verification evidence after `9901c9c`

Mandatory Rust order after Rust test changes:

- `cargo fmt --all` — PASS.
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS.
- `cargo check --workspace --all-targets` — PASS.

Focused checks:

- `cargo nextest run --all-features --test invite_authority` — PASS, 3/3.
- `cargo nextest run --all-features --all-targets -E 'test(join_result_requires_stored_expected_inviter) or test(treekem_invite_stub_matches_authority_base_hash)'` — PASS, 2/2.
- `cargo nextest run --all-features --test named_group_join_metadata_event -E 'test(non_creator_admin_invite_e2e_converges_through_real_daemons) or test(non_creator_admin_private_secure_invite_e2e_uses_expected_inviter_join_result)'` — FAIL before assertions with daemon startup health-timeout: `x0xd test-alice-12532 did not become healthy within 90s`; second selected test was not run due fail-fast. This command used the original over-broad TreeKEM test name before the wording/name was narrowed.
- `cargo nextest run --all-features --test named_group_join_metadata_event -E 'test(non_creator_admin_private_secure_invite_e2e_uses_expected_inviter_join_result)'` — FAIL before assertions with daemon startup health-timeout: `x0xd test-alice-37643 did not become healthy within 90s`. This command used the original over-broad TreeKEM test name before the wording/name was narrowed.

### Follow-up narrowing after code review

- Date: 2026-06-17
- Follow-up local build head: `feat/adr-0016-phase-1-authority-alignment` @ pending commit after `9901c9c2a4834ddf20065bd9cfab1d2730838b9d`
- Code review finding: the `private_secure` e2e is meaningful but cannot uniquely prove direct `JoinResultMessage::Result` routing, because final convergence could also be satisfied by metadata gossip plus Welcome retrieval.
- Disposition: narrowed the test comment/name and PR-note wording. The daemon e2e now claims TreeKEM secure-plane end-to-end join shape, Welcome/security-binding convergence, and creator-vs-inviter split. Direct expected-inviter sender/actor validation is explicitly attributed to the focused `join_result_requires_stored_expected_inviter` unit regression.

The e2e failures match the known startup-timeout class and are not assertion failures. They are recorded as attempted evidence, not a clean local functional pass.

### Review gate placeholders for rerun

- Verifier: **Not rerun after `9901c9c`**. The committed artifact `gsd/checkpoints/2026-06-17-slice-4-grounded-remediation-verification-9568470.md` verifies the previous `9568470` head only; rerun verifier on the new head after orchestrator push/CI.
- Adversarial review: **Not run after `9901c9c`**; required before marking Slice 4 Done unless Jim explicitly waives or defers.
- Craft Review: **Not run after `9901c9c`**; required before marking Slice 4 Done unless Jim explicitly waives or defers.
- Clean-context test: still deferred until PR-readiness / behaviour is exerciseable from repo/docs.

### Honesty / scope

- No changes to `tests/harness/**`, CI workflows, `.gsd/gate.sh`, daemon wrappers, build invocation, or environment setup.
- No serde role names, role bytes, hashing, signing, commit format, storage format, wire format, `roster_root`, or `state_hash` computation changes.
- No discovery receive-path hardening was performed; only planning/spec/PR-note wording scopes the claim and flags the observation.
- No PR opened and no push performed.

### Current gate status

The remaining remediation delta is locally implemented and the verifier artifact from the prior head is now committed in planning. The build head is ahead of the fork by one commit and has not been pushed, so PR #5 CI is not the green of record for `9901c9c`. Slice 4 should remain **not Done** until orchestrator push/CI and rerun verifier/adversarial/Craft gates complete on the new head.
