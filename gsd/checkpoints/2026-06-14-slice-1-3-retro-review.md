# Checkpoint — Slice 1-3 retro adversarial + craft review (ADR-0016 Phase 1)

- Date: 2026-06-14
- Scope: Slices 1-3 cumulative diff, `189b89c..6ebac93` on `feat/adr-0016-phase-1-authority-alignment`
- Feature head reviewed: `6ebac93e423e3fab60f91481adad6a86fb212445`
- Planning change paired with this retro: `gsd/plan/phase-1-plan.md` amended so substantial slices require per-slice adversarial + craft review before acceptance; end-of-phase gauntlet now adds clean-context plus integrated adversarial/craft sweep.
- Status: **Blocked before Slice 4.** Retro adversarial returned NOT-READY with HIGH findings; retro craft returned one CONFORMANCE finding requiring fix or Jim acceptance.

## Review cadence amendment filed

`gsd/plan/phase-1-plan.md` was amended per Jim's approved cadence note:

- Universal slice preamble now includes independent adversarial + craft review before accepting each substantial slice.
- Plan summary now says substantial slices are independently reviewed at checkpoints, with end-of-phase clean-context plus integrated adversarial/craft sweep over the composed branch.
- Post-slice step 2 is now the end-of-phase gauntlet, not the only adversarial gate.

## Retro reviews run

### Adversarial retro

- Reviewer/tool: `adversarial` subagent, model reported `openai/gpt-5.5`.
- Independence note: Implementers/checkpoints identify Claude Code/OpenCode; exact Slice 3 model was not recorded, so reviewer labelled independence good for Claude-authored work but weaker for Slice 3 if OpenCode used an OpenAI model.
- Result: **NOT-READY**.
- Gate impact: **blocks Slice 4** until HIGH findings are fixed or explicitly accepted by Jim.

#### HIGH — Slice 3 makes a last-admin self-leave corruption path reachable before Slice 5

Finding summary:

- Slice 3 allows any admin to remove the creator if another admin remains.
- `DELETE /groups/:id` for a non-creator GSS/non-TreeKEM group still mutates the live `GroupInfo` before `seal_commit` and has no last-admin precheck.
- `seal_commit` rejects the zero-admin state, but by then the in-memory group has already been mutated.

Anchors cited by reviewer:

- `src/bin/x0xd.rs:11309-11324` — Slice 3 role-based remove path allows admin remove and uses last-admin precheck for remove-member.
- `src/bin/x0xd.rs:11838-11877` — non-creator `DELETE /groups/:id` self-leave path mutates `info` before `seal_commit` and lacks last-admin precheck.
- `src/groups/mod.rs:441-450` — `seal_commit` enforces last-admin invariant after caller mutations are already in `self`.

Reviewer's concrete reachable sequence:

1. Creator A creates a non-TreeKEM/GSS group.
2. A promotes B to Admin.
3. B removes A. This now succeeds after Slice 3.
4. B is now sole admin but not `info.creator`.
5. B calls `DELETE /groups/:id`.
6. Handler removes B from `info` before `seal_commit`.
7. `seal_commit` rejects, response is 500, but live in-memory group is now non-withdrawn with zero active admins.

Disposition required:

- Fix before Slice 4, or Jim explicitly accepts the risk. Recommended fix: add clone-first/precheck for `leave_group` now, or otherwise prove the path unreachable.

#### HIGH — Reserved roles can still be assigned through signed gossip/state-commit path

Finding summary:

- REST parsing restricts role assignment to `admin`/`member` via `GroupRole::assignable_from_name`.
- Gossip apply for `MemberRoleUpdated` only rejects `Owner`, not `Moderator` or `Guest`.
- `validate_apply` checks signer authority, not target role assignability.
- `GroupInfo::set_member_role` accepts any `GroupRole` enum variant.

Anchors cited by reviewer:

- `src/groups/member.rs:85-99` — assignment parser accepts only `admin`/`member` and rejects reserved names.
- `src/bin/x0xd.rs:8669-8679` — `MemberRoleUpdated` apply arm rejects only `Owner`, then applies `next.set_member_role(&agent_id, role)`.
- `src/groups/state_commit.rs:599-603` — signed commit validator checks signer role only.
- `src/groups/mod.rs:835-840` — public role mutator accepts any `GroupRole`.

Disposition required:

- Fix before Slice 4, or Jim explicitly accepts. Recommended fix: reject non-assignable target roles on every role-update authoring/apply path; add gate-runnable test proving signed `MemberRoleUpdated { role: Moderator/Guest/Owner }` is rejected by gossip/apply.

#### MEDIUM — Legacy Owner “byte-for-byte chain replay” evidence is overstated

Finding summary:

- Slice 2 checkpoint says byte-for-byte legacy chain replay, but test mints “legacy” commits at runtime with current code and hardcodes only roster JSON hash / roster root.
- This proves current code can author/replay Owner-containing commits, not that a pre-Slice-2 historical chain verifies byte-for-byte.

Anchors cited by reviewer:

- `gsd/checkpoints/2026-06-14-slice-2-owner-retirement.md:53-56`
- `tests/owner_retirement.rs:337-364`
- `tests/owner_retirement.rs:21-24`, `305-319`

Disposition suggested:

- Either add a fixed fixture generated from `189b89c` / hardcoded expected commit-state hashes, or correct the checkpoint/PR wording.

#### MEDIUM — “REST coverage” is mostly daemon-free helper coverage, not actual handler coverage

Finding summary:

- Slice 3 tests mirror REST semantics in helper code instead of calling daemon handlers.
- `x0xd` bin has `test = false`; actual daemon tests are ignored.
- Helper tests missed the real `leave_group` mutate-before-seal path.

