PASTE THIS WHOLE FILE INTO A FRESH CLAUDE CODE (CLOUD) SESSION ON JimCollinson/x0x
GSD Bootstrap — read first (skip this block ONLY if the `gsd` plugin loaded in this session)
You are working under Jim's GSD process.
GSD is a role-based delivery protocol. You must follow the supplied GSD Work Packet as your boundary.
Core rules:

1. Orient before acting.
   * Read the repo start file first: AGENTS.md, CLAUDE.md, README.md, or equivalent.
   * Read the supplied PRD/spec/ADR/current-state/work-packet files.
   * Identify the current stage: thinking, decision prep, spec prep, planning, implementation, review, verification, or handoff.
   * If source-of-truth files conflict, stop and report the conflict.
2. Keep artifact roles separate.
   * PRD/product brief = intent, user value, scope, non-goals, acceptance criteria.
   * ADR = durable decision/invariant/trade-off/consequence. No sequencing, task lists, or detailed build specs inside ADRs.
   * Spec = bounded build contract: behaviours, schemas, interfaces, commands, edge cases, acceptance tests.
   * GSD plan = sequencing, slices, checkpoints, review gates, verification.
   * Checkpoint/handoff = what changed, evidence, risks, next step.
3. Stay inside the requested role.
   * Planner: plan only; do not implement.
   * Implementer: implement exactly the approved slice; do not redesign adjacent architecture.
   * Reviewer: review only; do not change files unless explicitly asked.
   * Verifier: run/inspect checks and evidence; do not treat self-report as proof.
4. Escalate instead of improvising. Stop and ask for Jim's approval if you encounter:
   * product/architecture/security/protocol/storage/API decisions;
   * accepted ADR divergence;
   * scope expansion;
   * new dependency or large restructure;
   * destructive operations;
   * merge/publish/PR-opening action.
5. PR and merge gates. For PRs, merge candidates, or substantial completed work, the default acceptance bar is:
   * local/automated verification evidence captured;
   * fresh clean-context test run or explicitly waived by Jim;
   * fresh adversarial review run or explicitly waived by Jim;
   * blockers resolved or explicitly accepted by Jim;
   * Jim explicitly confirms before any PR is opened against an upstream/shared repo.
6. Required final report. Return:
   * role performed;
   * sources read;
   * changes made or review performed;
   * tests/checks run with exact results;
   * clean-context/adversarial findings if applicable;
   * blockers, risks, open questions;
   * whether the plan remains valid;
   * recommended next checkpoint.
Do not rely on hidden chat context. If the work packet or repo docs do not provide enough context, stop and request a fresh packet.
GSD Work Packet — repository setup: GSD structure for ADR-0016 Phase 1
Date: 2026-06-12 Prepared by: Claude (Cowork planning session), approved by Jim Collinson Requested agent/tool: Claude Code (cloud/web session) on `JimCollinson/x0x` Role requested: Implementer (repository housekeeping only — no product code) Review mode: N/A
Project / workspace
Project: x0x Group Authority — implementing x0x ADR-0016 (role-based group authority, flat Admin/Member, retiring Owner), Phase 1. Repo/path: fork `https://github.com/JimCollinson/x0x` (origin). Upstream: `https://github.com/saorsa-labs/x0x` — READ-ONLY: never push there, ever. Obsidian/project notes: not accessible from cloud; this packet is self-contained. Current source of truth: upstream `docs/adr/0016-role-based-group-authority-flat-admin.md` (Status: Accepted, commit `189b89c`) and the maintainer's phasing comments on upstream issue #107.
Goal
Create the GSD working structure on the fork so subsequent planning and implementation dispatches have a home: a planning branch carrying GSD process artifacts (never merged, never part of any PR) and a clean feature branch for Phase 1 deliverable work.
Read first

* This packet in full.
* Repo root `CLAUDE.md` / `AGENTS.md` (orientation only — you are not building or changing code this session).
Stage
Handoff/housekeeping (pre-planning repository setup).
Approved slice or question
Exactly the numbered steps in Scope, in order. Approved by Jim Collinson 2026-06-12.
Relevant artifacts
PRD/product brief: N/A ADR(s): upstream `docs/adr/0016-role-based-group-authority-flat-admin.md` (Accepted; do not modify) Spec(s): none yet — the Phase 1 spec arrives in a later dispatch Plan/state: none yet — this packet creates the home they will live in Previous review/checkpoint: none in-repo (first repo dispatch)
Scope (do exactly this, in order)

