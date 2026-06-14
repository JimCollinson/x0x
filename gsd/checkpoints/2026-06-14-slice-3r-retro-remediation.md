# Checkpoint — Slice 3R retro remediation (ADR-0016 Phase 1)

- Date: 2026-06-14
- Slice/question: Slice 3R remediation after Slice 1-3 retro review
- Feature branch/head: `feat/adr-0016-phase-1-authority-alignment` @ `779835028dae3324a20534f07f0402c47e6d6fe8`
- Slice delta: `6ebac93e423e3fab60f91481adad6a86fb212445..779835028dae3324a20534f07f0402c47e6d6fe8`
- Status: **Blocked after adversarial confirmation. Do not dispatch Slice 4 yet.**

## What happened

- Planning hygiene / retro overclaim corrections were committed on `gsd/adr-0016-planning` as `6ce9f9b`.
- Initial Slice 3R audit found shipped pre-ADR code could author `MemberRoleUpdated` commits assigning `Moderator` / `Guest`; per packet stop condition, implementation paused for Jim.
- Jim selected R2 Option 1: **accept-and-document; do not reject `Moderator` / `Guest` on signed/gossip apply in Slice 3R.**
- Build commit created and pushed to Jim's fork:
  - `779835028dae3324a20534f07f0402c47e6d6fe8` — `fix(adr-0016-phase-1): remediate Slice 3R last-admin leave`
- Code review passed; verifier passed; PR #5 CI passed at `7798350`.
- Craft Review passed.
- Adversarial confirmation returned **NOT-READY** with one HIGH blocker.

## R2 decision recorded for checkpoint / PR note

Jim's R2 rationale to carry forward:

- `Moderator` (rank 2) and `Guest` (rank 0) grant **no authority** — the only authority threshold in the code is `at_least(Admin)`; both sit below it and appear in no privilege check. An applied Moderator/Guest is an inert label, not an escalation. (`Owner`, the one role that does grant authority, is admin-equivalent and already handled.)
- Authority comes from the signed commit + the `at_least(Admin)` check + the last-admin invariant — not from policing target-role vocabulary at apply time.
- In a gossip network the apply path must accept any validly-signed peer commit, including from daemons still on the old version. Rejecting on apply would (a) break byte-for-byte legacy replay and (b) fork live state between upgraded and un-upgraded daemons. The admin/member-only **assignment** rule belongs at **authoring**, not apply.

Adversarial follow-up found the first bullet is currently over-broad because active `Guest` has member-level authority in existing code paths; see blocker below.

## Evidence

CI arbiter / green of record:

- Location: draft mirror PR #5, <https://github.com/JimCollinson/x0x/pull/5>
- Status: **green at `779835028dae3324a20534f07f0402c47e6d6fe8`**. All reported checks passed; Soak Test skipped by workflow.

Local fast gate / `.gsd/gate.sh`:

- Installed? Clone-local pre-push hook ran on push.
- Commands run by pre-push hook:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- Result: PASS.

Files changed in build commit:

- `src/bin/x0xd.rs`
- `src/groups/mod.rs`
- `tests/last_admin_invariant.rs`
- `tests/membership_authority.rs`

Checks run / results:

- `cargo fmt --all` — PASS (operative mandatory order)
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS
- `cargo check --workspace --all-targets` — PASS
- `cargo nextest run --all-features --test membership_authority` — PASS, 14/14
- `cargo nextest run --all-features --test last_admin_invariant` — PASS, 12/12 (codereviewer/verifier)
- `cargo nextest run --all-features --test owner_retirement -E 'test(owner_retirement_role_assignment_accepts_only_admin_member_and_exact_errors)'` — PASS, 1/1 (verifier)
- `cargo nextest run --all-features -E 'test(membership_authority_non_creator_last_admin_self_leave) or test(membership_authority_signed_role_update_apply_accepts_current_and_legacy_labels) or test(last_admin_gossip_apply_rejects_owner_demoted_to_reserved_low_roles)'` — PASS, 3/3
- `cargo nextest run --all-features -E 'test(last_admin) or test(role) or test(member)'` — local FAIL only in `named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected` with macOS mesh `zero peers after 30s`; same failure reproduced at baseline `6ebac93` and prior baseline evidence exists in `gsd/evidence/2026-06-13-slice-1-local-gate.md`.

## Honesty rules check

