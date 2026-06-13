# Verification evidence — Slice 1 last-admin invariant: local gate on real hardware

- **Date:** 2026-06-13
- **Role:** GSD Verifier — read-only on product/tests/harness. No source, test, or
  harness file was modified. Nothing was committed anywhere except this note,
  and only to `gsd/adr-0016-planning` on the `fork` remote (`JimCollinson/x0x`).
  `upstream` (`saorsa-labs/x0x`) was never touched.
- **Machine:** Apple Silicon (`aarch64-apple-darwin`), Darwin 25.2.0 · `rustc 1.95.0` · `cargo-nextest 0.9.137`
- **Base (verified shared parent of both branches):** `189b89c`
- **Canonical:** `feat/adr-0016-phase-1-authority-alignment` @ **`903cf8d`** (exactly one commit on base)
- **Alt:** **`ed2f9a3`** (`feat(adr-0016-slice-1): enforce last-admin invariant`, exactly one commit on base) = `origin/exp/slice-1-alt`
  - ⚠️ **The local `exp/slice-1-alt` branch ref is stale at the base `189b89c`** ("[behind 1]"). The actual alt slice implementation is `ed2f9a3`. This run assessed `ed2f9a3` (its parent verified == `189b89c`).

---

## Bottom line

### (a) Does alt's full suite pass WITHOUT the `X0XD_TEST_BINARY` wrapper? → **NO.**

The real, unmodified `cargo nextest run --all-features --workspace` does **not** pass on
this machine without the bootstrap-stripping wrapper. Authoritative `--no-fail-fast`
result for alt: **`1713 tests run: 1707 passed, 6 failed, 160 skipped`**. All 6 failures
are environmental/pre-existing — **none is an alt-slice defect**:

- **5 deterministic daemon-mesh failures** in `named_group_join_metadata_event` — the
  2-node mDNS pair mesh never forms (`zero peers after 30s`). This file is **untouched by
  the slice**, fails identically on canonical, and fails identically at clean baseline.
- **1 non-deterministic timing flake** — a pre-existing `<100ms` / `within_500ms`
  assertion that tips over under full-suite CPU/IO load. The *specific* test varies between
  runs (`test_agent_creation_performance` in one run, `direct_send_reissues_on_replaced_connection_within_500ms`
  in another); each passes 5/5 when run isolated on a quiet machine.

The alt checkpoint's `1713 passed, 1713 passed` was achievable **only** because the wrapper
supplies an explicit-localhost bootstrap that lets the pair nodes mesh without mDNS.
**Alt's own slice gates pass cleanly with no wrapper:** fmt ✓, clippy ✓ (0 warnings),
`test(last_admin)` 10/10, `test(last_admin) --run-ignored all` 10/10.

### (b) Is `forged_member_joined_admin_role_or_secret_is_rejected` a genuine environmental mesh failure (same at baseline)? → **YES.**

It fails identically on **canonical**, **alt**, and clean baseline **`189b89c`** (neither
slice applied), every time with:

```
thread 'forged_member_joined_admin_role_or_secret_is_rejected' panicked at tests/harness/src/cluster.rs:430:17:
[cluster] FATAL: pair-alice-<port> has zero peers after 30s — mesh is disconnected.
```

(`pair-alice-51934` canonical · `pair-alice-10241` alt · `pair-alice-32139` baseline — only
the random ephemeral port differs.) The 2-node mDNS pair mesh never forms; the test dies in
`pair()` setup before any group/slice logic runs. **Environmental, not a regression** from
either branch.

---

## Per-branch verdict — *do the slice's own gates pass on real hardware with a real daemon?*

- **Canonical `903cf8d`: YES.** fmt ✓ · clippy `-D warnings` ✓ (0 warnings) ·
  `test(last_admin)` **18/18** ✓ · `test(last_admin) --run-ignored all` **19/19** ✓ (incl.
  the `#[ignore]`d single-daemon REST test `last_admin_rest_self_demote_returns_409_exact_string`).
  The only full-suite failures are the 5 pre-existing environmental
  `named_group_join_metadata_event` pair-mesh tests, which also fail at baseline. No slice defect.

- **Alt `ed2f9a3`: YES for the slice's own gates** (fmt ✓ · clippy ✓ 0 warnings ·
  `test(last_admin)` **10/10** ✓ · `--run-ignored all` **10/10** ✓) — **but the full
  unmodified suite does NOT pass without the wrapper.** Its real result is 6 failures = the
  same 5 environmental mesh tests + 1 pre-existing timing flake. No alt-specific defect was
  found; the green suite reported in the alt checkpoint is wrapper-dependent and does not
  reproduce unmodified.

---

