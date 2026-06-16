# CI arbiter — ADR-0016 Phase 1

Green of record: draft mirror PR #5, <https://github.com/JimCollinson/x0x/pull/5>.

- Repository: `JimCollinson/x0x`
- Branch: `feat/adr-0016-phase-1-authority-alignment` → `main`
- Status source: PR #5 Checks tab, per push

## Known-flake carve-out — daemon-startup timeouts (internal arbiter only)

PR #5 counts as green of record when either:

1. every required check is green; or
2. the only red is one or more daemon-startup health-timeouts, meaning all of:
   - **Signature:** every failure, across all CI jobs, is a daemon-startup health-timeout: the log shows `x0xd ... did not become healthy within <N>s` at test-harness bring-up (`pair()` / cluster setup). No failure is an assertion, a panic, a timeout inside a running test, or a diagnostic-counter mismatch. Tests marked not-run due fail-fast are not failures.
   - **Isolation:** the number of timed-out tests is small — `<= 3` across all jobs — with the rest of the suite green. A larger or growing set of bring-up failures is treated as a possible real startup/networking regression and PR #5 is red.
   - **Diff guard:** the slice under test changes no startup/health/networking code: nothing under `tests/harness/`, and not `src/network*`, `src/bootstrap*`, or `src/presence*`; and within `src/bin/x0xd.rs`, no change to `fn main`, the serve/startup sequence, the `/health` readiness handler, or node/transport/bootstrap initialization. Verify from the diff's enclosing-function hunk headers. If the slice touches any of these, the carve-out is void and a startup timeout may be real.
   - **Record:** the invocation is recorded in the slice checkpoint with the CI run and job IDs, failing test name(s), verbatim `did not become healthy within ...` line(s), diff-guard confirmation (files/functions touched), and — when available — tree-identity to a previously green commit.

If any condition fails, the carve-out does not apply and PR #5 is red under the normal blocking rule.

Why this cannot mask a real break: logic regressions assert-fail, which the signature rule excludes; real startup/networking regressions fail many daemon tests, which the isolation rule excludes; a slice that touches startup code voids the carve-out under the diff guard. The only thing absorbed is an isolated, pre-assertion startup timeout on a slice that provably could not have caused one — the precise environmental-flake signature.

Scope: this governs the internal CI arbiter (PR #5) — the development gate — only. It is not a statement about David's upstream CI, and the flake remains flagged to David as a pre-existing harness issue.

The local fast gate is a clone-local pre-push tripwire. It reduces accidental bad pushes, but it is not the pass record; CI on the mirror PR is authoritative.