- No-harness-modification: Pass.
  - No `.gsd/gate.sh`, CI workflow, test harness, daemon wrapper, build invocation, environment setup, serde names, role bytes, hashing, signing, commit format, or storage format changes.
- Baseline-diff for evidence: Pass with caveat.
  - Broad-filter local mesh failure was baseline-classified at `6ebac93` and not used as passing readiness evidence.
- Evidence reproducible-from-branch: Pass.
  - Build commit was pushed to fork; PR #5 CI is green of record.
- Local vs CI consistency: Local deterministic checks and CI pass; local macOS mesh setup failure is baseline/environmental and CI Linux passed.

## Review findings

Clean-context test:

- Reviewer/tool: Not run.
- Result: Not run.
- Findings: Deferred to end-of-phase / PR-readiness per Phase 1 plan; Slice 3R is a focused remediation, not the complete user-facing feature.

Code review:

- Reviewer/tool: `codereviewer` subagent.
- Result: passed.
- Findings: no blocker/high/medium/low/nit findings; confirmed CI still needed after push, then CI passed.

Verifier:

- Reviewer/tool: `verifier` subagent.
- Result: passed.
- Findings: functional 5/5 verification passed; note filed as `gsd/checkpoints/2026-06-14-slice-3r-verification.md`.

Adversarial review:

- Reviewer/tool: `adversarial` subagent, model reported `openai/gpt-5.5`.
- Required? Yes — meaningful work-unit, per-slice cadence.
- Result: **NOT-READY**.
- Findings:
  - **HIGH — R2 not closed: current daemon can still author an active `Guest`, and active `Guest` gets member-level authority.** Evidence cited:
    - `src/groups/mod.rs:823-845` inserts absent ban targets as `Guest` tombstones.
    - `src/bin/x0xd.rs:12287-12327` can ban targets without requiring existing membership.
    - `src/bin/x0xd.rs:12579-12590` unbans banned targets.
    - `src/groups/mod.rs:848-856` unban reactivates without changing `Guest` role.
    - `src/groups/mod.rs:883-887`, `src/bin/x0xd.rs:10147-10151`, and `src/groups/state_commit.rs:599-604` give active roles member-level access/self-action semantics.
  - MEDIUM — R1 regression test remains helper/seam coverage, not production daemon handler coverage.
  - MEDIUM — planning packet/addendum contradicted Jim's later R2 decision; this checkpoint and the paired planning edits record the decision.
  - LOW — TreeKEM last-admin self-leave still rejects via seal failure/500 rather than the §3 409, although clone-first means no corruption.

Craft Review:

- Reviewer/tool: `craft` subagent.
- Verdict: Pass.
- CONFORMANCE findings and dispositions: none.
- SIMPLICITY / NIT findings carried: none.
- Suggested PR note: keep the Moderator/Guest signed-apply replay exception explicit.

## Current blocker for Jim

The Slice 3R code fixed the original R1 corruption path and implemented Jim's R2 accept-and-document decision for `MemberRoleUpdated` apply. However, adversarial found a new active-`Guest` authoring/authority path outside `MemberRoleUpdated`:

1. banning an absent target creates a banned `Guest` tombstone;
2. unbanning reactivates it as `Guest`;
3. active `Guest` is treated as an active member for member-level access/self-action checks.

This contradicts the R2 rationale if stated as “Guest grants no authority” unless member-level authority is explicitly excluded from “authority”. It also means current-code authoring can create an active reserved role through ban/unban.

## Decision needed

Pick a remediation direction before Slice 4:

1. **Make `Guest` truly inert:** member-level checks require `role.at_least(Member)` instead of any active role; add tests for active `Guest` no member-write/self-action authority. This may have broader semantic blast radius and needs careful review.
2. **Prevent active `Guest` authoring:** change ban/unban semantics so an absent banned tombstone cannot become an active `Guest` (for example, unban removes absent-target tombstones or restores only prior members); add tests for ban-absent → unban.
3. **Accept member-level `Guest` semantics:** explicitly revise the R2 rationale / PR note to say Moderator/Guest grant no admin authority but active Guest may have member-level participation if created by legacy/ban flows; then decide whether active reserved-role authoring is acceptable.

Recommended next step: choose option 2 if the desired invariant is “current code must not create active reserved roles” with minimal impact; choose option 1 only if `Guest` must be globally inert everywhere.

## Recommended next step

Stop here. Do not dispatch Slice 4 until the adversarial HIGH is fixed or explicitly accepted by Jim.
