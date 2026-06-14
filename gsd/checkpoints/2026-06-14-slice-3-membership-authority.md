# Checkpoint — Slice 3: membership authority add/remove/ban (ADR-0016 Phase 1)

- Date: 2026-06-14
- Slice: 3 of 7 (`gsd/plan/phase-1-plan.md`, packet `gsd/packets/2026-06-14-slice-3-membership-authority.md`)
- Feature branch: `feat/adr-0016-phase-1-authority-alignment`
- Base entering slice: Slice 2 head `b9f6b37`
- Slice commit: `6ebac93e423e3fab60f91481adad6a86fb212445` — `feat(adr-0016-phase-1): align membership authority with admin roles`
- CI arbiter / green of record: draft mirror PR #5, <https://github.com/JimCollinson/x0x/pull/5>, head `6ebac93`
- Status: **Slice complete; local mandatory/targeted gates pass; PR #5 CI green of record passed.**

## What changed

- `src/bin/x0xd.rs`
  - Deleted creator identity gates from REST add/remove and TreeKEM add/remove member handlers.
  - Deleted `cannot remove creator` and `cannot ban owner` special cases.
  - Add/remove/ban REST handlers now use `require_admin_or_above` as the surviving authority check.
  - Add/remove/ban gossip apply paths now require event `actor` metadata to match the signed commit signer and rely on `ActionKind::AdminOrHigher` (or existing `MemberSelf` for self-leave) for authority.
  - TreeKEM queue authorization no longer creator-gates add/remove/ban; it keys membership events to signed `commit.committed_by` metadata consistency.
  - Remove/ban last-admin pre-checks remain before mutation / TreeKEM KeyPackage lookup / TreeKEM side effects.
  - Non-TreeKEM add/remove/ban changed to clone-first authoring before committing the new group state back into the map.
- `src/groups/mod.rs`
  - Added `LAST_ADMIN_PRECHECK_ERROR` and `last_admin_precheck_error` library seam so the exact §3 string is normal-gate testable outside the `x0xd` bin test target.
- `tests/membership_authority.rs`
  - New normal-gate coverage for promoted-admin add/remove/ban through REST-semantics helpers and gossip/state-commit apply.
  - Covers promoted admin removing/banning a legacy `Owner` when another admin remains.
  - Covers exact last-admin 409 string via the shared library seam.
  - Covers plain `member` add/remove/ban denial through REST semantics and gossip apply.
  - Pins TreeKEM ban precedence: last-admin conflict before missing KeyPackage; non-last-admin missing target material remains the existing 424-class error, not creator/owner 403.

## Current-head grep evidence / drift

Pre-edit current-head evidence matched the packet's expected sites with line drift from earlier slices:

- Add creator gates existed at `x0xd.rs` around 11027 and 11154.
- Remove creator target/authority gates existed around 11331, 11334, 11557, 11566.
- Ban owner guards existed around 12330 and 12482; queue/apply owner guards around 7605 and 8716.
- TreeKEM queue/apply creator gates existed around 7574, 7583, 8226, 8441.
- Out-of-scope creator uses remained at invite around 10609, leave/delete around 11416 and 11877, and provenance/routing surfaces around group-card/join-result handling.

Post-edit grep evidence at `6ebac93`:

- No add/remove/ban creator or owner-target authority strings remain:
  - Search for `local_agent != info.creator`, `agent_id_hex == hex::encode(info.creator`, `info.caller_role(&agent_id_hex) == Some(x0x::groups::GroupRole::Owner)`, `creator_auth`, `only the creator can add members`, `only the creator can remove other members`, `cannot remove creator`, `cannot ban owner` found **no add/remove/ban matches**.
- Remaining literal creator comparisons are out of scope for Slice 3:
  - `x0xd.rs:9193` — `GroupCardPublished` provenance/card publication path.
  - `x0xd.rs:17844` — join-result routing/provenance path.
  - Invite creator gate remains around `x0xd.rs:10596` for Slice 4.
  - Leave/delete creator semantics remain around `x0xd.rs:11392` and `x0xd.rs:11837` for Slice 5.
- Surviving role gates in Slice 3 paths:
  - Add REST / TreeKEM add: `x0xd.rs:11016`, `x0xd.rs:11144` use `require_admin_or_above(info, &actor_hex)`.
  - Remove REST / TreeKEM remove: `x0xd.rs:11309`, `x0xd.rs:11533` use `require_admin_or_above(info, &local_agent_hex)`.
  - Ban REST / TreeKEM ban: `x0xd.rs:12273`, `x0xd.rs:12428` use `require_admin_or_above(info, &caller_hex)`.
  - Last-admin pre-check ordering remains before KeyPackage lookup in TreeKEM remove/ban: `x0xd.rs:11543`, `x0xd.rs:12432`.
