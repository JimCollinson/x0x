# Checkpoint — Slice 3R final disposition (ADR-0016 Phase 1)

- Date: 2026-06-16
- Slice/question: Slice 3R final disposition after reserved-role / active-Guest retro findings
- Feature branch/head at start: `feat/adr-0016-phase-1-authority-alignment` @ `c7e91b2`
- Disposition: **accepted as pre-existing/out-of-scope; revert `c7e91b2`; keep `7798350`; proceed to Slice 4 after post-revert CI is green.**
- Supersedes: the open active-Guest blocker status in `gsd/checkpoints/2026-06-14-slice-3r-retro-remediation.md` and the failed-fix gap in `gsd/checkpoints/2026-06-16-slice-3r-active-guest-verification.md`.

## Status

Slice 3R is dispositioned as follows:

- `7798350` remains the Slice 3R code fix for the non-creator last-admin self-leave corruption path.
- `c7e91b2` was a follow-up attempt to change ban/unban tombstone behavior; it failed review/CI and is being reverted rather than folded into the PR.
- The active-`Guest` ban-tombstone behavior is accepted as pre-existing upstream behavior and out of scope for ADR-0016 Phase 1 Slice 3R.
- Reserved `Moderator` / `Guest` signed apply remains accepted for legacy replay/convergence; current assignment remains gated at authoring.
- Post-revert tree is byte-for-byte identical to the previously green `7798350` tree, but PR #5 is still not green at `449ac80` because the existing daemon-health test `named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected` failed twice after x0xd startup did not become healthy within 90s. Slice 4 remains blocked until PR #5 is actually green.

## Rationale

**Context.** During the Slice 1–3 retro review, two reserved-role observations surfaced. This records the decisions and security rationale.

**1. Reserved-role assignment on the signed apply path.** REST role assignment is restricted to `admin`/`member` (`GroupRole::assignable_from_name`), but the signed `MemberRoleUpdated` apply path rejects only `Owner` — so a validly-signed commit can carry `Moderator`/`Guest`. **Decision:** enforce the admin/member-only vocabulary at **authoring** (REST); **accept** `Moderator`/`Guest` on the apply path. **Why:** (a) they grant **no admin authority** — the only authority threshold is `role.at_least(Admin)`; they rank below it and appear in no privilege check; `Owner` (the admin-equivalent one) is already rejected on apply. (b) An active member of *any* role is **member-level** (self-actions check membership, not role) — not an escalation, and only an admin can author such a commit anyway. (c) The apply path must accept any validly-signed peer commit, including from older daemons, or daemons diverge; vocabulary belongs at assignment, not receipt. Rejecting on apply would break byte-for-byte legacy replay **and** fork live state across versions, for no security gain.

**2. Active "Guest" from a ban tombstone — pre-existing, left unchanged.** Banning an agent who is **not** a member inserts a `Guest`/`Banned` placeholder; unbanning reactivates an active `Guest` (member-level); a repeated ban→unban cycle can recreate it. **Provenance:** pre-existing upstream behaviour in `GroupInfo::ban_member` (authored 2026-04, commit `ba965266`, present unchanged at base `189b89c`) — a defensive `or_insert` fallback, **not** a documented or tested feature (no API-reference entry, no test). ADR-0016 did **not** introduce it or change its reachability. **Decision:** leave the maintainer's ban/unban code unchanged; flag it to him as a separate pre-existing item. **Why it's safe to accept here:** triggering it requires **admin** rights on every path (REST and signed apply both gate ban/unban on `at_least(Admin)`); the placeholder is **member-level, not admin**, and carries **no KEM key** — it never received the group secret and cannot decrypt anything; it is deterministic (no convergence break). Worst case: a cosmetic roster entry an admin could have created by adding a member anyway. Security framing assumes the ADR-0016 model where **Admin is root for the group** — a compromised admin can already admit/remove/rekey/end the group, so an admin-only, keyless artifact adds nothing to that threat.

**Disposition.** Revert `c7e91b2`; keep `7798350`; do not touch the maintainer's ban code; correct "inert" wording to "no admin authority / member-level"; flag the ban-tombstone to the maintainer (a clean optional direction for him: make banning a non-member a no-op). The prior fix attempt (`c7e91b2`) added fragile heuristic detection and altered a hash-bearing field on every ban, which broke CI — confirming this is the maintainer's call, not something to bolt onto this PR.

## Wording verification

The earlier over-broad wording has already been corrected in current planning notes. Remaining mentions of “inert” / “no authority” are historical or meta wording describing the overclaim, not live claims. The intended PR/checkpoint wording is: **no admin authority; an active member of any role is member-level.**

## Post-revert verification evidence

- Revert commit: `449ac8077dc55d7a91f9aa1acaaf6f992cc96ca7` — `Revert "fix(adr-0016-phase-1): prevent active guest from ban tombstones"`.
- Tree restoration check: `git diff --quiet 779835028dae3324a20534f07f0402c47e6d6fe8` returned `0`, so the post-revert source tree at `449ac80` matches the previously green `7798350` tree.
- Narrow restoration check: `git diff 7798350 -- src/groups/mod.rs tests/membership_authority.rs` produced no diff.
- `git grep is_never_member_ban_tombstone`: no matches.
- Local mandatory checks at `449ac80`: PASS.
  - `cargo fmt --all`
  - `cargo clippy --all-features --all-targets -- -D warnings`
  - `cargo check --workspace --all-targets`
- Local targeted checks at `449ac80`: PASS.
  - `cargo nextest run --all-features --test membership_authority` — 14/14.
  - `cargo nextest run --all-features -E 'test(membership_authority_non_creator_last_admin_self_leave) or test(membership_authority_signed_role_update_apply_accepts_current_and_legacy_labels) or test(last_admin_gossip_apply_rejects_owner_demoted_to_reserved_low_roles)'` — 3/3.
- Push evidence: pushed `449ac80` to Jim's fork; clone-local pre-push gate ran `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features -- -D warnings`, both PASS.
- PR #5 CI at post-revert head `449ac80`: **not green yet**.
  - First observed failure: run `27608164487`, job `81625359547`, `Test Suite`, `named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected`, `x0xd pair-alice-6444 did not become healthy within 90s`.
  - Failed-job rerun: same workflow run, job `81627981660`, `Test Suite`, same test, `x0xd pair-alice-23248 did not become healthy within 90s`.
  - All other PR #5 checks reported pass; Soak Test skipped by workflow.

## Current blocker

PR #5 is red at `449ac80`; do not dispatch Slice 4 until the CI arbiter is green. Because `449ac80`'s source tree matches the previously green `7798350` tree exactly, the repeated failure is classified as the known daemon startup / mesh-health flake class, not a Slice 3R code delta. Green of record is still absent until PR #5 reports all required checks passing at the post-revert head.

## PR-note location

Maintainer-facing notes are recorded in `gsd/plan/phase-1-pr-notes.md`.
