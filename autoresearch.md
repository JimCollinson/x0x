# Autoresearch: 90% Test Coverage

## Objective
Achieve 90% line coverage across all Rust production source files (`src/` excluding `src/bin/`). Currently at **48.79%** overall. The goal is to add unit tests to untested or under-tested production code.

## Metrics
- **Primary**: coverage_pct (%, higher is better) — overall line coverage percentage
- **Secondary**: tests_passed (count, higher is better) — number of passing tests
- **Secondary**: cli_coverage_pct (%, higher is better) — CLI commands coverage

## How to Run
`./autoresearch.sh` — runs `cargo llvm-cov` with nextest, outputs `METRIC coverage_pct=<value>` and `METRIC tests_passed=<value>`.

## Files in Scope
All Rust production source files in `src/` (excluding `src/bin/`). Key files needing coverage:

### Zero-coverage files (highest priority):
- `src/cli/commands/*.rs` — 20 CLI command files, 0% coverage (~1,800 lines)
- `src/crdt/persistence.rs` — 0% coverage (53 lines)
- `src/cli/mod.rs` — 18.9% coverage (243 lines)

### Under-tested files (medium priority):
- `src/api/mod.rs` — 32% (only Method enum tested, not endpoint registry)
- `src/exec/audit.rs` — 39.9% (138 lines)
- `src/dm_send.rs` — 42.9% (331 lines)
- `src/dm_inbox.rs` — 46.2% (461 lines)
- `src/exec/service.rs` — 46.1% (1,145 lines)
- `src/lib.rs` — 51.4% (4,642 lines — biggest impact)
- `src/upgrade/apply.rs` — 49.1% (324 lines)
- `src/crdt/sync.rs` — 49.2% (128 lines)
- `src/network.rs` — 68.6% (2,002 lines)
- `src/gossip/pubsub.rs` — 75.1% (804 lines)
- `src/groups/mod.rs` — 79.9% (612 lines)

### Well-tested files (near 90%+):
- `src/identity.rs` — 81.8%
- `src/storage.rs` — 85.4%
- `src/presence.rs` — 88.2%
- `src/dm.rs` — 87.2%
- `src/contacts.rs` — 96.5%
- Various CRDT, MLS, groups files at 90-100%

## Off Limits
- `src/bin/` — binary entry points (not production library code)
- `tests/` — integration tests (we add unit tests in `src/`)
- `benches/`, `fuzz/`, `proofs/` — not relevant to coverage
- Do NOT modify existing test code unless adding new tests alongside
- Do NOT change production logic — only add tests

## Constraints
- All existing tests must continue to pass
- No new external dependencies
- Follow existing test patterns: `#[cfg(test)] mod tests { #![allow(clippy::unwrap_used)] ... }`
- Tests must be meaningful — test actual logic, not trivial getters
- If a test fails, write the failure details to `test-failures-report.md` and continue
- CLI tests can use the `EndpointDef` registry to test route definitions without a daemon

## What's Been Tried
- Baseline: 48.79% overall, 1167 tests passing
- CLI commands at 0% coverage — biggest gap
- `crdt/persistence.rs` at 0% — easy win with tempfile-based tests
- `exec/audit.rs` at 39.9% — needs tests for audit event writing
- `api/mod.rs` at 32% — needs tests for endpoint registry iteration and display
