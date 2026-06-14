# GSD Setup Packet — x0x per-project gate + CI arbiter (one-time)

Date: 2026-06-14
Prepared by: Claude (Cowork planning session)
Status: **Approved by Jim 2026-06-14; filed to the planning branch for OpenCode.**
Role: **Implementer (setup)** — run locally via OpenCode using the local `gsd-project-setup` workflow. One-time; **not** a Phase 1 slice.

## Goal

Stand up x0x's own per-project guardrails so every future slice runs with no manual gate steps: a fast local check that runs automatically before each push, plus the automated server check (CI) declared as the official "did it really pass." The general guardrails are already in the tooling; this adds the two x0x-specific pieces Hermes did not (a project gate + a CI arbiter declaration).

## Decisions to honour (the x0x-specific shape)

1. **Local gate = fast checks only.** The pre-push check runs:
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   The full test suite is **not** in the local gate — it is slow and has known environment-specific failures on this Mac (the daemon-mesh tests). The full suite runs on CI, which is the real green of record. (If clippy's compile makes the push noticeably slow, dropping to `fmt`-only locally is acceptable — clippy still runs on CI.)
2. **CI is the green of record.** Declare the standing mirror PR on the fork (`feat/…` → `main` on `JimCollinson/x0x`) as the official pass check. Record the declaration on the planning branch (`gsd/ci-arbiter.md`). Confirm the mirror PR exists and its Actions run; if not, open/enable it.
3. **Where the gate file lives (the wrinkle).** The deliverable branch must stay free of GSD files and upstream is read-only, so the gate cannot be committed on the feature branch. Keep the **canonical** gate on the planning branch (`gsd/gate.sh`); deploy a copy to `.gsd/gate.sh` in the local build worktree and exclude it locally (`.git/info/exclude`) so it never enters a PR. (If the hook resolves the gate path differently, adapt — the rule is: reviewed/committed on the planning branch, never on the deliverable branch.)
4. **Pin the `time` dependency.** A fresh resolve selects `time 0.3.48`, which fails to compile on the current toolchain; pin `time 0.3.47` locally so the gate and builds are clean. (Lockfile is gitignored — local-only; this is convenience, not the green of record.)
5. **Gate feature branches generally**, not just this Phase 1 branch, so the setup persists across phases.
6. **Frozen after setup.** Once approved and committed, the gate is not regenerated or edited per slice; any change is a deliberate, reviewed edit — never mid-slice.

## Steps (run locally via OpenCode, `gsd-project-setup` workflow)

1. Scaffold `.gsd/gate.sh` with the two fast-gate commands above, in the exact format the pre-push hook expects.
2. Commit the canonical copy to the planning branch at `gsd/gate.sh`; deploy to `.gsd/gate.sh` in the build worktree, locally excluded.
3. Install the pre-push hook (`~/ops/gsd/hooks/gsd-pre-push`) in the x0x clone, gating feature branches.
4. Pin `time 0.3.47` locally; confirm `cargo fmt` and `cargo clippy` run clean.
5. Confirm/declare the CI arbiter: ensure the mirror PR exists and its checks run; write `gsd/ci-arbiter.md` on the planning branch naming it as the green of record.
6. Validate: a trivial push is blocked when format/lint fail and allowed when clean; CI runs on the mirror PR.
7. Report back: what was created and where, the exact `gate.sh` contents, the CI arbiter location, and confirmation the hook + CI work.

## Constraints

- Never push upstream; planning-branch + fork only. No GSD files on the feature/deliverable branch.
- Do not weaken the gate to make it pass; if the fast checks can't go green locally, surface why rather than papering over it.

## After this

The Slice 2 dispatch note's "no `.gsd/gate.sh` yet" caveat is removed; Slices 2–7 then run with the gate auto-checked locally and CI as the arbiter — no per-slice gate setup.
