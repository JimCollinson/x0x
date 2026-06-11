# Plan — Admin invite authority (#107 item a)

Working file for the `feat/admin-invite-authority` branch. Not part of the PR;
deleted before the branch is handed over for final review.

- Base commit: `5f086cf` ("ci(release): atomic version bump tool to end SKILL.md version_sync drift") — verified equal to `saorsa-labs/x0x` `main` head at branch time (2026-06-11).
- Contract: issue #107 body + dirvine triage comment (2026-06-10T15:54Z, only comment on the thread). Scope is item (a) only.
- Acceptance: A creates `private_secure` → A invites B → A promotes B to admin (converge) → **B issues invite → C joins → A/B/C converge with C Active**; a plain Member's invite attempt rejected with a **role-based** error.

## Pre-flight

- [x] Check zero — fork `main` synced with upstream (`5f086cf` == upstream head; fork main was force-synced, local ref refreshed)
- [x] ~~Check one~~ — **FAILED. Multi-daemon gossip peering is structurally impossible in this sandbox** (see "Check one verdict" below). Per the work package: build falls back to a local machine.
- [x] Read issue #107 in full (body + all comments)
- [x] Verify implementation pointers against post-#108 checkout

## Check one verdict (2026-06-11, sandbox)

`named_group_creator_delete_propagates_to_peer` run as CI runs it
(`cargo nextest run --all-features --test named_group_integration
--run-ignored ignored-only`) fails deterministically at the harness's own
mesh gate: `[cluster] FATAL: pair-alice has zero peers after 30s`.
Reproduced twice via nextest and twice manually with the **release** x0xd
(sandboxed and unsandboxed shells, 60–75 s waits) — not a flake, not the
debug/dhat slowdown.

Root cause: the sandbox container's only non-loopback interface is
`192.0.2.2` — inside RFC 5737 TEST-NET-1 (documentation range). ant-quic
promotes the harness's `127.0.0.1:<port>` bootstrap target to that
interface address, the QUIC handshake completes (both daemons log
successful NAT-traversal/address-discovery negotiation, mDNS sees the
peer), but ant-quic's bogon filtering rejects documentation-range
addresses at three layers (`candidate_discovery.rs` excludes
`is_documentation()` local candidates; `connection/nat_traversal.rs:829`
drops documentation-range peer candidates; `port_mapping.rs` same), so the
peer never enters gossip membership: `connected_peers=0` forever, on every
daemon. CI passes because GitHub runners have ordinary private addresses
that pass the filters.

No invocation tweak fixes this (it is the container's addressing scheme
vs. deliberate transport policy), and patching ant-quic's filter locally
would mean validating on a modified transport — degraded mode, forbidden.
Everything that does not require daemon-to-daemon convergence (fmt,
clippy, check, unit tests, doc build, single-daemon ignored tests) still
works here; anything convergence-shaped must run on the local machine.

## Findings from code reading (what the fix actually is)

The invite machinery is already per-issuer end to end:
- `SignedInvite.inviter` records the actual issuer; `record_issued_invite` stores the one-time secret on the issuing daemon (`src/bin/x0xd.rs:10443-10471`).
- The joiner routes `MemberJoined` to `invite.inviter` directly + via gossip (`x0xd.rs:10698-10726`); non-inviter receivers deliberately ignore it (`x0xd.rs:9307-9316`).
- Consume already re-checks the **current** role of the inviter at consume time (`x0xd.rs:9317-9327`) — a demoted issuer's outstanding invite already dies.
- `validate_apply` authority is purely role-based via `members_v2` (`src/groups/state_commit.rs:563-583`); commit signatures bind `committed_by` to the signer key (`verify_structure`).

Three creator gates block delegated invites (all predate the role model):
1. `create_group_invite` — the literal 403 gate (`x0xd.rs:10436`).
2. `apply_named_group_metadata_event_inner`, `MemberAdded` arm — receivers drop any `MemberAdded` whose gossip sender isn't the local `info.creator` (`x0xd.rs:8037`). Without fixing this, A and C silently drop B's authoritative add and the acceptance story cannot converge.
3. `authorized_treekem_membership_event_for_queue`, `MemberAdded` arm — same creator equality in the out-of-order-queue authorizer (`x0xd.rs:7385`); without it admin-authored adds get dropped instead of queued.

The `MemberBanned` arm in both places already uses the newer role-based idiom
(`actor == sender && caller_role(actor) >= Admin` + owner-target protection) —
that is the pattern to adopt (CLAUDE.md Rule 7: prefer the newer, role-model
pattern over the pre-role-model creator equality).

One correctness gap introduced by delegation: `join_group_via_invite` seeds the
joiner's `GroupInfo.creator` **and** genesis `creator_agent_id` from
`invite.inviter` (`x0xd.rs:10565-10583`). With an admin-issued invite, C would
record B as group creator — wrong display in `GET /groups`, wrong genesis
record, and C would then mis-validate every *creator-gated* event from the real
creator A (MemberRemoved/GroupDeleted/PolicyUpdated/GroupCardPublished arms all
compare against local `info.creator`). The invite must carry the true creator.

## Implementation checklist

