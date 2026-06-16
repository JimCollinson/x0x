# Verification — Slice 3R active-Guest remediation

- Date: 2026-06-16
- Verifier: GPT-5.5
- Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
- Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
- Remediation delta checked: `779835028dae3324a20534f07f0402c47e6d6fe8..c7e91b2`
- Full Slice 3R delta checked: `6ebac93e423e3fab60f91481adad6a86fb212445..c7e91b2`
- Status: **gaps_found**
- Supersession note: final disposition is recorded in `gsd/checkpoints/2026-06-16-slice-3r-disposition.md`. This verification note is preserved as evidence that the `c7e91b2` fix attempt was reviewed and found insufficient before it was reverted.

## Goal-backward result

The single-cycle absent-ban -> unban path is fixed, and legitimate active `Guest` remains active/member-level in the covered GSS path. However, the goal "ban/unban can no longer fabricate an active member from a never-member tombstone" is not fully achieved: a never-member tombstone can be unbanned to `Removed`, then banned again, then unbanned into an active `Guest`.

Static path:

1. First absent ban inserts a `Guest`/`Banned` record with `joined_at == updated_at` and no membership metadata (`src/groups/mod.rs:823-852`).
2. First unban detects that exact tombstone and sets it to `Removed` (`src/groups/mod.rs:855-881`).
3. The `Removed` record remains in `members_v2`, with role `Guest` and no membership metadata.
4. A second ban uses the existing-entry `and_modify` arm (`src/groups/mod.rs:827-839`). The tombstone predicate is false because the record is `Removed` before the ban, and the code forces `updated_at >= joined_at + 1` (`src/groups/mod.rs:833-837`).
5. A second unban no longer matches `is_never_member_ban_tombstone` because `joined_at != updated_at`, so it takes the `Active` branch (`src/groups/mod.rs:875-880`). That fabricates an active `Guest` from a never-member record.

## Verification questions

1. **No active Guest from never-member tombstone:** **Gap.** Single-cycle path fixed, repeated ban/unban still fabricates active `Guest` as above.
2. **Legitimate legacy active Guest remains member-level:** **Verified for covered GSS/core path.** Tests assert active `Guest` stays active/member-level and below Admin (`tests/membership_authority.rs:408-431`, `:434-457`); `GroupRole::Guest` rank is below Admin (`src/groups/member.rs:27-58`); admin checks require `at_least(Admin)` (`src/bin/x0xd.rs:11986-11994`).
3. **Moderator/Guest signed apply not rejected; Owner-on-apply unchanged:** **Verified.** Daemon role-update apply rejects only `Owner` (`src/bin/x0xd.rs:8646-8671`); shared signed-apply authority check is role-threshold only (`src/groups/state_commit.rs:592-604`); test asserts `Admin`, `Member`, `Moderator`, and `Guest` apply (`tests/membership_authority.rs:537-542`).
4. **Tests meaningful / adversarial:** **Partial.** The new tests would fail against the original single-cycle adversarial path (`tests/membership_authority.rs:384-404`; `src/groups/mod.rs:1124-1139`), but they do not test the repeated never-member tombstone cycle.
5. **Scope / forbidden mechanisms:** **Verified.** Remediation delta changes only `src/groups/mod.rs` and `tests/membership_authority.rs`; no `.gsd/gate.sh`, CI workflow, test harness, daemon wrapper, build invocation, or environment setup changes found.
6. **Validation / CI:** **Not sufficient yet.** Local targeted tests passed, but PR #5 CI at head `c7e91b26eb067bd56d167bc3d76dadd7971ffa47` still had required checks in progress when inspected; CI was not green of record.

## Local checks run by verifier

- `cargo nextest run --all-features --test membership_authority -E 'test(membership_authority_ban_absent_then_unban_does_not_create_active_member) or test(membership_authority_legacy_guest_remains_member_level_after_ban_unban) or test(membership_authority_legacy_guest_without_add_metadata_remains_member_level_after_ban_unban) or test(membership_authority_signed_role_update_apply_accepts_current_and_legacy_labels)'` — PASS, 4/4.
- `cargo test --lib ban_absent_then_unban_does_not_create_member` — PASS, 1/1.
- `cargo test --lib ban_unban_preserves` — PASS, 2/2.
- One attempted `cargo test --lib <three separate test names>` command failed due invalid cargo CLI usage, not a test failure.

## Recommendation

Remediate before Slice 4: ensure a never-member tombstone remains identifiable or is removed across repeated ban/unban cycles, add a normal-gate test for repeated absent ban/unban, then rerun mandatory checks and wait for PR #5 CI green at the remediation head.