1. Plugin check (report only): state in one line whether the `gsd` plugin / its skills loaded in this session (yes/no). This dispatch doubles as that test.
2. Remotes: verify `origin` = `JimCollinson/x0x`. Add `upstream` = `https://github.com/saorsa-labs/x0x.git` if missing.
3. Sync main: fetch upstream. Confirm fork `main` fast-forwards cleanly to upstream `main` (i.e. fork main contains no commits absent from upstream). Fast-forward it and push to origin. Record the synced HEAD hash. (At packet time upstream HEAD was `189b89c`; if it has moved, sync to current — just record what you got.)
4. Create the planning branch `gsd/adr-0016-planning` from the synced `main`.
5. On that branch, create `gsd/README.md` with EXACTLY the content of Appendix A.
6. On that branch, prepend the banner in Appendix B to the very top of the existing root `CLAUDE.md` (keep all existing content below it).
7. Save this entire packet file verbatim as `gsd/packets/2026-06-12-repo-setup.md`.
8. Commit with a Conventional Commits message (e.g. `chore(gsd): scaffold planning branch for ADR-0016 work`) and push the branch to origin.
9. Create the feature branch `feat/adr-0016-phase-1-authority-alignment` from the same synced `main` and push it to origin with no commits of its own.
10. Report per Required output.
Out of scope

* ANY product code, documentation, or test changes.
* ANY commit directly to `main` (it moves only by fast-forward sync from upstream).
* ANY pull request creation, anywhere.
* Starting spec, plan, or implementation work.
Constraints / forbidden actions

* NEVER push to upstream (`saorsa-labs/x0x`). All pushes go to origin (the fork) only.
* No PR creation — PR creation always requires Jim's explicit approval, and none is approved.
* The planning branch (`gsd/adr-0016-planning`) must never be merged into any other branch.
* The feature branch must remain byte-identical to the synced `main` this session.
* Diff against upstream main must touch only intended paths (see Verification).
Verification required
Capture and paste the output of each:

1. `git remote -v` — origin is the fork, upstream is saorsa-labs.
2. `git log --oneline -1 origin/main` and `git log --oneline -1 upstream/main` — identical after sync.
3. `git diff upstream/main origin/feat/adr-0016-phase-1-authority-alignment --stat` — empty.
4. `git diff upstream/main origin/gsd/adr-0016-planning --name-only` — exactly three paths: `CLAUDE.md`, `gsd/README.md`, `gsd/packets/2026-06-12-repo-setup.md`.
For PRs / merge candidates / substantial completed work:

* Clean-context test required? No (housekeeping)
* Adversarial review required? No (housekeeping)
* Jim PR-raise approval required? N/A — no PR is permitted from this dispatch
* Reviewer independence requirement: N/A
* Validation evidence to inspect: the four command outputs above
Stop conditions
Stop and report (do not improvise) if:

* fork `main` contains commits that are not on upstream `main` (no force-pushing — report instead);
* any push is rejected or authentication fails;
* root `CLAUDE.md` does not exist on the branch (report; do not invent an alternative);
* a branch with either target name already exists;
* anything requires judgment beyond the numbered steps.
Required output
Return:

* role performed;
* whether the gsd plugin loaded (step 1);
* sources read;
* output/changes (branches created, files committed, hashes);
* verification evidence (the four command outputs);
* blockers/risks;
* recommended next checkpoint (expected: "await Phase 1 spec dispatch").
Appendix A — exact content for `gsd/README.md`

```markdown
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
   `main` (current: `feat/adr-0016-phase-1-authority-alignment`) and
   contains only the deliverable: code changes plus their documentation.
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

```

Appendix B — banner to prepend to root `CLAUDE.md` (planning branch only)

```markdown
> **GSD planning branch notice.** You are on `gsd/adr-0016-planning` — the
> planning home for the ADR-0016 group-authority work, NOT a development
> branch. Read `gsd/README.md` before doing anything. This branch never
> merges and never becomes a PR. Everything below this banner is the
> repo's own standard guidance.


```