- [ ] 1. `create_group_invite`: replace the creator equality with `require_admin_or_above(info, &caller_hex)` (same helper the ban path uses). Error becomes 403 `admin role required`.
- [ ] 2. `SignedInvite`: add `#[serde(default)] pub group_creator: Option<String>` (hex agent id of the group creator). Set from `info.creator` in `create_group_invite`. Backward compatible: old invites/links deserialize with `None`.
- [ ] 3. `join_group_via_invite`: seed `GroupInfo` creator + genesis from `invite.group_creator`, falling back to `invite.inviter` (legacy invites — identical to today's behavior). `invite.inviter` keeps its routing/consume role untouched.
- [ ] 4. `apply_named_group_metadata_event_inner` `MemberAdded` arm: replace `sender_hex != creator_hex || actor != sender_hex` with the `MemberBanned` idiom — `actor == sender_hex && caller_role(actor) >= Admin`, plus `actor == commit.committed_by` (idiom from the `GroupDeleted` arm) so event attribution matches the signed chain. `validate_apply(AdminOrHigher)` continues to enforce chain authority.
- [ ] 5. `authorized_treekem_membership_event_for_queue` `MemberAdded` arm: same role-based form (mirror its own `MemberBanned` arm).
- [ ] 6. Out of scope, untouched: `MemberRemoved` arms + remove handlers (item c), KeyPackage distribution (item b), ownership transfer (item d), direct member-add endpoints (`add_named_group_member`, `add_treekem_named_group_member`), `GroupDeleted`/`PolicyUpdated`/`GroupCardPublished` creator gates.

## Tests

- [ ] 7. Integration (tests/named_group_integration.rs, `#[ignore]`, daemon harness):
  - [ ] a. Acceptance story (3 daemons via `trio_with_extra_config("")`): create `private_secure` → invite B → promote B to admin → converge → **B invites C → C joins → A/B/C state hashes converge, C Active on all three** (assert TreeKEM `security_binding` so the secure plane is actually exercised).
  - [ ] b. Negative: plain Member B `POST /groups/:id/invite` → 403, error `admin role required` (role-based, not creator-based).
  - [ ] c. Demoted issuer: B (admin) issues invite → A demotes B to member → both observe demotion → C joins via B's invite → C never becomes Active on any daemon (bounded wait), B's daemon refuses the consume.
- [ ] 8. Unit (x0xd.rs tests mod): `authorized_treekem_membership_event_for_queue` accepts admin-authored `MemberAdded` and rejects member-authored (pure function, mirrors existing classifier tests).
- [ ] 9. Existing tests: re-run full named-group suites; `named_group_invite_join_preserves_genesis_creation_nonce` must still pass (creator-issued invites take the `group_creator == inviter` path → behavior identical).

## Docs

- [ ] 10. CHANGELOG `[Unreleased]` → Fixed: admin invite authority, referencing #107 item (a). (api-reference makes no creator-only claim — verified — so no doc change there beyond the endpoint note if reviewers want one.)

## Gates (all green before handover, with counts)

- [ ] 11. `cargo fmt --all -- --check`
- [ ] 12. `cargo clippy --all-features --all-targets -- -D warnings`
- [ ] 13. `cargo check --workspace --all-targets`
- [ ] 14. `cargo nextest run --all-features --workspace` (non-ignored)
- [ ] 15. Ignored suites exactly as CI: `daemon_api_integration -- --ignored`; `named_group_integration --run-ignored ignored-only`; `named_group_d4_apply -- --ignored`; `gui_smoke`; `ws_integration -- --ignored`; `kv_first_join_bootstrap --test-threads 1 -- --ignored`; `local_topics --test-threads 1 -- --ignored`
- [ ] 16. `RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps`
- [ ] 17. Never stage `audit.jsonl`; no personal paths/identity values anywhere
- [ ] 18. Rebase on upstream main before handover; conventional commits; push `-u origin feat/admin-invite-authority`

## Judgment calls (confirmed defaults)

1. **Receive-path creator gates are in scope** (items 4–5 above). The literal task says "drop the creator gate" in `create_group_invite`, but the acceptance story ("A, B, C all converge") is impossible without aligning the `MemberAdded` receive/queue path to the same `AdminOrHigher` rule the chain already enforces. Uses only the idiom the neighbouring `MemberBanned` arm already established. `MemberRemoved` stays creator-gated (item c).
2. **Invite carries the true group creator** (items 2–3). New optional `SignedInvite.group_creator` field, fallback to `inviter` for legacy links. Without it, joiners via admin invites record the wrong creator and mis-validate later creator-gated events.
3. **Moderators may NOT invite** — state chain says membership adds are `AdminOrHigher`; `require_admin_or_above` enforces exactly that. (Not configurable; matches contract.)
4. **Demoted/banned issuer** — no new code: consume-time role re-check already exists (`x0xd.rs:9317-9327`). Inherent gossip race remains: if the join is consumed before the demotion commit reaches the issuer's daemon, the add succeeds (then converges normally) — same eventual-consistency window the creator path has today. Lock the deterministic case in with test 7c.
5. **`actor == commit.committed_by` added to the MemberAdded arm** — minor tightening beyond strict parity with `MemberBanned`, borrowed from the `GroupDeleted` arm, so the event's claimed actor cannot diverge from the signed chain author. Cheap, and the adversarial review stage would flag its absence.

## Review gauntlet (after build)

- [ ] Fresh clean-context Claude review (names commit hash)
- [ ] Fresh adversarial review — Codex GPT 5.5 (names commit hash)
- [ ] Local verification run on Jim's Mac (full gates + live acceptance flow)
- [ ] Jim final gate → PR `JimCollinson:feat/admin-invite-authority` → `saorsa-labs:main`, "Addresses #107 (item a)" (NOT "Closes"), request dirvine review, append PR number to CHANGELOG entry
