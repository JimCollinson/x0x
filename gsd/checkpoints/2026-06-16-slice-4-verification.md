# Verification — Slice 4 invites per issuer + creator provenance (ADR-0016 Phase 1)

- Date: 2026-06-16
- Verifier: OpenAI GPT-5.5 verifier
- Build worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-build`
- Planning worktree: `/Users/jimcollinson/code/x0x/.claude/worktrees/adr0016-planning`
- Feature head verified: `4fabfccada662d43719c7da71dd1d8818ccb5157`
- Full Slice 4 delta: `449ac8077dc55d7a91f9aa1acaaf6f992cc96ca7..4fabfccada662d43719c7da71dd1d8818ccb5157`
- Remediation delta: `680198b38c55c380bafc8adc3da1ac0a0b2f5607..4fabfccada662d43719c7da71dd1d8818ccb5157`
- Status: **passed**
- Supersession note: later adversarial review returned **NOT-READY** with a HIGH state-hash/roster coherence finding. This verification note is preserved as historical goal-backward evidence, not final Slice 4 acceptance.

## Goal-backward result

Score: **8/8 goals verified**.

1. **Any active Admin can issue invites; plain members are rejected — VERIFIED.**
   - `create_group_invite` now uses `require_admin_or_above(info, &inviter_hex)` and no longer checks `agent_id != info.creator`.
   - `tests/invite_authority.rs` covers promoted non-creator Admin success, creator success, and plain member rejection without tracking a secret.

2. **Invite issue/consume/track remains per issuing daemon; inviter remains join-result polling/routing target — VERIFIED.**
   - Invite creation still records `info.record_issued_invite(...)` on the local issuer's `GroupInfo`.
   - `MemberJoined` consume path still accepts only on `local_is_inviter` and checks the inviter role.
   - Join-result response validation now requires `sender_hex == inviter_agent_id`, not creator, and polling sends fetch requests to parsed `invite.inviter`.

3. **Joiner `GroupInfo.creator` derives from genesis/base state, never unsigned invite metadata — VERIFIED.**
   - `SignedInvite::creator_agent_id_from_base_state()` derives creator from `base_members_v2` seeded entry (`added_by == None`, preferring `group_created_at`) and errors instead of falling back to `invite.inviter`.
   - `join_group_via_invite` parses `creator_hex` from that helper, then keeps parsed `inviter` separate for routing.

4. **Non-TreeKEM non-creator-admin convergence bug reproduced and fixed with meaningful tests — VERIFIED.**
   - Pre-remediation `680198b` seeded `base_members_v2`/`base_state_hash` only inside `if invite_is_treekem`, while non-TreeKEM recomputed state after adding the joiner locally.
   - Remediation hoists invite base-state seeding into `invite_join_group_info` for all invite joins and recomputes only when `invite.base_state_hash.is_none()`.
   - New regression `non_treekem_admin_invite_joiner_validates_member_added_state_chain` validates creator, inviter, and joiner converge to the same state hash/revision after a non-creator Admin's `MemberAdded`.

5. **TreeKEM behavior remains compatible/safe — VERIFIED.**
   - `invite_join_group_info` preserves `shared_secret = None` for TreeKEM, uses base state hash/revision, and does not add the joiner locally before Welcome/authoritative result.
   - `treekem_invite_stub_matches_authority_base_hash` passes.
   - Join-result polling still targets `invite.inviter`.

6. **Remaining `creator` comparisons are classified outside invite authority — VERIFIED.**
   - Remaining matches are public-card provenance (`sender_hex != creator_hex`), API output fields, invite provenance helper, Slice 5 leave/delete behavior, TreeKEM leave-disposition tests, and a join-request direct-notification placeholder. No remaining invite issue-side creator authority gate found.

7. **Forbidden areas untouched — VERIFIED.**
   - Full Slice 4 diff touches only `src/bin/x0xd.rs`, `src/groups/invite.rs`, and `tests/invite_authority.rs`.
   - Hunk headers are limited to invite issue/join, MemberJoined consume wording, join-result sender check, invite helper/tests. No changes under `tests/harness/`, CI, daemon wrappers, startup/health, network/bootstrap/presence modules, role bytes, commit format, storage format, or signing/hash primitives.
   - Invite struct gained serde-default metadata fields already present in the Slice 4 delta for base-state provenance; no serde names/role-byte/storage format changes were found.

8. **PR #5 internal CI arbiter satisfied under approved daemon-startup timeout carve-out — VERIFIED WITH CAVEAT.**
   - PR #5 head is `4fabfccada662d43719c7da71dd1d8818ccb5157`.
   - All checks pass except `Test Suite` and `Multi-Agent Integration`; both latest failures are isolated daemon startup health timeouts:
     - `named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected`: `x0xd pair-alice-54818 did not become healthy within 90s`.
     - `named_group_integration::named_group_creator_delete_propagates_to_peer`: `x0xd pair-bob-59792 did not become healthy within 90s`.
   - This is weaker than clean CI because the GitHub UI remains red, but it satisfies `gsd/ci-arbiter.md` as the approved internal green-of-record carve-out for this slice.

## Commands/evidence run or inspected

- `git status --short && git rev-parse HEAD && git log --oneline --decorate -5` — clean build worktree at `4fabfcc`.
- `git diff --stat 449ac8077dc55d7a91f9aa1acaaf6f992cc96ca7..4fabfccada662d43719c7da71dd1d8818ccb5157` — only `src/bin/x0xd.rs`, `src/groups/invite.rs`, `tests/invite_authority.rs`.
- `git diff --stat 680198b38c55c380bafc8adc3da1ac0a0b2f5607..4fabfccada662d43719c7da71dd1d8818ccb5157` — remediation only `src/bin/x0xd.rs`, `src/groups/invite.rs`.
- `git diff --check 449ac8077dc55d7a91f9aa1acaaf6f992cc96ca7..4fabfccada662d43719c7da71dd1d8818ccb5157` — PASS.
- `cargo fmt --all -- --check` — PASS.
- `cargo clippy --all-features --all-targets -- -D warnings` — PASS.
- `cargo check --workspace --all-targets` — PASS.
- `cargo nextest run --all-features --test invite_authority` — PASS, 3/3.
- `cargo nextest run --all-features --all-targets -E 'test(non_treekem_admin_invite_joiner_validates_member_added_state_chain) or test(treekem_invite_stub_matches_authority_base_hash) or test(creator_provenance)'` — PASS, 5/5.
- `cargo nextest run --all-features -E 'test(invite) & !binary(named_group_join_metadata_event)'` — PASS, 23/23.
- `gh pr view 5 --repo JimCollinson/x0x --json headRefName,headRefOid,statusCheckRollup,url,isDraft,title` — PR head verified at `4fabfcc`; checks inspected.
- `gh run view 27622938623 --job 81680553773 --repo JimCollinson/x0x --log | rg ...` — failing Test Suite log matches daemon-startup timeout signature.
- `gh run view 27622939039 --job 81680551200 --repo JimCollinson/x0x --log | rg ...` — failing Multi-Agent Integration log matches daemon-startup timeout signature.

## Remaining caveat

The internal CI arbiter is satisfied only via the documented daemon-startup timeout carve-out; GitHub's PR checks are not visually all green. Maintainer/upstream CI evidence is therefore weaker until the startup-health flake is cleared or rerun green.

## Recommendation

Proceed to the next phase/slice from a Slice 4 goal-verification standpoint, while carrying the CI-carve-out caveat and any separately-required clean-context/adversarial/Craft gates according to the GSD plan.