## Method & environment (transparency)

- **No wrapper anywhere.** `X0XD_TEST_BINARY` was **unset** for every run (recorded as
  `<unset>` at the top of each run log). The flag-stripping wrapper described in the alt
  checkpoint was not used in any run, including the full suite.
- **Real daemon, the legitimate way.** With `X0XD_TEST_BINARY` unset, the harness's
  `find_x0xd_binary()` (`tests/harness/src/cluster.rs:442`) resolves `<root>/target/release/x0xd`.
  Each branch was therefore built with a genuine `cargo build --release --bin x0xd` (a real
  cargo build, **not** a wrapper); the daemon still receives `--no-hard-coded-bootstrap`
  (`cluster.rs:547`) exactly as the harness intends. This matches the repo's own
  `tests/CLAUDE.md` ("Build release binary" before daemon/e2e tests).
- **Clean checkouts.** Canonical ran in the existing worktree `.claude/worktrees/adr0016-build`
  (clean @ `903cf8d`). Alt and baseline ran in a fresh detached worktree, checked out clean at
  `ed2f9a3` then `189b89c` (`git status` clean at each; baseline reused the warm target via an
  incremental rebuild back to base — source was a clean detached `189b89c`).
- **Lockfile (env-only, gitignored).** `Cargo.toml` is byte-identical across base/canonical/alt,
  so `Cargo.lock` was seeded with the working `time = 0.3.47` pin. A fresh resolve selects
  `time 0.3.48`, which fails the `rustc 1.95.0` coherence check — the documented machine quirk.
  No source/git change.
- **No product/test/harness modification.** `audit.jsonl` was restored after every run and
  never staged. Disk pressure during the alt cold build was handled by **macOS auto-purging an
  APFS Time Machine local snapshot under load** — no manual snapshot deletion was performed.

---

## Side-by-side gate results (exact command outputs)

All commands run with `X0XD_TEST_BINARY` **unset**. nextest summary strings quoted verbatim.

| Command | Canonical `903cf8d` | Alt `ed2f9a3` |
|---|---|---|
| `cargo build --all-features` | exit 0 (95s) | exit 0 (287s) |
| `cargo build --release --bin x0xd` | exit 0 (296s) | exit 0 (467s) |
| `cargo fmt --all -- --check` | **PASS** (exit 0, no output) | **PASS** (exit 0, no output) |
| `cargo clippy --all-targets --all-features -- -D warnings` | **PASS** — exit 0, 0 warnings | **PASS** — exit 0, 0 warnings |
| `cargo nextest run --all-features --workspace` *(plain, default fail-fast)* | exit 100 — `1716/1721 tests run: 1715 passed, 1 failed, 161 skipped`; first failure `forged_member…`, then `5/1721 tests were not run due to test failure` | exit 100 — `1262/1713 tests run: 1261 passed, 1 failed, 160 skipped`; first failure `test_agent_creation_performance` (timing flake), then `451/1713 tests were not run due to test failure` |
| `cargo nextest run --all-features --workspace --no-fail-fast` *(authoritative enumeration)* | exit 100 — `1721 tests run: 1716 passed, 5 failed, 161 skipped` | exit 100 — `1713 tests run: 1707 passed, 6 failed, 160 skipped` |
| `cargo nextest run --all-features -E 'test(last_admin)'` | exit 0 — `18 tests run: 18 passed, 1856 skipped` | exit 0 — `10 tests run: 10 passed, 1855 skipped` |
| `cargo nextest run --all-features -E 'test(last_admin)' --run-ignored all` | exit 0 — `19 tests run: 19 passed, 1855 skipped` | exit 0 — `10 tests run: 10 passed, 1855 skipped` |
| `cargo nextest run --all-features -E 'test(forged_member…)'` | exit 100 — `1 test run: 0 passed, 1 failed`; `pair-alice-51934 … zero peers after 30s` | exit 100 — `1 test run: 0 passed, 1 failed`; `pair-alice-10241 … zero peers after 30s` |

> Note: the test totals differ (1721 vs 1713) because the two implementations carry different
> test files — see "Test-count delta" below. This is a design difference, not a defect.

---

## (a) Alt full-suite failure enumeration (`--no-fail-fast`) + flake classification

`1713 tests run: 1707 passed, 6 failed, 160 skipped`. The 6 failures:

