# Checkpoint — Slice 5 leave/disband split (ADR-0016 Phase 1)

- Date: 2026-06-18
- Slice/question: Slice 5 — resolved §3 leave/end-group split
- Prepared by: OpenCode orchestrator + operative
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `7c09a689af0e8f9cbd54a0e5a968b505137095b5`
- Status: **Blocked — code-review HIGH withdrawal propagation gap**
- Meaningful work-unit? Yes — non-trivial Rust API/CLI/group-authority behavior in a shared/upstream-bound repo.
- Review cadence: per-unit reviews required before acceptance; no waiver/deferral for adversarial or Craft Review.

## What changed

- `DELETE /groups/:id` is now pure self-leave for all ranks. The creator-special withdrawal/delete branch was removed.
- TreeKEM creator leave now routes through the same TreeKEM leave path as any active member.
- `leave_treekem_group` now runs the ADR-0016 §3 self-leave pre-check before TreeKEM work so sole active admins get the exact `before leaving` 409.
- `GroupDeleted` production emission/direct-delivery was retired. Receive/apply compatibility remains for old peers/replays.
- `x0x group disband <group_id>` is the primary CLI for existing `POST /groups/:id/state/withdraw`; `state-withdraw` remains a hidden/deprecated alias.
- `src/api/mod.rs` registry now describes `DELETE /groups/:id` as leave-only and uses `group disband` as the primary withdrawal CLI.
- `docs/api-reference.md` targeted rows/prose were corrected for Slice 5 only.

## Commit

- `5761a1fb5a961014e73961843025103f77626a8c` — `feat(adr-0016-phase-1): split leave and disband`
- `7c09a689af0e8f9cbd54a0e5a968b505137095b5` — `chore(adr-0016-phase-1): refresh API manifest`

## Files changed on feature branch

- `src/bin/x0xd.rs`
- `src/bin/x0x.rs`
- `src/cli/commands/group.rs`
- `src/api/mod.rs`
- `docs/api-reference.md`
- `tests/membership_authority.rs`
- `tests/parity_cli.rs`
- `docs/design/api-manifest.json`

## Local verification evidence

Mandatory Rust order after code changes:

- `cargo fmt --all` — PASS
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS
- `cargo check --workspace --all-targets` — PASS

Targeted and supporting checks:

- `cargo nextest run --all-features -E 'test(leave) or test(disband) or test(withdraw)'` — PASS, 23/23
- `cargo nextest run --all-features --test membership_authority --test parity_cli` — PASS, 28/28
- `cargo nextest run --all-features -E 'test(treekem_leave_disposition) or test(cli::commands::group)'` — PASS, 37/37
- `X0X_REGEN_MANIFEST=1 cargo test --test api_manifest` — PASS, regenerated `docs/design/api-manifest.json`
- `cargo nextest run --all-features --no-fail-fast --test api_manifest --test parity_cli --test api_coverage --test gui_smoke --test gui_named_group_parity` — PASS, 42/42
- Local `.gsd/gate.sh` commands (`cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings`) — PASS
- `git diff --check HEAD~1..HEAD` — PASS

## Closing creator / GroupDeleted sweep

- `== info.creator` / `!= info.creator` / `CreatorMustDelete` across Rust sources — no matches.
- `NamedGroupMetadataEvent::GroupDeleted` remains only in enum/kind/receive/apply compatibility and an internal test fixture. No production emit/direct-delivery site remains.
- `GroupDeleted` apply comments were narrowed to legacy compatibility and current disband-over-withdrawal/card semantics.

## Blocker — withdrawal propagation stop-rule

Final code review found a HIGH issue matching Slice 5's binding stop-rule: the existing withdrawal mechanism may be inadequate as the now-primary disband propagation path, especially for hidden/private groups.

Evidence:

- `src/bin/x0xd.rs:11841-11864` — `withdraw_group_state` seals/saves withdrawal locally, then calls only `publish_group_card_to_discovery_inner(&state, &id, false)`.
- `src/bin/x0xd.rs:6509-6528` — `publish_group_card_to_discovery_inner` does not publish Hidden cards; ListedToContacts uses contact-scoped delivery, but Hidden returns after logging `skipping fan-out (Hidden — stays local)`.

Impact: default/private Hidden groups may be disbanded locally without a metadata-topic/direct-member propagation path; other members can keep a supposedly disbanded group live. The approved Slice 5 dispatch said to **flag as pre-existing and stop**, not to invent direct delivery or metadata-withdrawal propagation in Slice 5. Therefore Slice 5 is not accepted and must not be presented as done.

## Slice 7 deferred surface backlog

Deferred by Jim on 2026-06-18; Slice 5 intentionally did not expand into the full R9 surface sweep.

