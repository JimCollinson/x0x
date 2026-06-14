# GSD Dispatch Note — Slice 2 finish-off: byte-for-byte legacy-chain test (ADR-0016 Phase 1)

Date: 2026-06-14
Prepared by: Claude (Cowork planning session)
Status: **Approved by Jim 2026-06-14; filed for OpenCode.**
Role: **Implementer — one small test addition to complete Slice 2.** Continue on `feat/adr-0016-phase-1-authority-alignment` from `4da985e`.

## Why

Slice 2 (`4da985e`) is implemented and CI-green, but one **plan-specified** Slice 2 test is missing. The plan's Slice 2 test list requires: *"a historical chain containing Owner entries verifies byte-for-byte."* `tests/owner_retirement.rs` covers legacy-owner-administers and owner→admin normalization, but not a multi-commit legacy-chain replay. This is the direct proof of ADR-0016's migration promise (existing groups' histories verify unchanged) and a standing guard against signed-format / hash drift across Slices 3–7.

## The test to add (gate-runnable: `tests/`, not `#[ignore]`d)

Prove that a pre-Slice-2-style group history with legacy `Owner` entries still validates and hashes identically under the current code:

1. Build a legacy group whose roster holds a legacy `Owner` (the retained `GroupMember::new_owner` constructor / the `legacy_owner_group` helper pattern already in `owner_retirement.rs` is the starting point), ideally a **mixed roster** (`Owner` + `Admin` + `Member`) so every role's serialized byte ordering is exercised. Author a short **chain of a few signed commits** over it (e.g. a policy update and a membership change), capturing each commit's `state_hash` / `roster_root` as authored.
2. Replay that chain on a fresh replica through the real validate → mirror-mutation → finalize sequence (as `owner_retirement.rs`'s `gossip_apply` helper does), under the current code.
3. **Assert byte-for-byte identity:** each replayed step's recomputed `state_hash` / `roster_root` equals the value the legacy chain committed — i.e. the new code reads legacy `Owner` data to the same bytes/hashes it always did. Include at least one direct serialization/hash check on a legacy `Owner` roster, so a future `serde`/role-byte change would fail this test.

## Constraints / honesty

- **Test only.** Do NOT modify any production serialization, role-byte values, hashing, or the commit format to make it pass. If the legacy chain does NOT verify identically, that is a real migration regression — **stop and surface it; do not adjust the test to go green.**
- Continue on the feature branch from `4da985e`; push to the fork only; never upstream; no PR; no GSD files on the feature branch.

## After

Re-run the local fast gate (`fmt` + `clippy`) and confirm CI green on PR #5. Then the Slice 2 checkpoint can be filed claiming full plan conformance. (Claude will draft that checkpoint once this test is in and green.)