1. `x0x::x0x_0041_prefer_newest_test direct_send_reissues_on_replaced_connection_within_500ms` — **timing flake**, `tests/x0x_0041_prefer_newest_test.rs:290`
2. `x0x::named_group_join_metadata_event forged_member_joined_admin_role_or_secret_is_rejected` — mesh
3. `x0x::named_group_join_metadata_event issued_invite_secret_is_recorded_on_inviter` — mesh
4. `x0x::named_group_join_metadata_event member_joined_event_is_idempotent` — mesh
5. `x0x::named_group_join_metadata_event member_joined_event_propagates_to_inviter` — mesh
6. `x0x::named_group_join_metadata_event tampered_member_joined_signed_role_is_rejected_before_role_policy` — mesh

**The 5 mesh failures (#2–#6) are exactly the set that fails on canonical** (canonical's
`--no-fail-fast` failed the identical 5). They live in `named_group_join_metadata_event.rs`,
which neither slice modifies.

**Timing-flake classification (proven, not assumed):**

| Flaky test | Location (identical at base/alt/canonical) | Observed failure | Isolated re-runs on quiet machine |
|---|---|---|---|
| `test_agent_creation_performance` | `comprehensive_integration.rs:458` (`assert … "Agent creation should be < 100ms"`, line 460) | alt **plain** run: `Agent creation time: 100.870708ms` (0.87 ms over) | **5/5 PASS** (0.03–0.07 s) |
| `direct_send_reissues_on_replaced_connection_within_500ms` | `x0x_0041_prefer_newest_test.rs:290` (`within_500ms`) | alt **--no-fail-fast** run: FAIL [0.841s] | **5/5 PASS** (0.45–0.63 s) |

Both assertion lines are byte-identical at `189b89c`, `ed2f9a3`, and `903cf8d` (`git grep`
confirmed) → pre-existing, untouched by either slice. In the alt plain run the perf test
flaked and the `direct_send` test passed; in the alt `--no-fail-fast` run the perf test
passed and the `direct_send` test flaked — i.e. *which* timing test tips over is
non-deterministic, the signature of load-sensitivity, not a deterministic regression.
Canonical happened to pass both during its run (quieter machine state / warm build).

---

## (b) `forged_member…` three-way comparison

| Tree | Slice applied? | Result | FATAL |
|---|---|---|---|
| Canonical `903cf8d` | last-admin (canonical) | FAIL | `pair-alice-51934 has zero peers after 30s` |
| Alt `ed2f9a3` | last-admin (alt) | FAIL | `pair-alice-10241 has zero peers after 30s` |
| **Baseline `189b89c`** (detached, clean) | **none** (`last_admin_rest.rs` & `last_admin_invariant.rs` absent) | FAIL | `pair-alice-32139 has zero peers after 30s` |

Same panic, same location (`tests/harness/src/cluster.rs:430`), same mechanism on all three —
the test dies in 2-node `pair()` mesh setup before any group logic. Failing identically at
clean `189b89c` proves it is **environmental, not introduced by either slice.**

---

## Notes / follow-ups (not blockers; for Jim's awareness)

1. **Root cause of (a) and the wrapper's existence** — the harness starts daemons with
   `--no-hard-coded-bootstrap` (`cluster.rs:547`), which clears the generated localhost
   `bootstrap_peers`; on this Mac the pair/trio nodes then can't mesh because mDNS does not
   form the local mesh. Both checkpoints already flag the in-repo fix (don't pass the flag to
   nodes that have explicit local bootstrap peers, or split the flag semantics). Until that
   lands, the multi-daemon mesh tests will fail unmodified on this hardware regardless of slice.
2. **Pre-existing flaky timing tests on this hardware:** `test_agent_creation_performance`
   (`<100ms`) and `direct_send_reissues_on_replaced_connection_within_500ms` (`within_500ms`).
   Both pass isolated; both flake under full-suite load. Candidates for threshold relaxation or
   `quic-localhost`-style serialization.
3. **Test-count delta is a design difference, not a defect.** Canonical's last-admin coverage
   is 18 non-daemon tests (8 in-crate unit + 11 in `tests/last_admin_invariant.rs`) plus one
   `#[ignore]`d single-daemon REST test (→ 19 under `--run-ignored`). Alt's is 9 in-crate unit
   tests plus one **non-`#[ignore]`d** single-daemon REST test
   (`last_admin_rest_precheck_exact_string_for_remove_ban_and_demote`) (→ 10). Same invariant,
   different decomposition; both REST assertions are single-daemon (no mesh) and pass unmodified.
4. **Scope honoured.** This was a verification pass only. No product/test/harness change; the
   `x0xd.rs` `last_admin_precheck_*` unit tests remain dead-by-config (`test = false`) on
   canonical, as the canonical checkpoint already noted — not re-litigated here.

---

*Raw per-command logs captured under `/tmp/x0x-verify/logs/{canonical,alt,baseline}/` at run
time (ephemeral). Verifier: Claude Code (Opus 4.8), 2026-06-13.*
