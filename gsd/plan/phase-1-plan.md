# GSD Plan: ADR-0016 Phase 1 — authority alignment

- Status: **Approved by Jim, 2026-06-12** (open questions resolved with plan defaults: provisional "disband" if #107 unanswered at S5; clean-break endpoint semantics; S3/S4 separate; property test after S5).
- Date: 2026-06-12
- Role performed: Planner (gsd-plan). Planning only.
- Objective: implement ADR-0016 Phase 1 (flat Admin/Member authority, retiring Owner) as one PR from `JimCollinson/x0x` branch `feat/adr-0016-phase-1-authority-alignment`, sliced into independently verifiable work packets for Claude Code cloud sessions.

## Sources read

- `gsd/spec/phase-1-authority-alignment.md` @ `gsd/adr-0016-planning` (`JimCollinson/x0x`, blob `84b7c06`) — the approved build contract; all R-numbers, code sites, and exact strings below come from it.
- `gsd/README.md` @ `gsd/adr-0016-planning` (blob `baad918`) — process rules: upstream read-only, planning branch never merges, gates before PR.
- `docs/adr/0016-role-based-group-authority-flat-admin.md` @ `saorsa-labs/x0x` main `189b89c` (Accepted) — the contract above the spec.
- `Cargo.toml` and `tests/harness/Cargo.toml` @ `189b89c` — verified `proptest = "1.4"` already present (workspace dev-dependencies and harness).
- GSD slice template (gsd skill `references/template-slice.md`).

No new ADR is needed: every decision the plan relies on is recorded in Accepted ADR-0016 or in the approved spec (including the resolved leave/end-group design point). The plan sequences; it adds no requirements.

## Plan summary

Seven slices, dispatched serially (one cloud session each) on the single feature branch. The last-admin invariant (R2) lands first as the safety net; the role model is then flattened (Owner retirement, role-API restriction, genesis change); the membership and invite creator gates are deleted next, each replaced by the surviving role-based layer; the leave/end-group endpoint split follows once both the invariant and any-admin group-ending exist; the dedicated last-admin property test then attacks the finished behavior on both delivery paths; and the surfaces audit plus blunt documentation closes the branch. At no point between slices does the branch hold a state where the creator/owner protections are gone but the invariant is absent, and every slice leaves the full quality gates green. After the slices: final rebase, the gauntlet (clean-context test + adversarial review), Jim's local maintainer-side test gate, the pending verb swap if #107 has been answered, and PR creation only on Jim's explicit confirmation.

## Universal slice preamble (binding for every slice)

1. Sync fork `main` with upstream `saorsa-labs/x0x` `main`; rebase `feat/adr-0016-phase-1-authority-alignment` onto it if upstream has moved.
2. **Re-verify every cited code site before relying on it.** All line numbers below are pinned to upstream `189b89c` and upstream ships daily. Record drift in the slice checkpoint. If a cited mechanism has materially changed upstream (not just moved), stop and surface to Jim before implementing.
3. Work only this slice, from its approved packet. No GSD files on the feature branch. Conventional Commits.
4. No production `unwrap`/`expect`/`panic` in new or touched code.
5. §3 error strings are fixed verbatim; HTTP status codes follow repo precedent — record any deviation from the 400/409 listed in the spec in the slice checkpoint, do not improvise strings.
6. End with a checkpoint (gsd-checkpoint skill) to `gsd/checkpoints/` on the planning branch: evidence, drift found, deviations, recommended next step. **A slice is done only when its checkpoint is reviewed and the next packet is approved.**

**Verification commands, every slice (exact):**

```
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features --workspace
```

plus the per-slice targeted filter listed in each slice. The `#[ignore]`d daemon-API suite and multi-daemon convergence tests are **not** run inside cloud slices — they are Jim's local final gate (see post-slice steps); slices touching daemon API behavior (2–5) note their exposure to that gate in their checkpoints.

---

## Slice 1 — Last-admin invariant (R2)

**Goal:** the net goes up before any guard comes down. Reject, at the `validate_apply` choke-point on every delivery path, any commit whose post-mutation, non-withdrawn state contains zero active members of rank ≥ Admin.

**In scope (spec R2; sites pinned at `189b89c`):**
- New check at the `validate_apply` choke-point (`src/groups/state_commit.rs:521` region; authority is the final pipeline step). Withdrawn (group-ending) state is exempt. Legacy `Owner` counts as Admin — implement via `role.at_least(Admin)` (Owner rank 4 > Admin rank 3) so the alias evaluation is automatic.
- The post-mutation roster problem (spec-verified subtlety): the commit carries only `roster_root` (a hash) and `ApplyContext.members_v2` is the *parent* state. The invariant must be evaluated over the **proposed post-mutation roster computed by the applier**, fed to the check at the same choke-point on **both** delivery paths (REST and gossip-apply). New `validate_apply` argument vs adjacent mandatory check is implementer latitude; the binding constraints are: same choke-point semantics, all delivery paths, post-mutation evaluation. Use one shared helper so both paths compute the proposed roster identically.
- Rejections use existing `ApplyError::Invariant` ("invariant violation").
- REST friendly pre-checks (§3) on the handlers whose acts can zero the admin set — remove-member, ban, `update_member_role` (demotion) — returning 409 with the exact string: `{"error":"a group must always have at least one admin; make another member an admin first"}`. (The "before leaving" 409 belongs to Slice 5 with the leave endpoint; the choke-point already blocks last-admin self-leave from this slice onward.)

**Out of scope:** deleting any creator gate or owner-target guard (Slices 2–4); genesis seeding (Slice 2); `DELETE /groups/:id` semantics (Slice 5); the property test (Slice 6); docs (Slice 7).

**Tests this slice adds:**
- Choke-point unit tests: demote-last-admin rejected; remove-last-admin rejected; ban-last-admin rejected; withdrawal commit from a sole-admin state accepted (exempt); sole legacy `Owner` self-demoting to `admin` accepted (admin count stays ≥ 1); sole legacy `Owner` demoted to `member` rejected; legacy `Owner` counted as admin in mixed rosters.
- Both-path coverage (acceptance criterion 5 starts here): a crafted zero-admin commit rejected via gossip-apply at the choke-point, not only via REST pre-check; REST pre-check tests asserting the exact §3 string and status code.
- A test asserting the proposed roster fed to the check hashes to the commit's `roster_root` (guards the computed-roster seam).

**Verification:** the three universal commands, plus targeted: `cargo nextest run --all-features -E 'test(last_admin)'` (name new tests with a `last_admin` prefix for filterability).

**Done when:** all gates green; rejection demonstrated on both paths; exact strings asserted; the pre-existing suite passes untouched — the invariant should be behavior-neutral for currently-reachable states (creator gates still stand), so if an existing test trips it, investigate the test's roster construction before adjusting anything.

**Stop if:** the check cannot be implemented without altering hashing, signing, the commit format, or validation-pipeline ordering beyond inserting this one step; or any delivery path cannot be given the post-mutation roster without restructuring beyond the choke-point; or re-verification reveals a third delivery path (e.g., state seeding/sync) that applies commits without passing the choke-point — that is a spec-level finding for Jim.

---

## Slice 2 — Owner retirement: flatten the role model (R1 owner cluster, R4, R5, R6)

**Goal:** every former owner-only act becomes an ordinary admin act; `owner` becomes unassignable legacy vocabulary; new groups are born with an ordinary first Admin.

**In scope (sites pinned at `189b89c`):**
- `ActionKind::OwnerOnly` → `AdminOrHigher` for its three acts: `state_commit.rs` 474/489/571 (GroupDeleted, PolicyUpdated, role-change), and the three daemon apply sites `x0xd.rs` 8587/8633/8695. Retire the `OwnerOnly` variant.
- The receive-path creator gate in the `PolicyUpdated` gossip arm (`creator_auth` at ~8625: `sender_hex == creator_hex && actor == sender_hex`) — deleted; the gossip path must rely on the role layer only. While in this arm, grep the receive path for sibling literal creator comparisons and record findings.
- Two-tier role matrix deleted at both enforcement sites: gossip-apply 8681–8691 and REST `update_member_role` 12261–12267. Any admin may change any member's role, including another admin's (subject to the Slice 1 invariant).
- `require_owner` (def 12037, sole use 12131 in `update_group_policy`) deleted; replaced with `require_admin_or_above` (def 12026; the 8 existing call sites — 12064, 12331, 12483, 12626, 12683, 12837, 13009, 13166 — model the target pattern).
- Ownership-transfer 400 stub (12223–12229) removed.
- R5: the role-assignment path accepts exactly `admin` and `member`; rejects with the exact §3 400s: `{"error":"'owner' is a legacy role and cannot be assigned; valid roles: 'admin', 'member'"}` and `{"error":"role '<name>' is reserved and cannot be assigned; valid roles: 'admin', 'member'"}`. `GroupRole::from_name` (`member.rs` 6–71) keeps parsing all stored names — restriction at assignment, never at deserialization.
- R6: the five `new_owner` genesis call sites (`groups/mod.rs` 266, 570; `x0xd.rs` 678, 13590, 13631) seed the creator as first **Admin**; no new group contains an `owner` entry. The constructor and the `state_commit.rs:597` test helper may remain for legacy-roster tests.
- In-code doc comments updated to the flat model (`state_commit.rs` module header "Owner for policy changes"; the `ActionKind` docs).

**Out of scope:** the membership creator gates and owner-target guards (Slice 3); the invite gate (Slice 4); `DELETE /groups/:id` semantics (Slice 5); user-facing docs and rendering (Slice 7); any change to serde names or `role_byte` values (they feed the roster hash — untouchable).

**Tests this slice adds/updates:**
- Promoted (non-creator) Admin updates policy, changes another admin's role, and ends the group — validated on REST and gossip-apply paths (acceptance criterion 1 backbone).
- Legacy-roster fixtures: a roster with an `Owner` entry administers unchanged; a historical chain containing `Owner` entries verifies byte-for-byte; one ordinary `MemberRoleUpdated` owner→admin normalization commit validates (acceptance criterion 3).
- Role API rejects `owner`, `moderator`, `guest` with the exact strings (acceptance criterion 4, error half).
- Genesis tests: new group roster holds the creator as `admin`, never `owner`.
- Expected churn: existing fixtures that assume an Owner-creator updated without weakening their assertions.

**Verification:** universal commands plus `cargo nextest run --all-features -E 'test(role) or test(owner) or test(genesis)'` (adjust to actual test names; record the filter used).

**Done when:** no `OwnerOnly` classification or `require_owner` remains; role API contract enforced with exact strings; genesis seeds Admin; legacy chains verify byte-for-byte; all gates green.

**Stop if:** any change would touch serde role names, `role_byte` values, or anything feeding `roster_root`/`state_hash`; or the receive-path grep reveals additional authority mechanisms beyond the two the spec documents (literal creator checks + role layer) — that is spec drift, surface it.

---

## Slice 3 — Membership authority: add, remove, ban (R1 membership gates, R3)

**Goal:** add/remove/ban are authorized by committed-roster role, never creator identity; the owner-target special cases vanish because the Slice 1 invariant subsumes the protection they provided.

**In scope (sites pinned at `189b89c`):**
- Creator gates deleted, role lookup is the survivor: add-member 11043–44 and 11170–74; remove-member 11350–51 and 11578–83 (all `x0xd.rs`).
- R3 deletions: "cannot remove creator" 11348 and 11574; "cannot ban owner" 12339 and 12489.

**Out of scope:** the invite gate (Slice 4); KeyPackage distribution in `MemberAdded` (Phase 2 — the existing 424 KeyPackage error on delegated ban is **unchanged**, explained in the PR description); rekey/committer behavior (Phase 3); the full #107 repro with a non-inviting admin banning (explicitly not claimed by this PR).

**Tests this slice adds:**
- Promoted Admin adds and removes members — REST and gossip-apply both exercised at the choke-point (acceptance criterion 5).
- Removing or banning a legacy `Owner` who is *not* the last admin now succeeds (the old guards are gone; R2 holds the line only where it should).
- Removing/banning/demoting the last admin through these specific handlers returns the exact §3 409 (re-asserting Slice 1's net through the now-unguarded paths).
- Delegated-ban nuance pinned down: a promoted admin whose daemon holds the target's KeyPackage (e.g., added the target themselves) can ban; one without the material receives the unchanged 424 — asserted as 424, not a creator-identity 403.
- A `member` cannot add/remove/ban (role lookup rejects).

**Verification:** universal commands plus `cargo nextest run --all-features -E 'test(member) and (test(add) or test(remove) or test(ban))'` (adjust to real names; record).

**Done when:** zero literal creator comparisons remain in the add/remove/ban paths; both-path tests green; all gates green.

**Stop if:** deleting a guard exposes a state the choke-point invariant does not catch (a gap between REST pre-check and `validate_apply`) — defect in Slice 1's seam, surface before patching around it; or re-verification finds the remove/ban paths consulting creator identity at uncited sites beyond line drift.

---

## Slice 4 — Invites per-issuer + creator provenance (R7, R8)

**Goal:** any Admin issues invites, with issue/consume/track running on the issuing daemon; a joiner's recorded `creator` is genesis-derived history, never unsigned invite metadata.

**In scope (sites pinned at `189b89c`):**
- The invite creator gate (`x0xd.rs` 10625–10628) deleted; role lookup is the survivor. The existing per-issuer issue/consume/track mechanic is kept, simply no longer creator-locked (R7).
- R8: the joiner's `GroupInfo.creator` is no longer seeded from unsigned `invite.inviter`; it derives from the seeded base state / genesis. Inviter identity remains the routing target for join-result polling — keep the two variables distinct in code and in tests.
- Closing sweep: with Slices 2–4 done, all literal creator-identity authority checks named by the spec are gone. Run and record (as slice evidence) a search for remaining `creator` comparisons in authority paths; anything found is either provenance/routing (fine, documented) or drift (surface it).

**Out of scope:** invite wire-format changes; KeyPackage distribution (Phase 2); GUI/CLI invite surfaces (Slice 7).

**Tests this slice adds:**
- Promoted (non-creator) Admin issues an invite; the joiner consumes it against the *issuing* daemon and joins (acceptance criterion 1, invite leg).
- A `member` cannot issue an invite.
- When inviter ≠ creator: the joiner's stored `GroupInfo.creator` equals the genesis creator, not the inviter; join-result polling still routes to the inviter.
- Regression: creator-issued invites still work (the common path must not break).

**Verification:** universal commands plus `cargo nextest run --all-features -E 'test(invite)'`.

**Done when:** any-admin invites work end-to-end on the issuing daemon; provenance and routing identities verifiably distinct; the no-creator-authority sweep recorded; all gates green.

**Stop if:** the invite flow proves structurally keyed to the creating daemon beyond the deleted gate (storage layout, polling assumptions) such that per-issuer operation needs redesign rather than gate removal; or fixing provenance would require changing what the invite carries on the wire.

---

## Slice 5 — Leave / end-group split (spec §3 resolved design point)

**Goal:** exactly two user-facing actions. `DELETE /groups/:id` means *leave* for everyone regardless of rank; ending the group is a separate, explicit, any-admin act on the existing terminal-withdrawal mechanism.

**In scope:**
- **First step (spec-mandated):** verify the current `DELETE /groups/:id` handler semantics on freshly synced main (documented today as "leave or delete" switched on caller identity). If reality differs materially from that description, stop before implementing.
- `DELETE /groups/:id` becomes a pure self-act for all ranks. Last-active-admin leaver (legacy `Owner` counts) blocked with the exact §3 409: `{"error":"a group must always have at least one admin; make another member an admin before leaving"}` as the REST pre-check, with the Slice 1 choke-point invariant backing every path.
- End-group: the existing terminal-withdrawal commit; endpoint path `POST /groups/:id/state/withdraw` kept for wire/API compatibility; any admin may perform it at any time, including as last admin (invariant-exempt per the ADR).
- CLI: one primary command — provisionally `x0x group disband <id>` (the spec's proposed verb; "delete" is the fallback pending the maintainer's answer on #107). Isolate the verb string so the swap is one line. `state-withdraw` retained as a quiet deprecated alias.
- `docs/api-reference.md` entry for the endpoint: "<Verb> the group for everyone (permanent; propagates to all members)". (The full language sweep is Slice 7.)

**Out of scope:** renaming the chain's internal `withdrawn` record (wire-frozen); self-leave PCS rekey (Phase 3); the broader R9 sweep.

**Tests this slice adds:**
- Leave as member, as non-last admin, and as a legacy `Owner` when another admin exists (previously impossible — creator-DELETE meant delete-group) — each a pure self-removal that converges.
- Last-admin leave → exact "before leaving" 409; choke-point rejection also asserted on the gossip path for a crafted last-admin self-removal commit.
- Last-admin end-group succeeds (exempt); a `member` cannot end the group; a promoted admin's end-group converges to other members (acceptance criterion 1, end-group leg).
- CLI command and deprecated alias behavior.

**Verification:** universal commands plus `cargo nextest run --all-features -E 'test(leave) or test(disband) or test(withdraw)'`.

**Done when:** the two-action model holds with exact error strings; old creator-DELETE delete-group behavior is gone (and flagged for the PR description as an intentional endpoint-semantics change); all gates green.

**Stop if:** the verified current handler semantics diverge from the spec's description in a way that changes the design point; or honoring wire/API compatibility for the withdraw endpoint proves impossible without format changes; or the verb decision turns out to require more than the localized swap.

---

## Slice 6 — Last-admin property test (acceptance criterion 2, dedicated)

**Goal:** property-level proof that across generated commit sequences, on both delivery paths, no non-withdrawn state with zero active admins is ever reachable, with legacy `Owner` counted as admin.

**In scope:**
- A proptest-based suite (proptest 1.4 is already a workspace dev-dependency and present in `tests/harness` — no new dependency; follow existing repo property-test conventions). Generator: an action enum over add/remove/ban, role changes (promote/demote, including owner→admin normalization), self-leave, policy update, and group-ending withdrawal; initial rosters mixing `admin`, `member`, and legacy `owner` entries; shrinking-friendly.
- Sequences applied via both paths: REST handler flow (pre-checks + choke-point) and gossip-apply (choke-point only — the check itself, not just pre-checks).
- Properties: every accepted sequence leaves ≥ 1 active member of rank ≥ Admin unless the state is withdrawn; rejected actions never mutate state; withdrawal remains reachable from sole-admin states (the exit valve is never sealed).

**Out of scope:** multi-daemon convergence (Jim's local gate); fuzzing other pipeline invariants; performance tuning beyond keeping CI runtime sane (case count per repo convention; note the local high-case run in evidence).

**Tests:** the property suite itself, named with a `last_admin` prefix.

**Verification:** universal commands plus `cargo nextest run --all-features -E 'test(last_admin)'`; record the proptest case count and seed policy in the checkpoint.

**Done when:** the property holds at a meaningful case count on both paths; suite green in the workspace run; all gates green.

**Stop if:** the property finds a genuine counterexample — that is a defect in Slices 1–5, not a test problem: stop, record the shrunken sequence in the checkpoint, and report; the fix belongs to the violated slice's scope and gets its own packet. Never weaken the property to pass.

---

## Slice 7 — Surfaces audit + documentation (R9, R10, §3 language sweep) — final slice

**Goal:** no surface requires an owner to exist; legacy `owner` renders readably wherever roles display; the docs say plainly what Admin means.

**In scope:**
- R9 sweep across: the GUI surface, `src/cli/`, `src/api/`, `src/bin/gui_coverage.rs`, `docs/api-reference.md`. Known findings to fix (from the spec's full read): the AdminOnly write-access description ("only `Admin` or `Owner` may send"); `POST /groups/:id/state/seal` documented "(owner/admin)"; `POST /groups/:id/state/withdraw` documented "(owner)"; add-member and remove-member rows described as "Creator-authored".
- R10: repo docs state plainly that **Admin is root for the group** — a hostile or compromised admin can admit, remove, rekey, change policy, and end the group; keep the admin set small; do not map softer application roles onto x0x Admin. Role-assignment docs reflect the admin/member-only contract. `x0x group set-role` CLI help lists `admin` and `member` with a one-line meaning each (verified gap: the CLI is a thin pass-through with no role help today). `docs/api-reference.md` gains the short plain-language "Roles" explainer near the role-assignment endpoint (admin = full control including ending the group; member = participant; `owner` = legacy alias rendered for old groups, equivalent to admin, not assignable). This is where the §3 error messages' "valid roles" pointer finds its depth.
- §3.3 language sweep: no user-facing surface (CLI help, GUI, api-reference) describes the group-ending act as "withdrawing" without the chosen verb alongside.
- Verb consistency: if the maintainer has answered #107 by now, apply the verb everywhere; otherwise keep the provisional "disband" and note it for the pre-PR swap.

**Out of scope:** any behavioral change (if the audit finds one needed, that triggers the stop-rule); restructuring docs beyond the named additions.

**Tests:** update `gui_coverage.rs` and any CLI-help assertions the repo has; otherwise the deliverable evidence is the **sweep record** — every surface checked, with disposition (clean / fixed / escalated) — in the slice checkpoint.

**Verification:** universal commands (docs changes still must not break fmt/clippy/nextest), plus the sweep record.

**Done when:** sweep record complete with no unescalated findings; the four known api-reference findings fixed; R10 text landed; all gates green.

**Stop if (binding spec stop-rule):** any finding is unexpectedly large — stop and surface to Jim before expanding scope. A surface that *behaviorally* requires an owner to exist is an automatic stop, not a fix-in-place.

---

## Dependency and ordering rationale

```
S1 (R2 net) ──> S2 (owner retirement) ──> S3 (add/remove/ban) ──> S4 (invites) ──> S5 (leave/end split) ──> S6 (property test) ──> S7 (surfaces+docs)
```

- **S1 before everything** — Jim's binding constraint, honored strictly rather than via "together": the invariant lands while every old protection still stands, so the branch is never in a state where the owner-target guards are deleted and the net is absent. Between S1 and S3 the protections are doubled, never zeroed.
- **Genesis (R6) moved from kickoff seam (c) into S2 — the one deliberate seam adaptation.** If genesis seeded Admin while `OwnerOnly` still existed, the creator of any new group (now Admin, not Owner) would fail policy/end-group/role-change acts, breaking the suite mid-branch and violating per-slice green gates. R6 therefore travels with the `OwnerOnly` → `AdminOrHigher` switch it depends on. The invariant slice (S1) stays pure R2, which also keeps the riskiest plumbing (post-mutation roster at the choke-point) in a session with nothing else on its plate.
- **S2 before S3/S4/S5:** S3 and S4 are technically independent of S2 after S1 — but landing the role-model flattening early means every later slice tests against the final model, and S5 hard-requires S2 (any-admin end-group) and S1 (last-admin leave blocked).
- **S3 before S4:** kickoff seams (a)/(b) kept separate — invite e2e is a different testing shape from gate-deletion-plus-guard-deletion, and R8's two-identities subtlety deserves an undistracted session. S4 closes out the last literal creator checks, which is why the "no creator comparisons remain in authority paths" sweep lands at the end of S4.
- **S6 after S5:** the generator should cover the final action set including self-leave.
- **S7 last** — Jim's binding decision, with the spec's stop-rule.
- **Serial dispatch only:** one feature branch, slices as ordered commit groups; no parallel sessions.

## Risk notes (most likely failure per slice)

- **S1 — the computed-roster seam.** Highest design risk in the PR: a delivery path computing the proposed roster differently from what its `roster_root` hashes would make the invariant check the wrong state. Mitigated by the shared-helper requirement and the roster-hashes-to-`roster_root` test. Second risk: an uncited delivery path bypassing the choke-point (explicit stop condition).
- **S2 — widest test blast radius.** Genesis-seeds-Admin will churn many fixtures that assume an Owner creator; the risk is quietly weakening assertions while updating them. Also: siblings of the `creator_auth` receive-path check may exist — hence the in-slice grep.
- **S3 — pre-check/choke-point asymmetry and 424 confusion.** Tests must distinguish "authority denied" (gone) from "material missing" (424, intentionally unchanged).
- **S4 — conflating the two identities.** Inviter-as-routing-target vs creator-as-provenance; breaking join-result polling is the likely regression.
- **S5 — intentional breaking change, drifting baseline.** Creator-DELETE changes meaning (was delete-group, now leave); must be loud in the PR description. Handler re-verified at slice time. The pending verb is contained by the one-line-swap isolation.
- **S6 — late counterexamples and CI runtime.** A real counterexample is a stop condition (defect in S1–S5); runaway proptest runtime is the mundane risk.
- **S7 — scope creep in the GUI surface.** The stop-rule exists precisely for this; the audit reports, it does not redesign.
- **Cross-cutting:** upstream velocity makes every pinned line number perishable — re-verification preamble and per-session rebase are the mitigation; keep dispatch cadence tight.

## Post-slice steps (plan steps, not slices — each is a gate)

1. **Final rebase + full gates.** Rebase the feature branch onto freshly synced upstream `main`; re-run fmt/clippy/nextest workspace-wide; capture evidence to `gsd/evidence/` on the planning branch.
2. **The gauntlet (before PR).**
   - *Clean-context test* (gsd-clean-context-tester): a fresh agent, repo and docs only, exercises acceptance criteria 1, 3, 4 and the leave/disband flows from the documentation alone.
   - *Adversarial review* (gsd-adversarial-reviewer): tries to disprove readiness — priority targets: invariant bypass routes, residual creator-identity authority, §3 string/code mismatches, roster-hash/migration breakage on legacy chains, the S5 endpoint-semantics change's blast radius.
   - Gate: blockers fixed or explicitly accepted by Jim.
3. **Jim's local maintainer-side gate:** the `#[ignore]`d daemon-API suite and multi-daemon convergence tests on Jim's machine. Never run inside cloud slices.
4. **Verb confirmation:** if the maintainer has answered #107, apply the one-line swap before PR; if unanswered, ship "disband" provisionally (Jim's accepted default).
5. **PR creation — Jim's explicit confirmation required, always.** PR description must state openly: deferred items (delegated ban non-operational until Phase 2; deterministic committer and the two-admin metadata race window until Phase 3); the unchanged 424 explained; the `DELETE /groups/:id` semantics change for creators; the deliberately-kept items (stored `owner` entries, reserved variants, `new_owner` constructor, `creator_agent_id`, audit fields).

## Assumptions

- proptest 1.4 in workspace dev-dependencies at `189b89c` is still present at build time (re-verify in S6's preamble).
- The spec's byte-verified code sites are accurate at `189b89c`; slices re-verify rather than trust line numbers.
- "GUI surface" in R9 lives inside the x0x repo; no external-repo work.
- §3's 400/409 codes are compatible with repo precedent; strings are fixed, codes may deviate with a recorded checkpoint note.
- Cloud sessions can run the full nextest workspace suite (minus the `#[ignore]`d set) in reasonable time.

## Resolved questions (Jim, 2026-06-12)

1. Verb: proceed with provisional "disband" if #107 unanswered at S5 (one-line swap isolated).
2. S5 endpoint change: clean break, loudly flagged in PR description; no transitional behavior.
3. S3/S4: stay separate; merging is a Jim call at dispatch time only.
4. Property test: after S5, as planned.

## Recommended first slice

**Slice 1 — last-admin invariant.** The binding ordering constraint, the riskiest single design seam, and behavior-neutral for currently-reachable states — it can land, be verified on both paths, and be checkpointed before any protective code is deleted anywhere.
