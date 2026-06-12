# Plan addendum 1 — Slice 1 evidence requirement (binding)

- Date: 2026-06-12
- Status: Accepted by Jim; binding on Slice 1. Amends `phase-1-plan.md` (Slice 1 "Done when" and checkpoint requirements).
- Origin: independent external review of the approved plan (second model, full concurrence with sequencing and gates; one tightening adopted).

## The tightening

The plan's highest-risk assumption is that **every delivery path that mutates group state passes through the `validate_apply` choke-point**, and that each path can compute the proposed post-mutation roster correctly. Slice 1's stop conditions already cover discovering a bypass — but discovery must be active, not incidental.

**Therefore, binding on Slice 1:** the slice checkpoint MUST include, as its **first evidence item**, an **authority/apply-path map**: an enumerated list of every REST handler and every gossip-apply arm that mutates group membership, roles, policy, or group lifecycle, showing for each —

1. where it computes (or will receive) the proposed post-mutation roster;
2. where it passes the choke-point (file/function reference at the verified commit);
3. confirmation that the shared roster-computation helper is used (or why a path differs).

Any path that mutates state without passing the choke-point is an immediate stop-and-report (existing stop condition) — the map is what proves the search was exhaustive. Produce the map **before or alongside** implementation, not retrospectively.

## Also noted from the review (no plan change)

- Serial dispatch confirmed correct; do not parallelize slices.
- REST pre-check / apply-side drift: the shared-helper requirement stands; reviewers should check it specifically in the gauntlet.
- Slice 5's endpoint-semantics change must be loud in PR notes (already required).
- Verb naming (#107): nudge for the maintainer's answer before Slice 5 if possible; provisional "disband" default stands.