- `src/gui/x0x-gui.html`: update owner-gated state controls and user-facing `Withdraw` language to any-admin disband language.
- `tests/gui_named_group_parity.rs`: update GUI expectations if labels/data hooks change.
- `docs/api.md`: update `DELETE /groups/:id`, `/state/withdraw`, `state-withdraw`, and creator-authored member rows.
- `docs/primers/groups.md`: update `owner` and `withdraw / hide` wording.
- `docs/api-reference.md`: finish adjacent stale rows outside Slice 5 scope, especially `Creator-authored member add/removal` and state-chain owner/admin wording.
- `README.md`, proof reports, and design notes: classify remaining `owner`, `creator-authored`, `withdraw`, `delete group`, and `Leave or delete` occurrences as intended legacy/internal vocabulary or stale user-facing text.
- Broader R9 grep before PR: search docs/GUI/API/CLI for `owner`, `creator-authored`, `withdraw`, `delete group`, `Leave or delete`, and `state-withdraw`; fix stale user-facing text or record intentional legacy/internal uses.

## Honesty rules check

- No-harness-modification: PASS — no changes to tests/harness, CI workflow, `.gsd/gate.sh`, daemon wrappers, build invocation, or environment.
- Baseline-diff for evidence: CONCERN — the withdrawal propagation gap appears pre-existing in the existing `withdraw_group_state` / discovery-card path, but no base reproduction was performed because the binding stop-rule says to surface and stop rather than expand scope.
- Evidence reproducible-from-branch: PASS for local commands; all readiness commands use committed repo commands, no uncommitted scripts/wrappers/env vars.
- Local vs CI consistency: CI pending for this head when stopped; local checks pass, but code review HIGH blocks acceptance before CI/adversarial/Craft.

## CI arbiter status

Green of record source: PR #5, <https://github.com/JimCollinson/x0x/pull/5>.

- Status: **red** for `7c09a68`.
- Passing checks include API + GUI Parity Gate, API Coverage Guard, Cargo Audit, Clippy Lint, Coverage Gate, Documentation, Format Check, Panic Scanner, Property Tests, release metadata validation, and platform builds. Soak Test skipped by workflow.
- `Test Suite`: run `27752423931`, job `82105920746` — FAIL with known startup-timeout signature.
  - Failing test: `x0x::named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected`.
  - Verbatim line: `x0xd pair-alice-1742 did not become healthy within 90s`.
  - This alone would match the internal carve-out signature, but CI is still red because of the Multi-Agent failure below.
- `Multi-Agent Integration`: run `27752423927`, job `82105920578` — FAIL with real assertion/test-expectation mismatch, not startup timeout.
  - Failing test: `x0x::named_group_integration::named_group_creator_delete_propagates_to_peer`.
  - Verbatim assertion context: `delete response: Object {"error": String("a group must always have at least one admin; make another member an admin before leaving"), "ok": Bool(false)}` with `left: Bool(false)` / `right: true`.
  - Interpretation: an ignored multi-agent test still expects creator `DELETE` to disband/propagate. Under Slice 5, creator `DELETE` is self-leave and the sole active admin is correctly blocked. This test needs a Slice 5 expectation update or replacement, but acceptance is already blocked by the withdrawal-propagation HIGH.
- Known internal carve-out remains only for isolated daemon-startup `x0xd ... did not become healthy within <N>s` failures per `gsd/ci-arbiter.md`.

## Review gates

- Initial code review after `5761a1f`: `issues_found`, no blockers; noted one pre-existing x0xd binary unit failure reproduced on base and deferred docs/GUI stale wording.
- Verifier after `5761a1f`: `gaps_found`; blocker was stale `docs/design/api-manifest.json` versus `src/api/mod.rs`.
- Manifest remediation commit `7c09a68` fixed the verifier manifest gap; parity bundle passed 42/42.
- Final code review after `7c09a68`: `issues_found` with HIGH blocker on hidden/private withdrawal propagation. This blocks Slice 5 acceptance.
- Verifier: not rerun after final code-review HIGH; blocked.
- Adversarial review: not run — code-review HIGH blocks before adversarial gate.
- Craft Review: not run — code-review HIGH blocks before Craft gate.
- Clean-context: not run — blocked before clean-context gate.

## Current gate status

Slice 5 implementation and manifest parity are locally green, and `7c09a68` has been pushed to Jim's fork, but Slice 5 is **blocked** by the code-review HIGH withdrawal propagation gap and PR #5 is red due a real stale multi-agent test expectation. It is **not accepted/done**.

## Recommended next step

Stop for Jim/maintainer decision on withdrawal propagation. Do not expand Slice 5 in-session. Options to decide in a follow-up plan: keep disband as explicit local withdrawal plus documented propagation limits, defer private/hidden disband propagation to a later slice/phase, or approve a new scoped mechanism change for withdrawal propagation. Any follow-up also needs to update/replace the ignored `named_group_creator_delete_propagates_to_peer` expectation so CI no longer asserts the retired creator-delete behavior.