- Gossip/apply authority disposition:
  - MemberAdded apply uses actor/committer consistency then `ActionKind::AdminOrHigher`.
  - MemberRemoved apply uses actor/committer consistency then `ActionKind::AdminOrHigher` or existing `MemberSelf` self-leave handling.
  - MemberBanned apply uses actor/committer consistency then `ActionKind::AdminOrHigher`.

No uncited remove/ban creator-identity authority sites were found. No stop condition fired.

## Verification evidence

### Local mandatory Rust checks

Run in exact required order after final code/comment changes:

| Command | Result |
|---|---|
| `cargo fmt --all` | PASS |
| `cargo clippy --all-features --all-targets -- -D warnings` | PASS — `Finished dev profile`, zero warnings |
| `cargo check --workspace --all-targets` | PASS |

Pre-push GSD tripwire also passed on push (`cargo fmt --all -- --check`; `cargo clippy --all-targets --all-features -- -D warnings`).

### Local Slice 3 tests

| Command | Result |
|---|---|
| `cargo nextest run --all-features -E 'test(member) and (test(add) or test(remove) or test(ban))'` | PASS — 25/25 (includes all 12 new `membership_authority` tests; one existing proptest marked slow but passed) |
| `cargo nextest run --all-features --test membership_authority` | PASS — 12/12 |

Supplemental full workspace run:

| Command | Result |
|---|---|
| `cargo nextest run --all-features --workspace` | Local FAIL in known macOS mesh setup class: 1738 passed, 1 failed, 161 skipped, 5 not run due fail-fast. Failure: `named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected` panicked in harness setup with `[cluster] FATAL: pair-alice-43281 has zero peers after 30s — mesh is disconnected`. This matches the baseline class recorded in Slice 1 / Slice 2 checkpoints and was not used as readiness evidence. |

### CI arbiter / green of record

Draft mirror PR #5, <https://github.com/JimCollinson/x0x/pull/5>, head `6ebac93`:

- API + GUI Parity Gate: PASS
- API Coverage Guard: PASS
- Build: PASS on `linux-arm64-gnu`, `linux-x64-gnu`, `linux-x64-musl`, `macos-arm64`, `macos-x64`, `windows-x64`
- Cargo Audit: PASS
- Clippy Lint: PASS
- Coverage Gate: PASS
- Documentation: PASS
- Format Check: PASS
- Multi-Agent Integration: PASS
- Panic Scanner: PASS
- Property Tests: PASS
- Test Suite: PASS
- Validate release metadata: PASS
- Soak Test: SKIPPED by workflow

CI is the green of record.

## Honesty rules check

- No-harness-modification: PASS. No `.gsd/gate.sh`, CI workflow, test harness, daemon wrapper, build invocation, or environment setup changed.
- Baseline-diff for evidence: PASS. The only local non-green was the known macOS mesh setup class already baseline-recorded in previous checkpoints; readiness rests on mandatory/targeted local checks plus PR #5 CI green.
- Evidence reproducible-from-branch: PASS. Commands run from committed feature branch head `6ebac93`; no uncommitted scripts, wrappers, or environment variables needed.
- Local vs CI consistency: Expected local macOS mesh difference; CI Linux mirror is green of record.

## Deviations / notes

- Added a small public `x0x::groups` helper/constant solely to make the exact last-admin §3 string normal-gate testable, per the packet's instruction that `x0xd` bin tests are not run and daemon tests are ignored.
- No ignored HTTP daemon tests were added; normal-gate coverage was sufficient through the library/state-commit seam.
- Delegated TreeKEM ban without target KeyPackage remains a 424-class missing-material condition for non-last-admin cases, as required and still deferred to Phase 2 for full operational support.
- TreeKEM apply still preserves existing self-leave handling; Slice 5 remains responsible for leave/disband semantics.
- The planning packet file `gsd/packets/2026-06-14-slice-3-membership-authority.md` is present as an untracked orchestrator-provided file in the planning worktree; this checkpoint commit stages only the checkpoint file.

## Files changed on feature branch

- `src/bin/x0xd.rs`
- `src/groups/mod.rs`
- `tests/membership_authority.rs`

## Recommended next step

Review Slice 3 diff and this checkpoint. If accepted, dispatch Slice 4 — invites per issuer / creator provenance (`gsd/plan/phase-1-plan.md` Slice 4). Carry forward:

- Remaining invite creator gate is expected Slice 4 scope.
- Leave/delete creator semantics are expected Slice 5 scope.
- Full workspace local macOS mesh setup flake remains a local evidence caveat; PR #5 CI is green.
- Plan remains valid.
