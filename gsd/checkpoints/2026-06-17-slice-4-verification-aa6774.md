# Verification — Slice 4 invites per issuer + creator provenance (ADR-0016 Phase 1)

- Date: 2026-06-17
- Verifier: OpenAI GPT-5.5 verifier
- Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
- Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
- Feature branch/head verified: `feat/adr-0016-phase-1-authority-alignment` @ `aa6774a3a8c7b8ee2744ce15a58334ece01d5d05`
- Slice 4 delta inspected: `449ac8077dc55d7a91f9aa1acaaf6f992cc96ca7..aa6774a3a8c7b8ee2744ce15a58334ece01d5d05`
- Latest remediation delta inspected: `67bda7513634a534991aaf7d96e4087c9be9e92b..aa6774a3a8c7b8ee2744ce15a58334ece01d5d05`
- Status: **human_needed for final acceptance; 7/7 implementation goals verified**

## Result

Score: **7/7 requested goals verified**.

The implementation goals are satisfied at code/artifact level after `aa6774a`. Final Slice 4 acceptance still needs a CI arbiter decision because PR #5 was not final at verification time: two CI jobs were red with daemon-startup health-timeout signatures and `Build windows-x64` was still pending. The strict `gsd/ci-arbiter.md` diff guard may require human judgement because the Slice 4 diff includes an `AppState` field initialization hunk inside `fn main`, even though no startup/health/networking logic was changed.

## Goal-backward checks

1. **Any active Admin can issue invites; plain Member cannot — VERIFIED.**
   - `create_group_invite` now derives `inviter_hex` from the local daemon agent and calls `require_admin_or_above(info, &inviter_hex)`.
   - The old creator-only rejection (`agent_id != info.creator`) is absent.
   - `tests/invite_authority.rs` covers promoted non-creator Admin success, creator success, plain Member rejection, and rejected-member no-secret-tracking.

2. **Joiner `GroupInfo.creator` derives from invite base/genesis, not unsigned `invite.inviter` — VERIFIED.**
   - `SignedInvite::creator_agent_id_from_base_state()` derives creator provenance from the invite `base_members_v2` seeded creator entry and errors on missing/invalid base state.
   - `join_group_via_invite` parses `creator_hex` from that helper, then keeps parsed `inviter` separate for routing/polling.
   - `invite_join_group_info` seeds `GroupInfo` / `GroupGenesis::with_existing_id` with the derived creator, not `invite.inviter`.

3. **Non-TreeKEM admin-issued invite stubs seed coherent base state and do not pre-commit a missing joiner under the base hash; inviter-authored `MemberAdded` advances roster/hash together — VERIFIED.**
   - `invite_join_group_info` copies invite base revision, members, state hash, previous state hash, secure plane, epoch, and security binding for all invite joins.
   - For modern non-TreeKEM invites with `base_state_hash`, the helper does not add a missing local joiner before the authority-signed `MemberAdded` commit.
   - `apply_stateful_event_to_group` + `finalize_applied_commit` validate that the post-apply roster/hash matches the inviter-authored commit.
   - Focused regression `non_treekem_admin_invite_joiner_validates_member_added_state_chain` passed and checks creator, inviter, and joiner converge to the same hash/revision.

4. **Existing self-rejoin REST/display behavior works without reintroducing committed active local joiner mutation under base hash — VERIFIED.**
   - When the invite base already contains the local joiner, `invite_join_group_info` refreshes only non-committed display/key-package metadata and leaves role/state/hash at the authority frontier.
   - `compute_roster_root` ignores display/key-package fields; the regression recomputes and confirms hash coherence.
   - Real daemon ignored test `named_group_rejoin_after_leave` passed in this verification run.

5. **Expected join-result inviter MEDIUM fixed — VERIFIED.**
   - New `expected_join_result_inviters` map stores the expected inviter keyed by stable group id + member id before TreeKEM join-result polling.
   - `handle_join_result_message` now requires a stored expected inviter and rejects responses unless both direct-message sender and `MemberAdded.actor` match that expected inviter.
   - `poll_join_result_until_treekem_ready` sends fetches to parsed `invite.inviter`, not creator, and removes the expected entry on success/timeout.
   - Unit test `join_result_requires_stored_expected_inviter` passed and covers missing expected inviter, wrong sender, wrong actor, and success.

6. **Missing-base legacy invite behavior explicit; stale unsigned-inviter fallback removed — VERIFIED.**
   - `creator_agent_id_from_base_state()` explicitly errors with `invite missing base member snapshot; cannot derive creator provenance`.
   - `join_group_via_invite` returns that error before constructing local `GroupInfo`, so the production path does not fall back to unsigned `invite.inviter`.
   - The remaining helper fallback is documented as defensive direct/helper construction only and does not derive authority/provenance from `invite.inviter`.
   - Unit test `creator_provenance_does_not_fall_back_to_unsigned_inviter` passed.

