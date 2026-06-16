# CI arbiter — ADR-0016 Phase 1

Green of record: draft mirror PR #5, <https://github.com/JimCollinson/x0x/pull/5>.

- Repository: `JimCollinson/x0x`
- Branch: `feat/adr-0016-phase-1-authority-alignment` → `main`
- Status source: PR #5 Checks tab, per push

## Known-flake carve-out (internal arbiter only)

PR #5 counts as green of record when either:

1. every required check is green; or
2. the only red is the Rust test job, failing solely on the known environmental mesh flake — meaning all of:
   - no required check is red except the Rust test job;
   - every failing test in that job is (i) in the enumerated known-flaky set and (ii) failed on a daemon-startup health-timeout — the log contains `did not become healthy within` for an `x0xd` instance — not an assertion, panic, or diagnostic-counter mismatch in the test body;
   - tests marked not-run due fail-fast are not failures;
   - the enumerated known-flaky set is exactly `named_group_join_metadata_event::forged_member_joined_admin_role_or_secret_is_rejected`, extendable only by naming a specific test — never a wildcard or pattern;
   - the invocation is recorded in the slice checkpoint with the CI run ID, failing test name(s), the verbatim `did not become healthy within ...` line, and — when available — confirmation the head's tree is identical to a previously green commit (`git diff --quiet <green-commit>`).

If any condition fails — more than the test job red, a non-enumerated test failing, an assertion/logic failure of an enumerated test, or any startup-timeout on a daemon test outside the enumerated set — the carve-out does not apply and PR #5 is red under the normal blocking rule.

Why this cannot mask a real break: a genuine logic regression makes the test run and assert-fail, which condition 2 excludes; a genuine startup/networking regression would fail many daemon tests, including ones outside the enumerated set, which the named-test and sole-failure conditions exclude. The carve-out only absorbs an isolated, pre-assertion startup timeout on a named environmental test — the exact narrow signature of this flake.

Scope: this governs the internal CI arbiter (PR #5) — the development gate — only. It is not a statement about David's upstream CI, and the flake remains flagged to David as a pre-existing harness issue.

The local fast gate is a clone-local pre-push tripwire. It reduces accidental bad pushes, but it is not the pass record; CI on the mirror PR is authoritative.