Anchors cited by reviewer:

- `tests/membership_authority.rs:3-6`
- `Cargo.toml:105-108`

Disposition suggested:

- Add targeted daemon-backed coverage where needed, or stop claiming actual REST handler coverage where only helper semantics were exercised.

#### LOW — Planning evidence is dirty / partially uncommitted

Finding summary:

- Planning worktree has uncommitted plan/packet changes, weakening reproducibility from committed planning branch alone.

Disposition suggested:

- Commit or explicitly track/discard planning changes before using them as approved source of truth.

### Adversarial test-quality / evidence notes

Reviewer locally checked these and reported pass:

- `cargo fmt --all -- --check`
- `cargo nextest run --all-features --test membership_authority` — 12/12 pass
- `cargo nextest run --all-features --test owner_retirement -E 'test(owner_retirement)'` — 8/8 pass
- `cargo nextest run --all-features -E 'test(last_admin)'` — 22/22 pass

Reviewer confirmed PR #5 CI green at head `6ebac93` and agreed CI is correctly used as green of record. Local macOS mesh/timing failures were baseline-classified in prior evidence.

### Adversarial overstatement list

- Slice 1/3 checkpoints imply the last-admin choke point safely blocks self-leave; current `leave_group` can mutate live state before seal failure.
- Slice 2 checkpoint overstates “byte-for-byte legacy chain” proof; commits are generated with current code.
- Slice 3 checkpoint overstates REST coverage; much is helper simulation, not real handler execution.
- “Role API rejects reserved roles” is true for REST parsing, false for signed `MemberRoleUpdated` apply.

## Craft retro

- Reviewer/tool: `craft` subagent.
- Result: **Concerns**.
- Gate impact: one **CONFORMANCE** finding requires fix or explicit Jim acceptance before Slice 4.

### CONFORMANCE — Slice 2 reserved role assignment is restricted on REST but not on parallel gossip role-update path

Finding summary:

- Same issue as adversarial HIGH reserved-role finding, framed as concrete sibling divergence between REST and gossip role assignment paths.

Anchors cited by reviewer:

- `src/groups/member.rs:85-99` — `GroupRole::assignable_from_name` accepts only `admin` / `member`.
- `src/bin/x0xd.rs:12176-12179` — REST assignment uses assignment parser.
- `src/bin/x0xd.rs:8669-8671` — gossip apply rejects only `GroupRole::Owner`.
- `src/bin/x0xd.rs:8677-8679` — gossip apply performs `next.set_member_role(&agent_id, role)`.

Disposition required:

- Fix now by rejecting any role other than `Admin` or `Member` in `MemberRoleUpdated`, or explicitly justify a legacy-replay exception. **Needs fix or Jim acceptance before Slice 4.**

### SIMPLICITY — Advisory sender/actor checks are uneven across parallel signed gossip receive arms

Finding summary:

- `GroupDeleted` documents and implements “authority from `commit.committed_by`, not `sender_hex`”.
- Membership arms use similar signer-binding shape.
- `PolicyUpdated` ignores `actor` entirely.
- `MemberRoleUpdated` still gates on `actor == sender_hex` plus `actor_role`.
- `validate_apply` is the real authority gate, so this is not a correctness claim; it is a consistency/maintainer-readability smell in ADR-0016 receive-path code.

Anchors cited by reviewer:

- `src/bin/x0xd.rs:8563-8574` — `GroupDeleted` comment/shape.
- `src/bin/x0xd.rs:8219-8220` — `MemberAdded`.
- `src/bin/x0xd.rs:8434-8435` — `MemberRemoved`.
- `src/bin/x0xd.rs:8704-8705` — `MemberBanned`.
- `src/bin/x0xd.rs:8608-8618` — `PolicyUpdated` ignores actor.
- `src/bin/x0xd.rs:8657-8662` — `MemberRoleUpdated` actor/sender role gate.

Disposition suggested:

- If not normalizing now, add checkpoint/PR note explaining the intentional split and carry a follow-up to make signed state-event metadata handling uniform.

### Craft suggested PR note

> Authority is enforced by signed `GroupStateCommit` validation; unsigned event `actor`/transport `sender` checks are advisory and not yet uniform across all metadata arms. Phase 1 removes creator gates; a follow-up should normalize actor/committer/sender handling across signed state events.

## Current decision / blocker state

Do **not** dispatch Slice 4 yet. Required before Slice 4 acceptance:

1. Fix or Jim-accept HIGH adversarial finding: non-creator last-admin `DELETE /groups/:id` self-leave can mutate live state before seal failure.
2. Fix or Jim-accept HIGH adversarial / CONFORMANCE craft finding: reserved roles can still be applied through signed `MemberRoleUpdated` gossip path.
3. Decide whether to fix now or carry medium evidence/wording issues:
   - legacy byte-for-byte overstatement;
   - REST coverage overstatement / daemon-backed coverage gap;
   - planning branch uncommitted packet/plan state.
4. Decide whether to normalize or explicitly note advisory sender/actor check inconsistency.

## Recommended remediation sequence

1. Dispatch a focused remediation packet before Slice 4:
   - clone-first + last-admin precheck (or equivalent) in non-creator `DELETE /groups/:id` self-leave path;
   - reject non-assignable `MemberRoleUpdated` roles on gossip apply path, with gate-runnable tests.
2. Update Slice 1-3 checkpoint/evidence wording for any overclaims, or record Jim's acceptance of the wording as-is.
3. Re-run mandatory checks and PR #5 CI.
4. Re-run adversarial/craft only on the remediation delta or ask reviewers to confirm blocker closure.
5. Then dispatch Slice 4.