7. **Scope boundaries — VERIFIED WITH CI-ARBITER CAVEAT.**
   - Full Slice 4 diff changes only `src/bin/x0xd.rs`, `src/groups/invite.rs`, and `tests/invite_authority.rs`.
   - Latest remediation `67bda75..aa6774a` changes only `src/bin/x0xd.rs` and `src/groups/invite.rs`.
   - No changes found under `tests/harness/**`, CI workflows, `.gsd/gate.sh`, daemon wrappers, storage/wire/hash/signing formats, `src/network*`, `src/bootstrap*`, or `src/presence*`.
   - Caveat: the diff adds `expected_join_result_inviters: RwLock::new(HashMap::new())` in the `AppState` construction inside `fn main`. This is an in-memory state initialization, not a startup/health/networking behavior change, but the internal CI carve-out's strict diff guard names `fn main`; a human/arbiter decision is needed if CI remains red only on startup timeouts.

## Artifact verification matrix

| Artifact | Exists | Substantive | Wired | Status |
|---|---:|---:|---:|---|
| `src/bin/x0xd.rs` invite issue/join/result paths | Yes | Yes | Yes — REST routes, metadata apply path, direct join-result listener/poller | VERIFIED |
| `src/groups/invite.rs` creator provenance helper | Yes | Yes | Yes — called by `join_group_via_invite`; unit-covered | VERIFIED |
| `tests/invite_authority.rs` | Yes | Yes | Yes — integration test binary; ran 3/3 PASS | VERIFIED |
| x0xd focused unit regressions | Yes | Yes | Yes — compiled in `bin/x0xd` test target; ran 5/5 PASS | VERIFIED |
| `named_group_rejoin_after_leave` daemon test | Yes | Yes | Yes — ignored integration test; ran 1/1 PASS | VERIFIED |

## Evidence run/inspected by verifier

- `git status --short --branch` in build worktree — clean at `aa6774a`.
- `git rev-parse HEAD && git log --oneline -8` — verified head and remediation history.
- `git diff --name-status fork/main...HEAD` — no forbidden harness/CI/gate/wrapper/network module changes.
- `git diff --name-status 449ac80..HEAD` — Slice 4 touches only `src/bin/x0xd.rs`, `src/groups/invite.rs`, `tests/invite_authority.rs`.
- `git diff --name-status 67bda75..aa6774a` — latest remediation touches only `src/bin/x0xd.rs`, `src/groups/invite.rs`.
- `git diff --check 449ac80..HEAD` — PASS.
- `cargo nextest run --all-features --all-targets -E 'test(non_treekem_invite_stub_refreshes_existing_joiner_display_without_rehash) or test(non_treekem_admin_invite_joiner_validates_member_added_state_chain) or test(treekem_invite_stub_matches_authority_base_hash) or test(join_result_requires_stored_expected_inviter) or test(creator_provenance_does_not_fall_back_to_unsigned_inviter)'` — PASS, 5/5.
- `cargo nextest run --all-features --test invite_authority` — PASS, 3/3.
- `cargo nextest run --all-features --test named_group_integration --run-ignored ignored-only -E 'test(named_group_rejoin_after_leave)'` — PASS, 1/1.
- `cargo nextest run --all-features -E 'test(invite) & !binary(named_group_join_metadata_event)'` — PASS, 23/23.
- `gh pr checks 5 --repo JimCollinson/x0x` — at verification time: `Test Suite` FAIL and `Multi-Agent Integration` FAIL with daemon-startup timeout signatures; all completed non-failing jobs green/skipped; `Build windows-x64` still pending.

Previously reported evidence after latest code changes, not re-run in full by verifier:

- `cargo fmt --all` — PASS.
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS.
- `cargo check --workspace --all-targets` — PASS.
- Repeat code review after `aa6774a` — `passed`.

## CI / acceptance caveat

PR #5 is the CI green of record, but it was **not final** at verification time:

- `Test Suite` run `27675273703`, job `81848930111` — FAIL: `x0xd pair-alice-22183 did not become healthy within 90s` in `forged_member_joined_admin_role_or_secret_is_rejected`.
- `Multi-Agent Integration` run `27675273711`, job `81848930407` — FAIL: `x0xd pair-alice-30564 did not become healthy within 90s` in `named_group_import_rejects_tampered_metadata_topic`.
- `Build windows-x64` — pending when checked.

These red jobs match the daemon-startup health-timeout signature and are small in count, but final arbiter status requires the pending build to complete and either a rerun green or an explicit human/arbiter acceptance of the carve-out, especially because the strict diff guard names `fn main` and Slice 4 necessarily added an in-memory map initialization there.

## Recommendation

Implementation goals are verified. Do **not** mark Slice 4 final/PR-ready solely from this verifier result until CI reaches a final arbiter determination. Recommended next step: wait for/re-run PR #5 checks; if only the same daemon-startup health-timeout failures remain, have Jim/Hermes explicitly decide whether the carve-out applies despite the non-behavioural `fn main` AppState initialization hunk. Then run/record the required adversarial and Craft Review gates before PR/merge readiness.
