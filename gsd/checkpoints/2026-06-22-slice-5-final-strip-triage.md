# Checkpoint — Slice 5 final-strip triage

- Date: 2026-06-22
- Slice/question: ADR-0016 Phase 1 Slice 5 — final-strip nit triage
- Prepared by: OpenCode operative (`openai/gpt-5.5`)
- Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
- Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
- Starting build head: `f1ecb9f2d2719f4afbab72223cc7abe570db8570`
- Resulting build commit: `ea86d235a8b38fccb58d29959e254b5fce1e97c9` (`fix(adr-0016-phase-1): guard ban group writes`)
- Status: implementation complete locally; no push and no PR action taken.

## Approved nit buckets

1. **Ours + cheap** — non-TreeKEM `ban_group_member` post-rotate write was clean to substitute. It now drops the named-group write guard and calls `store_named_group_info(&state, &id, next)` before save/publish. If the store guard rejects because the group became withdrawn, the handler returns `409 group is withdrawn` and does not save/publish.
2. **Pre-existing / separate subsystem** — raw `POST`/`DELETE /mls/groups/:id/members` endpoints are outside named-group terminality. They use separate `state.mls_groups`, which disband wipes, and cannot resurrect a named group. Recorded in `gsd/plan/phase-1-pr-notes.md` for David; not fixed here.
3. **Cosmetic/docs-only** — long API/docs wording and CLI alias cleanup remain deferred to Slice 7.

## Scope notes

- TreeKEM direct ban was left unchanged. It uses the TreeKEM durable persist path and is not the non-TreeKEM post-rotate raw GSS write covered by this final-strip nit.
- No changes to tests/harness, CI workflows, `.gsd/gate.sh`, daemon wrappers, build invocation, environment setup, Cargo files, wire/commit/hash logic, or storage formats.

## Evidence

Mandatory Rust checks run in the build worktree after the Rust edit:

1. `cargo fmt --all` — PASS after final edit.
2. `cargo clippy --all-features --all-targets -- -D warnings` — PASS after final edit.
3. `cargo check --workspace --all-targets` — PASS after final edit.

Full suite: not run by this operative. Placeholder for orchestrator final suite: `cargo nextest run --all-features --workspace` — pending/not run here.

## Files changed

- Build: `src/bin/x0xd.rs`
- Planning: `gsd/plan/phase-1-pr-notes.md`, this checkpoint file

## Blockers / risks

- No blocker found for the approved final-strip implementation.
- Remaining risk/caveat is the explicitly recorded raw MLS endpoint separate-subsystem note and Slice 7 wording backlog.
