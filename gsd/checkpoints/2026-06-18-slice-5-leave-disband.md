# Checkpoint — Slice 5 leave/disband split (ADR-0016 Phase 1)

- Date: 2026-06-18
- Slice/question: Slice 5 — resolved §3 leave/end-group split
- Prepared by: OpenCode orchestrator + operative
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `f5cbe486bb6f731f705c8677cf66929cb8f0dbd4`
- Status: **Blocked — code-review HIGH durable withdrawal tombstone decision needed**
- Meaningful work-unit? Yes — non-trivial Rust API/CLI/group-authority behavior in a shared/upstream-bound repo.
- Review cadence: per-unit reviews required before acceptance; no waiver/deferral for adversarial or Craft Review.

## What changed

- `DELETE /groups/:id` is now pure self-leave for all ranks. The creator-special withdrawal/delete branch was removed.
- TreeKEM creator leave now routes through the same TreeKEM leave path as any active member.
- `leave_treekem_group` now runs the ADR-0016 §3 self-leave pre-check before TreeKEM work so sole active admins get the exact `before leaving` 409.
- `POST /groups/:id/state/withdraw` remains the explicit any-admin group-ending act and is surfaced primarily as `x0x group disband <group_id>`; `state-withdraw` remains a hidden/deprecated CLI alias.
- Existing unchanged `NamedGroupMetadataEvent::GroupDeleted { group_id, revision, actor, commit }` is reparented to disband propagation. Creator `DELETE` does not emit `GroupDeleted`; it emits only self-leave `MemberRemoved`.
- Disband seals the existing withdrawal commit, publishes `GroupDeleted` over the group metadata topic, direct-delivers the same event to active members, refreshes the withdrawn-card path for public discovery supersession, then drops local group state.
- `GroupDeleted` receive/apply now requires a signed withdrawal commit, validates it under `ActionKind::AdminOrHigher`, and removes live local state including named-group/card/MLS/TreeKEM maps, snapshots, and metadata listeners.
- `docs/api-reference.md` targeted rows/prose were corrected for Slice 5 only.
- The stale ignored daemon test was rewritten/renamed to assert creator `DELETE` returns the before-leaving 409, then any-admin `private_secure` disband propagates through `GroupDeleted`.

## Commits

- `5761a1fb5a961014e73961843025103f77626a8c` — `feat(adr-0016-phase-1): split leave and disband`
- `7c09a6864997eec415178e5794a08a8785c4a923` — `chore(adr-0016-phase-1): refresh API manifest`
- `a141d83e3394c42c1bd98c9784db60c731d51634` — `fix(adr-0016-phase-1): propagate disband via group delete event`
- `5abe652d54b8b00a204d0031e509a99a724860bb` — `chore(adr-0016-phase-1): address disband review notes`
- `dfd9896` — `fix(adr-0016-phase-1): close disband mutation window`
- `9486d3f` — `fix(adr-0016-phase-1): route snapshot cleanup safely`
- `ef80d4e` — `fix(adr-0016-phase-1): reject post-withdrawal mutators`
- `5aed24d` — `fix(adr-0016-phase-1): guard terminal group authoring paths`
- `f5cbe48` — `fix(adr-0016-phase-1): preserve withdrawal terminality`

## Files changed on feature branch

- `src/bin/x0xd.rs`
- `src/bin/x0x.rs`
- `src/cli/commands/group.rs`
- `src/api/mod.rs`
- `docs/api-reference.md`
- `docs/design/api-manifest.json`
- `tests/membership_authority.rs`
- `tests/named_group_integration.rs`
- `tests/parity_cli.rs`

## Local verification evidence

Mandatory Rust order after code changes:

- `cargo fmt --all` — PASS
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS
- `cargo check --workspace --all-targets` — PASS

Targeted and supporting checks at `a141d83`:

- `cargo nextest run --all-features -E 'test(leave) or test(disband) or test(withdraw)'` — PASS, 23/23
- `cargo nextest run --all-features --test membership_authority --test parity_cli` — PASS, 29/29
- `cargo nextest run --all-features --no-fail-fast --test api_manifest --test parity_cli --test api_coverage --test gui_smoke --test gui_named_group_parity` — PASS, 42/42
- `cargo test --all-features --bin x0xd treekem_snapshot_drop_file_name_rejects_path_traversal_ids` — PASS
- Local `.gsd/gate.sh`/pre-push commands (`cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings`) — PASS
- `git diff --check HEAD~1..HEAD` — PASS

