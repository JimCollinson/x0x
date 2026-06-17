# Verification — Slice 4 invites per issuer + creator provenance (ADR-0016 Phase 1)

- Date: 2026-06-17
- Verifier: OpenAI GPT-5.5 verifier
- Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
- Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
- Feature branch/head verified: `feat/adr-0016-phase-1-authority-alignment` @ `8085b340586a94539b1b3cd3e1a19418b493c8fa`
- Slice 4 delta inspected: `449ac8077dc55d7a91f9aa1acaaf6f992cc96ca7..8085b340586a94539b1b3cd3e1a19418b493c8fa`
- Latest static/transient expected-inviter refactor inspected: `aa6774a3a8c7b8ee2744ce15a58334ece01d5d05..8085b340586a94539b1b3cd3e1a19418b493c8fa`
- Status: **human_needed for final acceptance; 7/7 requested implementation goals verified**

## Result

Score: **7/7 requested implementation goals verified**.

The code/artifact goals are satisfied after `8085b34`. The latest refactor removed the expected-inviter field and initialization from `AppState` / `fn main` in the net Slice 4 diff; the expected inviter is now transient process-local state behind a `LazyLock<StdMutex<HashMap<...>>>` and is accessed only by join-result record/read/clear helpers.

Final Slice 4 acceptance is still **not green-of-record**: PR #5 checks for head `8085b34` are red. The red state is **not** the prior “only daemon-startup health-timeout” carve-out: `Coverage Gate` failed on an assertion in `peer_lifecycle_integration::peer_health_snapshot_observable_for_live_peer`, and `Test Suite` failed with GitHub runner `No space left on device`. Human/operator action is needed to rerun or triage CI before claiming PR #5 green.

## Goal-backward checks

1. **Active Admin invite issuance works; plain Member rejection works — VERIFIED.**
   - `create_group_invite` derives the local daemon agent as `inviter_hex` and calls `require_admin_or_above(info, &inviter_hex)`.
   - `require_admin_or_above` delegates to `GroupInfo::caller_role`, which returns a role only for active members, and accepts `Admin` or higher.
   - The old creator-only gate is absent from the invite issue handler.
   - `tests/invite_authority.rs` covers promoted non-creator Admin success, creator success, plain Member rejection, and rejected-member no-secret-tracking.

2. **Creator provenance derives from invite base state, not unsigned `invite.inviter` — VERIFIED.**
   - `SignedInvite::creator_agent_id_from_base_state()` derives creator provenance from the base roster snapshot (`added_by == None`, preferring `group_created_at` match), validates 32-byte hex, and explicitly documents that `invite.inviter` is unsigned routing metadata.
   - `join_group_via_invite` calls this helper before constructing local group state, parses `creator` from the helper result, and keeps parsed `inviter` separate for routing/polling.
   - `invite_join_group_info` seeds `GroupInfo` / `GroupGenesis::with_existing_id` with the derived creator, not `invite.inviter`.

3. **Non-TreeKEM base-state invite stubs stay coherent and inviter-authored `MemberAdded` advances roster/hash together — VERIFIED.**
   - `invite_join_group_info` seeds revision, roster, state hash, prev hash, secure plane, epoch, and security binding from the invite base state.
   - For modern non-TreeKEM invites carrying `base_state_hash`, it does not insert a missing local joiner under the base hash.
   - The inviter-side `MemberJoined` handler builds and signs an authority `MemberAdded` commit; receivers apply through `apply_stateful_event_to_group` / commit finalization so roster and hash advance together.
   - Focused regression `non_treekem_admin_invite_joiner_validates_member_added_state_chain` passed and checks creator, inviter, and joiner converge to the same coherent post-apply state.

4. **Self-rejoin REST/display behavior works without committed active local joiner mutation under base hash — VERIFIED.**
   - If the base roster already contains the local joiner, `invite_join_group_info` refreshes only display/key-package metadata and keeps role/state/hash at the authority frontier.
   - The regression recomputes state hash after display refresh and confirms hash coherence.
   - Ignored real-daemon test `named_group_rejoin_after_leave` passed locally in this verification run.

5. **Expected join-result inviter is checked against stored expected inviter; direct sender and `MemberAdded.actor` must match — VERIFIED.**
   - `record_expected_join_result_inviter` stores the expected inviter by stable group id + member id when TreeKEM join-result polling is prepared.
   - `handle_join_result_message` reads the stored expected inviter and calls `validate_join_result_inviter`; missing expected inviter, wrong direct sender, or wrong `MemberAdded.actor` are rejected.
   - `poll_join_result_until_treekem_ready` sends fetches to the parsed original inviter and clears the expected entry on completion/timeout; successful apply clears it immediately.
   - The latest refactor moved this storage out of `AppState` / `fn main` while preserving the transient record/read/clear behavior.
   - Unit test `join_result_requires_stored_expected_inviter` passed and covers missing expected inviter, wrong sender, wrong actor, and success.

6. **Missing-base legacy invite behavior is explicit; stale fallback removed; no fallback to `invite.inviter` — VERIFIED.**
   - `creator_agent_id_from_base_state()` returns `invite missing base member snapshot; cannot derive creator provenance` when base members are missing.
   - `join_group_via_invite` returns that error before local `GroupInfo` construction.
   - The remaining no-base helper path is documented as defensive direct/helper construction only and deliberately does not derive creator/member authority from unsigned `invite.inviter`.
   - Unit test `creator_provenance_does_not_fall_back_to_unsigned_inviter` passed.

