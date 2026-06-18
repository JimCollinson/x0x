# Checkpoint — Slice 5 leave/disband split (ADR-0016 Phase 1)

- Date: 2026-06-18
- Slice/question: Slice 5 — resolved §3 leave/end-group split
- Prepared by: OpenCode orchestrator + operative
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `5761a1fb5a961014e73961843025103f77626a8c`
- Status: **Local verified; CI/review gates pending**
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

## Files changed on feature branch

- `src/bin/x0xd.rs`
- `src/bin/x0x.rs`
- `src/cli/commands/group.rs`
- `src/api/mod.rs`
- `docs/api-reference.md`
- `tests/membership_authority.rs`
- `tests/parity_cli.rs`

## Local verification evidence

Mandatory Rust order after code changes:

- `cargo fmt --all` — PASS
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS
- `cargo check --workspace --all-targets` — PASS

Targeted and supporting checks:

- `cargo nextest run --all-features -E 'test(leave) or test(disband) or test(withdraw)'` — PASS, 23/23
- `cargo nextest run --all-features --test membership_authority --test parity_cli` — PASS, 28/28
- `cargo nextest run --all-features -E 'test(treekem_leave_disposition) or test(cli::commands::group)'` — PASS, 37/37
- Local `.gsd/gate.sh` commands (`cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings`) — PASS
- `git diff --check HEAD~1..HEAD` — PASS

## Closing creator / GroupDeleted sweep

- `== info.creator` / `!= info.creator` / `CreatorMustDelete` across Rust sources — no matches.
- `NamedGroupMetadataEvent::GroupDeleted` remains only in enum/kind/receive/apply compatibility and an internal test fixture. No production emit/direct-delivery site remains.
- `GroupDeleted` apply comments were narrowed to legacy compatibility and current disband-over-withdrawal/card semantics.

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
- Baseline-diff for evidence: N/A so far — no local failures or skips are being dismissed as pre-existing.
- Evidence reproducible-from-branch: PASS for local commands; all readiness commands use committed repo commands, no uncommitted scripts/wrappers/env vars.
- Local vs CI consistency: CI not yet inspected for this head.

## CI arbiter status

Green of record source: PR #5, <https://github.com/JimCollinson/x0x/pull/5>.

- Status: pending — commit not yet pushed at this checkpoint draft.
- Known internal carve-out remains only for isolated daemon-startup `x0xd ... did not become healthy within <N>s` failures per `gsd/ci-arbiter.md`.

## Review gates

- Code review: pending.
- Verifier: pending.
- Adversarial review: pending; required before Slice 5 acceptance.
- Craft Review: pending; required before Slice 5 acceptance.
- Clean-context: pending if behavior is exerciseable from repo/docs; required before PR-ready.

## Current gate status

Slice 5 implementation is locally verified and ready to push to Jim's fork for PR #5 CI, then per-unit review gates. It is **not accepted/done** until CI arbiter and required reviews pass.

## Recommended next step

Push `5761a1f` to Jim's fork, inspect PR #5 CI, then run code-review, verifier, adversarial, Craft Review, and clean-context gates as applicable.
