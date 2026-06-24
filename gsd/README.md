# GSD planning home — ADR-0016 group authority work

This branch (`gsd/adr-0016-planning`) is the planning home for implementing
x0x ADR-0016 (role-based group authority — flat Admin/Member, retiring
`Owner`) per the phasing agreed on issue #107. It holds GSD process
artifacts ONLY:

- `gsd/spec/` — phase specs (bounded build contracts)
- `gsd/plan/` — GSD plans and slice definitions
- `gsd/packets/` — dispatched work packets (disposable orientation)
- `gsd/checkpoints/` — session handover / checkpoint notes
- `gsd/evidence/` — verification evidence

## Binding rules

1. **This branch never merges into any other branch and never becomes a
   PR.** Nothing under `gsd/` may ever reach upstream.
2. **Upstream (`saorsa-labs/x0x`) is read-only. Never push there.**
   Changes reach upstream only via pull requests from this fork's feature
   branches, and only with Jim Collinson's explicit prior approval —
   PR creation is always a human gate.
3. **Deliverable work happens on feature branches** cut from freshly-synced
   `main` (current v0.26 re-home: `feat/adr-0016-phase-1-on-v0.26`;
   historical pre-v0.26 branch: `feat/adr-0016-phase-1-authority-alignment`)
   and contains only the deliverable: code changes plus their documentation.
   No GSD files there, ever — a PR ships the whole branch diff.
4. **Upstream ships several times a day.** Sync `main` with upstream at
   session start; rebase in-flight feature branches before review and
   again before any PR.
5. **Gates before any PR:** all upstream quality gates green (fmt, clippy
   `-D warnings`, nextest; no production `unwrap`/`expect`/`panic`),
   gauntlet review (independent clean-context test + adversarial review),
   and the maintainer-side final test gate on Jim's local machine
   (multi-daemon convergence + the `#[ignore]`d daemon-API suite).
6. **Work only your assigned slice from an approved packet.** If no
   approved packet covers what you are about to do — stop and request one.

## Start here

Read `gsd/spec/`, then `gsd/plan/`, then your packet in `gsd/packets/`.
The formal contract is upstream
`docs/adr/0016-role-based-group-authority-flat-admin.md` (Accepted) and
the maintainer's phasing comments on issue #107: Phase 1 = authority
alignment (this work), Phase 2 = KeyPackage distribution (wire-shape
sketch on the issue REQUIRED before any implementation), Phase 3 =
deterministic committer + race handling.
