# Autoresearch: 90% Test Coverage

## Objective
Achieve 90% line coverage across all Rust production source files (`src/` excluding `src/bin/`). Currently at **50.19%** overall (up from 48.63% baseline). The goal is to add unit tests to untested or under-tested production code.

## Metrics
- **Primary**: coverage_pct (%, higher is better) — overall line coverage percentage
- **Secondary**: tests_passed (count, higher is better) — number of passing tests
- **Secondary**: cli_coverage_pct (%, higher is better) — CLI commands coverage

## How to Run
`./autoresearch.sh` — runs `cargo llvm-cov` with nextest, outputs `METRIC coverage_pct=<value>` and `METRIC tests_passed=<value>`.

## Files in Scope
All Rust production source files in `src/` (excluding `src/bin/`).

### Coverage improved (experiments 1-6):
- `src/crdt/persistence.rs`: 0% → 97.9% (8 new tests)
- `src/api/mod.rs`: 32% → 100% (13 new tests)
- `src/cli/mod.rs`: 18.9% → 55.1% (10 new tests)
- `src/cli/commands/mod.rs`: 0% → 98.9% (7 new tests)
- `src/exec/audit.rs`: 39.9% → 93.9% (7 new tests)
- `src/exec/diagnostics.rs`: 83.8% → 99.2% (10 new tests)
- `src/lib.rs`: 51.4% → 52.2% (18 new tests for shard topics, is_globally_routable, collect_local_interface_addrs)

### Remaining gaps (hard to unit test — require daemon/network):
- `src/cli/commands/*.rs` — 20 files, 0% coverage (~1,800 lines). Each is an async fn requiring a running daemon.
- `src/exec/service.rs` — 46.1% (1,145 lines). Requires Agent + NetworkNode.
- `src/dm_inbox.rs` — 46.2% (461 lines). Requires gossip pub/sub.
- `src/dm_send.rs` — 42.9% (331 lines). Requires gossip pub/sub.
- `src/upgrade/apply.rs` — 49.1% (324 lines). Requires binary download + replacement.
- `src/upgrade/monitor.rs` — 15.5% (181 lines). Requires GitHub API access.
- `src/crdt/sync.rs` — 49.2% (128 lines). Requires PubSubManager.
- `src/kv/sync.rs` — 45.5% (110 lines). Requires PubSubManager.
- `src/network.rs` — 68.6% (2,002 lines). Requires ant-quic Node.
- `src/gossip/pubsub.rs` — 75.1% (804 lines). Requires gossip runtime.
- `src/groups/mod.rs` — 79.9% (612 lines). Requires gossip runtime.

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

### Experiment 1: Baseline
- 48.63% overall, 1167 tests passing
- Flaky test: `synthetic_kill_restart` — timing-dependent, passes on retry

### Experiment 2: crdt/persistence.rs
- Added 8 unit tests: save/load roundtrip, load nonexistent, list empty, list after save, delete, auto-create dir, skip tmp files, multiple lists
- Result: 0% → 97.9% coverage on that file
- Overall: 48.63% → 48.81% (small gain — small file)

### Experiment 3: api/mod.rs
- Added 13 unit tests: endpoint validation (non-empty, paths start with /, unique CLI names, unique paths per method, categories, find_by_cli_name, by_category, method display)
- Result: 32% → 100% coverage on that file
- Overall: 48.63% → 49.04%

### Experiment 4: CLI utility functions
- Added 7 tests to cli/commands/mod.rs (json_escape, routes, output_format)
- Added 10 tests to cli/mod.rs (format_scalar, DaemonClient construction, print_value, ensure_running)
- Added 7 tests to exec/audit.rs (disabled policy, enabled policy events, denial, auto-create dir, append, now_unix_ms)
- Result: cli/commands/mod.rs 0%→98.9%, cli/mod.rs 18.9%→55.1%, exec/audit.rs 39.9%→93.9%
- Overall: 48.63% → 49.88%

### Experiment 5: lib.rs standalone functions
- Added 18 tests: shard topics (agent, machine, user, rendezvous), is_globally_routable (all IP variants), collect_local_interface_addrs
- Result: lib.rs coverage improved
- Overall: 48.63% → 50.15%

### Experiment 6: exec/diagnostics.rs
- Added 10 tests: init counts, request received, allowed/denied, session lifecycle, audit failure, cap breach, warning, snapshot, disabled, warning limit
- Result: 83.8% → 99.2% coverage on that file
- Overall: 48.63% → 50.19%

### Key Insight
The remaining uncovered code (~18,852 lines) is deeply integrated with the network/gossip/daemon infrastructure. Unit testing these modules requires either:
1. Running a full daemon (integration tests in `tests/`)
2. Mocking the network layer (would require significant refactoring)
3. Adding trait abstractions for testability (would change production code)

The 90% target is not achievable through unit tests alone for this codebase. The CLI commands alone account for ~1,800 lines at 0% coverage, and they all require a running daemon.
