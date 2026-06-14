# GSD Dispatch Note — Slice 2: owner retirement / flatten the role model (ADR-0016 Phase 1)

Date: 2026-06-14
Prepared by: Claude (Cowork planning session)
Status: **Approved by Jim 2026-06-14; filed to the planning branch. Runs after the per-project gate setup.**
Role: **Implementer — Slice 2 of 7 only.** Dispatch via the gsd orchestrator (local).

This is a lean dispatch note, not a full packet. **The slice definition lives in the plan** — `gsd/plan/phase-1-plan.md`, the "Slice 2" section plus the binding "Universal slice preamble" — and that governs. This note adds only the changes we want that surfaced *after* the plan was approved; it does not restate the plan.

## Branch & workspace

Continue on `feat/adr-0016-phase-1-authority-alignment` (carries the accepted, verified Slice 1 at `903cf8d`). **Run from a clean worktree on that branch** — the root `main` worktree currently holds unrelated dirty/staged changes; do not build Slice 2 on top of them. Push only to Jim's fork — `origin` or `fork`, verify with `git remote -v` (both currently point to `JimCollinson/x0x`); never to upstream (`saorsa-labs/x0x`, push-disabled). No PR (Jim's gate). In this clone, `upstream/main` is currently `189b89c`; still sync and re-verify the cited sites at dispatch per the preamble.

## Read first (planning branch)

1. `gsd/plan/phase-1-plan.md` — Slice 2 section + Universal slice preamble: the full slice definition (scope, out-of-scope, tests, verification, done-when, stop-if), including the in-scope receive-path grep for sibling creator comparisons.
2. `gsd/checkpoints/2026-06-12-slice-1-last-admin-invariant.md` — inherit the authority/apply-path map and the behavior-neutrality reasoning.
3. `gsd/evidence/2026-06-13-slice-1-local-gate.md` — the Slice 1 verification outcome and full failure analysis (backs the honesty points below).

## Changes we want (surfaced after planning)

The plan was approved before the verification-guardrails work and before Slice 1 ran. These refinements apply on top of it:

1. **Green of record is CI, not local output.** The authoritative "all gates green" is the standing draft mirror PR on the fork — `feat/adr-0016-phase-1-authority-alignment` → `main` on `JimCollinson/x0x` — read its Checks tab, per push; not the local terminal. Confirm the PR number at dispatch (`gh pr list --repo JimCollinson/x0x`); if none is open, open/refresh one (or push to trigger the fork's CI) before claiming green. Reconcile any local-vs-CI difference in the checkpoint. (The plan predates the CI mirror.)
2. **Honest green — no wrapper, no shim.** Do not modify the test harness, the daemon wrapper, the build, or the environment to turn a red suite green; if the slice appears to *need* such a change, stop and surface it rather than editing in-session. Any failing or skipped test you rely on must be reproduced on clean `189b89c` (baseline-diff) and recorded as environmental. A wrapper-dependent green is not a green — it is why the alternative Slice 1 implementation was rejected.
3. **Make the new error-string checks gate-runnable.** Slice 2 introduces the R5 role-assignment rejections (the exact `'owner' is a legacy role…` / `role '<name>' is reserved…` 400 strings). Slice 1 found that REST-handler unit tests in the `x0xd` binary never run under the normal gates (`Cargo.toml` sets `test = false` for that bin), so its exact-string contract is only assertable at the maintainer gate. Don't inherit that gap: assert the R5 strings in the **normal** gates — e.g. a single-daemon (no-mesh) integration test that is **not** `#[ignore]`d, or by placing the string constants where in-crate unit tests can see them. Record the approach.

**Gate status — the per-project gate is set up first, separately.** A one-time setup packet (`gsd/packets/2026-06-14-x0x-gsd-perproject-setup.md`) wires x0x's local fast gate + CI arbiter before this slice runs; once it is in, Slice 2 runs under it (fast local check + CI as green of record). **Do not create or modify the gate or the pre-push hook as part of Slice 2** — that belongs to the setup job; changing tooling mid-slice trips the stop conditions.

**Environment heads-up — `time` crate (not a code change):** a fresh dependency resolve may select `time 0.3.48`, which fails the toolchain's coherence check. A local `time 0.3.47` pin (the lockfile is gitignored) unblocks a *local* build, but **it cannot be the green of record**: if CI can't reproduce a green, baseline-diff the failure against `189b89c` and surface it — do not claim readiness from a local-only lockfile pin.

## Output (per the plan + preamble)

Commit + push the slice to the feature branch; file a checkpoint to `gsd/checkpoints/2026-06-14-slice-2-owner-retirement.md` referencing this note and the plan's Slice 2 section. Make the receive-path creator-comparison grep result a first-class evidence item, alongside exact verification command outputs, the upstream-diff file list, drift vs the pinned lines, deviations, how the R5 strings were made gate-runnable, and the recommended next step.