7. **Scope boundaries — VERIFIED at code diff level; CI still blocks final acceptance.**
   - Net Slice 4 diff (`449ac80..8085b34`) touches only `src/bin/x0xd.rs`, `src/groups/invite.rs`, and `tests/invite_authority.rs`.
   - Latest refactor (`aa6774a..8085b34`) touches only `src/bin/x0xd.rs`.
   - No changes found under `tests/harness/**`, CI workflows, `.gsd/gate.sh`, daemon wrappers, `src/network*`, `src/bootstrap*`, or `src/presence*`.
   - No invite wire-format field changes in Slice 4; `src/groups/invite.rs` changes are comment/helper/test-only around existing fields.
   - Net Slice 4 `x0xd.rs` hunk headers do **not** include `fn main` or `AppState` initialization. The earlier `aa6774a` AppState/main expected-inviter hunk is removed in the net `8085b34` slice diff.

## Artifact verification matrix

| Artifact | Exists | Substantive | Wired | Status |
|---|---:|---:|---:|---|
| `src/bin/x0xd.rs` invite issue/join/result paths | Yes | Yes | Yes — REST routes, metadata apply path, direct join-result listener/poller | VERIFIED |
| `src/groups/invite.rs` creator provenance helper | Yes | Yes | Yes — called by `join_group_via_invite`; unit-covered | VERIFIED |
| `tests/invite_authority.rs` | Yes | Yes | Yes — integration test binary; ran 3/3 PASS | VERIFIED |
| x0xd focused invite-stub / join-result regressions | Yes | Yes | Yes — compiled in `bin/x0xd` test target; ran 5/5 PASS | VERIFIED |
| `named_group_rejoin_after_leave` daemon test | Yes | Yes | Yes — ignored integration test; ran 1/1 PASS | VERIFIED |

No relevant changed implementation artifact is a stub: the inspected functions are real logic, not placeholder returns/TODO-only implementations.

## Evidence run/inspected by verifier

- `git status --short --branch` in build worktree — clean at `8085b34`.
- `git rev-parse HEAD && git log --oneline -12` — verified head and remediation history.
- `git diff --name-status fork/main...HEAD` — full feature branch contains expected Phase 1 files.
- `git diff --name-status 449ac8077dc55d7a91f9aa1acaaf6f992cc96ca7..HEAD` — Slice 4 touches only `src/bin/x0xd.rs`, `src/groups/invite.rs`, `tests/invite_authority.rs`.
- `git diff --name-status aa6774a3a8c7b8ee2744ce15a58334ece01d5d05..HEAD` — latest refactor touches only `src/bin/x0xd.rs`.
- `git diff 449ac8077dc55d7a91f9aa1acaaf6f992cc96ca7..HEAD -- src/bin/x0xd.rs | rg '^@@'` — net Slice 4 hunks exclude `fn main` / AppState initialization.
- `git diff --check 449ac8077dc55d7a91f9aa1acaaf6f992cc96ca7..HEAD` — PASS.
- `cargo fmt --all -- --check` — PASS. (Verifier used non-mutating `--check`; implementer-reported `cargo fmt --all` PASS after latest code changes.)
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS.
- `cargo check --workspace --all-targets` — PASS.
- `cargo nextest run --all-features --all-targets -E 'test(non_treekem_invite_stub_refreshes_existing_joiner_display_without_rehash) or test(non_treekem_admin_invite_joiner_validates_member_added_state_chain) or test(treekem_invite_stub_matches_authority_base_hash) or test(join_result_requires_stored_expected_inviter) or test(creator_provenance_does_not_fall_back_to_unsigned_inviter)'` — PASS, 5/5.
- `cargo nextest run --all-features -E 'test(invite) & !binary(named_group_join_metadata_event)'` — PASS, 23/23.
- `cargo nextest run --all-features --test invite_authority` — PASS, 3/3.
- `cargo nextest run --all-features --test named_group_integration --run-ignored ignored-only -E 'test(named_group_rejoin_after_leave)'` — PASS, 1/1.
- `gh pr view 5 --repo JimCollinson/x0x --json headRefOid,statusCheckRollup` — PR #5 head is `8085b34`; checks are complete but red.

## CI / acceptance caveat

PR #5 is the CI green of record and is **not green** at this verification point:

- `Coverage Gate` — FAIL, run `27677058259`, job `81854835682`: `peer_lifecycle_integration::peer_health_snapshot_observable_for_live_peer` assertion failed (`snapshot.connected` was false; close_reason `Superseded`). This is not a daemon-startup health-timeout.
- `Test Suite` — FAIL, run `27677058259`, job `81854835787`: GitHub runner `System.IO.IOException: No space left on device` while writing runner diagnostics.
- `Multi-Agent Integration` — PASS.
- `Build windows-x64` and other build jobs — PASS.
- Format, Clippy, Documentation, API/GUI parity, API coverage guard, cargo audit, panic scanner, property tests, and release metadata — PASS.

Because the red jobs are not exclusively daemon-startup health-timeouts, the `gsd/ci-arbiter.md` carve-out does not apply as-is. No baseline-diff reproduction was run for the peer lifecycle assertion, so it must not be dismissed as pre-existing/flaky by this verifier.

## Recommendation

Implementation goals are verified, but do **not** mark Slice 4 final/PR-ready until PR #5 has a green-of-record determination. Recommended next step: rerun or triage the failed PR #5 jobs. If CI turns green, proceed to the required adversarial and Craft Review gates; if the peer lifecycle assertion repeats, treat it as a blocking CI failure unless a base-commit reproduction or human arbiter explicitly classifies it outside Slice 4.
