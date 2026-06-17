# Slice 4 final readiness verification — ADR-0016 Phase 1

- Date: 2026-06-17
- Verifier: OpenAI GPT-5.5 verifier
- Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
- Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
- Branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `8085b340586a94539b1b3cd3e1a19418b493c8fa`
- PR CI arbiter: <https://github.com/JimCollinson/x0x/pull/5>
- Status: **passed under `gsd/ci-arbiter.md` daemon-startup health-timeout carve-out**

## CI arbiter result

Current PR #5 status was rechecked with `gh pr checks 5 --repo JimCollinson/x0x --json ...` and `gh pr view 5 --repo JimCollinson/x0x --json headRefOid,statusCheckRollup`.

- PR head: `8085b340586a94539b1b3cd3e1a19418b493c8fa`.
- `Coverage Gate`: **PASS**, workflow `CI`, run `27677058259`, job `81858991967`.
- `Test Suite`: **FAIL**, workflow `CI`, run `27677058259`, job `81858991921`.
- All other listed PR checks pass, except `Soak Test` is skipped.

The remaining `Test Suite` red qualifies for the internal carve-out:

- Signature: exactly one failed test, `x0x::named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected`; failure is daemon bring-up health, not a slice assertion, application panic, running-test timeout, or diagnostic mismatch. Verbatim line: `x0xd pair-alice-49630 did not become healthy within 90s`. Rust reports it via the harness panic at `tests/harness/src/cluster.rs:68`, which is the health-timeout mechanism; the backtrace is through `cluster::start_instance`, `cluster::pair_with_extra_config`, `cluster::pair`, before the test body's assertions.
- Isolation: summary was `1747/1752 tests run: 1746 passed, 1 failed, 161 skipped`; `5/1752` were not run only because nextest fail-fast stopped after the single failure. Across PR #5, this is the only red job/failure after the rerun, so the health-timeout count is `1 <= 3`.
- Diff guard: `git diff --name-status 449ac8077dc55d7a91f9aa1acaaf6f992cc96ca7..HEAD` shows only `src/bin/x0xd.rs`, `src/groups/invite.rs`, and `tests/invite_authority.rs`. No changes under `tests/harness/`, `.github/`, `.gsd/`, `src/network*`, `src/bootstrap*`, or `src/presence*`. `src/bin/x0xd.rs` hunk headers are imports, `PendingJoinResult`, `apply_named_group_metadata_event_inner`, `create_group_invite`, `join_group_via_invite`, join-result helpers, `handle_join_result_message`, `poll_join_result_until_treekem_ready`, and tests; no `fn main`, serve/startup sequence, `/health` readiness handler, or node/transport/bootstrap initialization hunk.
- Record: this file records the run/job IDs, failing test, verbatim health-timeout line, and diff-guard confirmation required by `gsd/ci-arbiter.md`.

Conclusion: PR #5 counts as **green of record for this internal development gate** under `gsd/ci-arbiter.md`. This does not waive upstream/maintainer CI expectations.

## Goals result summary

No source re-verification changed the prior implementation result. Slice 4 remains **7/7 goals verified**:

- Any active Admin can issue invites; plain Members cannot. `create_group_invite` gates on `require_admin_or_above(info, &inviter_hex)` and no creator-only gate remains there.
- Invite issue/consume/track remains per issuing daemon: the invite secret is recorded on the inviter's local `GroupInfo`.
- Joiner `GroupInfo.creator` provenance is derived by `SignedInvite::creator_agent_id_from_base_state()` from base roster/genesis state; it does not fall back to unsigned `invite.inviter`.
- Inviter identity remains distinct and is still used as the join-result polling/routing target.
- Join-result handling validates the stored expected inviter against both the direct sender and `MemberAdded.actor`.
- Tests in `tests/invite_authority.rs` cover promoted non-creator Admin invite issuance, plain Member rejection, and creator-issued invite provenance.
- Changed artifacts are substantive and wired; no relevant stub/TODO implementation was found.

## Honesty rules

- No-harness-modification: **pass**. Slice diff does not touch test harness, CI workflow, gate, daemon wrappers, build invocation, startup/health path, or networking/bootstrap/presence modules.
- Baseline-diff for evidence: **pass for readiness claim**. The only remaining CI red is not being dismissed generally; it is accepted solely under the predeclared `gsd/ci-arbiter.md` internal daemon-startup health-timeout carve-out. The prior non-carve-out Coverage Gate assertion now passes after rerun and was locally reproduced passing by the operator.
- Evidence reproducible from branch: **pass**. Verification used committed branch head plus GitHub PR checks/logs; no uncommitted wrapper/script/env var is required.
- Local vs CI consistency: **pass** under arbiter. Local targeted reproduction for the prior Coverage assertion passed, CI Coverage Gate now passes, and the only remaining CI red is the predeclared isolated daemon-startup health-timeout class.

## Blockers / open questions

- No Slice 4 final-readiness blocker under the internal PR #5 arbiter.
- The daemon-startup health-timeout remains a known harness/CI flake to keep visible; this carve-out is internal only and should not be presented as upstream CI green.