Targeted ignored daemon proof attempted locally:

- `cargo nextest run --all-features --test named_group_integration -E 'test(named_group_admin_disband_propagates_to_peer_after_creator_delete_409)' --run-ignored ignored-only` — FAIL before exercising the test body with the known startup-timeout signature.
- Verbatim local line: `x0xd pair-alice-50225 did not become healthy within 90s`.
- No harness, daemon wrapper, build invocation, CI, `.gsd/gate.sh`, or environment changes were made.

## Closing creator / GroupDeleted sweep

- `== info.creator` / `!= info.creator` / `CreatorMustDelete` across Rust sources — no matches.
- `NamedGroupMetadataEvent::GroupDeleted` production construction is now exactly the disband path in `withdraw_group_state`; remaining occurrences are enum/kind/receive/apply/test fixture.
- Creator `DELETE` remains self-leave and routes through `MemberRemoved`, not `GroupDeleted`.
- Comments updated from legacy-only to current disband propagation plus old-peer/replay compatibility.

## CI arbiter status

Green-of-record source: PR #5, <https://github.com/JimCollinson/x0x/pull/5>.

- Status for `f5cbe48`: red/pending at time of stop; CI is not the gating issue because code review has a real HIGH blocker.
- Passing checks include API + GUI Parity Gate, API Coverage Guard, platform builds, Cargo Audit, Clippy Lint, Coverage Gate, Documentation, Format Check, Panic Scanner, Property Tests, release metadata validation. Soak Test skipped by workflow.
- `Multi-Agent Integration`: run `27767436747`, job `82157915580` — FAIL with startup-timeout signature before the rewritten disband propagation test body ran.
  - Failing test: `x0x::named_group_integration::named_group_admin_disband_propagates_to_peer_after_creator_delete_409`.
  - Verbatim line: `x0xd pair-alice-46184 did not become healthy within 90s`.
- `Test Suite`: run `27767437240`, job `82157915936` — FAIL with startup-timeout signature in a pre-existing daemon-start test, not in Slice 5 code paths.
  - Failing test: `x0x::named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected`.
  - Verbatim line: `x0xd pair-alice-44566 did not become healthy within 90s`.
- These failures match `gsd/ci-arbiter.md`'s internal carve-out: startup-timeout-only reds may be accepted; anything else red is real. No non-timeout assertion failure remains at `a141d83`.

## Review gates / current blocker

- Code review after `a141d83`: initially passed, then subsequent adversarial/code-review passes found real terminality and cleanup issues.
- Remediations pushed through `f5cbe48`:
  - disband now holds the per-group lock until local live state is removed, then releases before `GroupDeleted` publish/direct delivery;
  - TreeKEM `.snap` deletion paths route through a safe filename helper with regression coverage;
  - local REST/request/invite/self-leave authoring paths reject `withdrawn=true` groups;
  - repeat disband on an already-withdrawn local group is rejected;
  - non-withdrawn card import is rejected when an existing local `GroupInfo` is already withdrawn.
- Current final code review at `f5cbe48`: `issues_found` with HIGH blocker:
  - `withdraw_group_state` removes local `GroupInfo` after disband;
  - the non-withdrawn card-import guard only works while a withdrawn `GroupInfo` still exists;
  - after local drop/restart/no tombstone, an old signed non-withdrawn card can create a fresh local stub with `withdrawn=false` and an active Admin, enabling later local authoring.
- This requires a durable withdrawn tombstone (or equivalent stable-group terminality memory) consulted before card import / invite join / stub creation.
- That directly conflicts with the approved Slice 5 dispatch note that “keep local withdrawn tombstone on every member” is deferred, and likely requires a storage/model decision. Stop for Jim/maintainer decision rather than silently adding storage semantics.
- Verifier/adversarial/Craft/clean-context final gates are not complete because code review has a real HIGH blocker.

## Slice 7 deferred surface backlog

Deferred by Jim on 2026-06-18; Slice 5 intentionally did not expand into the full R9 surface sweep.

