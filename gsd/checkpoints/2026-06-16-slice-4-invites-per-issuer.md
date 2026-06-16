# Checkpoint — Slice 4 invites per issuer + creator provenance (ADR-0016 Phase 1)

- Date: 2026-06-16
- Slice/question: Slice 4 — Invites per-issuer + creator provenance (R7/R8), plus code-review HIGH remediation
- Prepared by: OpenCode implementer
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `4fabfccada662d43719c7da71dd1d8818ccb5157`
- Status: **Implementation remediated; ready for independent review**

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
- Repeat independent code review after remediation: Not run — required before Slice 4 can be accepted as Done.
- Clean-context test: Not run — deferred until behaviour is complete enough to exercise from repo/docs / PR-readiness.
- Adversarial review: Not run — required before Slice 4 can be accepted as Done unless Jim explicitly waives or defers.
- Craft Review: Not run — required before Slice 4 can be accepted as Done unless Jim explicitly waives or defers.

## Current gate status

Implementation and local verification are complete. PR #5 internal CI arbiter is satisfied under the approved generalized known-flake carve-out. Slice 4 should not be marked Done until the remediation receives repeat independent code/adversarial/Craft review, or Jim explicitly waives/defers those gates.

## Recommended next step

Run repeat independent code review focused on `680198b..4fabfcc`, then run/record the required adversarial and Craft reviews for Slice 4. If those pass with no blocking findings, approve the next packet (Slice 5).