- `src/gui/x0x-gui.html`: update owner-gated state controls and user-facing `Withdraw` language to any-admin disband language.
- `tests/gui_named_group_parity.rs`: update GUI expectations if labels/data hooks change.
- `docs/api.md`: update `DELETE /groups/:id`, `/state/withdraw`, `state-withdraw`, and creator-authored member rows.
- `docs/primers/groups.md`: update `owner` and `withdraw / hide` wording.
- `docs/api-reference.md`: finish adjacent stale rows outside Slice 5 scope, especially `Creator-authored member add/removal` and state-chain owner/admin wording.
- `README.md`, proof reports, and design notes: classify remaining `owner`, `creator-authored`, `withdraw`, `delete group`, `Leave or delete`, and `state-withdraw` occurrences as intended legacy/internal vocabulary or stale user-facing text.
- Broader R9 grep before PR: search docs/GUI/API/CLI for `owner`, `creator-authored`, `withdraw`, `delete group`, `Leave or delete`, and `state-withdraw`; fix stale user-facing text or record intentional legacy/internal uses.

## Honesty rules check

- No-harness-modification: PASS — no changes to tests/harness, CI workflow, `.gsd/gate.sh`, daemon wrappers, build invocation, or environment.
- Baseline-diff for evidence: PASS/CONCERN — no non-timeout failure is dismissed. Startup-timeout-only CI/local reds are classified under the already-declared project carve-out; code reviewer also reproduced equivalent daemon-start failure at base.
- Evidence reproducible-from-branch: PASS — readiness commands use committed repo commands, no uncommitted scripts/wrappers/env vars.
- Local vs CI consistency: CONCERN but within carve-out — local focused checks pass; CI red is startup-timeout-only.

## Current gate status

Slice 5 implementation has strong local evidence at `f5cbe48`, but it is **not accepted/done**. Code review found a real HIGH durable-terminality gap that cannot be resolved without deciding whether Slice 5 now includes persistent withdrawn tombstones / terminality memory.

### 2026-06-19 update — keyless withdrawn shell attempt

- Build worktree local head: `939ab8c fix(adr-0016-phase-1): retain withdrawn disband shell`.
- Build branch status: local commit only; **not pushed** because code review failed.
- Implemented approved direction: retain withdrawn `GroupInfo` shell, wipe local crypto material, guard stale-card reanimation, update focused docs/CLI help, and update tests from removed-record semantics to withdrawn-shell semantics.
- Local evidence at `939ab8c`:
  - `cargo fmt --all` — PASS
  - `cargo clippy --all-features --all-targets -- -D warnings` — PASS
  - `cargo check --workspace --all-targets` — PASS
  - `cargo nextest run --all-features -E 'test(leave) or test(disband) or test(withdraw)'` — PASS, 23/23
  - `cargo nextest run --all-features --bin x0xd -E 'test(withdrawn_group_record_guard_matches_stable_id_for_stale_card_imports) or test(withdrawn_group_card_marks_existing_stub_without_regressing_newer_stub)'` — PASS, 2/2
  - `cargo nextest run --all-features --test membership_authority --test parity_cli` — PASS, 29/29
  - `cargo nextest run --all-features --no-fail-fast --test api_manifest --test parity_cli --test api_coverage --test gui_smoke --test gui_named_group_parity` — PASS, 42/42
  - Ignored private-secure proof still failed at startup before assertions with known carve-out line: `x0xd pair-alice-787 did not become healthy within 90s`.
- Independent code review result: **issues_found**; Slice 5 still not done.
  - HIGH: post-withdraw TreeKEM events can still be queued before terminality rejection (`src/bin/x0xd.rs:8241-8264`, `7698-7724`, `7759-7793`). Fix class: short-circuit withdrawn groups before TreeKEM queue path, except explicitly allowed duplicate terminal-withdrawal handling.
  - HIGH: withdrawn card import can wipe a live group without validating card authority against the roster (`src/groups/directory.rs:162-164`, `src/bin/x0xd.rs:13932-13954`, `src/bin/x0xd.rs:644-666`). Fix class: for existing local groups, do not mutate/wipe from a card alone unless authority/chain proof is validated; keep stale non-withdrawn reanimation guard intact.
  - MEDIUM: alias records are not all marked withdrawn (`src/bin/x0xd.rs:11660-11668`, `12241-12265`, `12431-12435`). Fix class: ensure every local alias record for the same stable group is marked `withdrawn=true`, not merely key-cleared.

## Recommended next step

Remediate the `939ab8c` code-review findings before pushing the build branch: block withdrawn groups before TreeKEM queuing/catchup, prevent card-only withdrawn imports from wiping live groups without authority/chain validation, and mark all same-group aliases withdrawn. Do not call Slice 5 done, and do not open a PR, until code review/verifier/adversarial/Craft/clean-context gates pass or Jim explicitly accepts/defers a finding.
