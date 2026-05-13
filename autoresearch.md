# Autoresearch: 90% Test Coverage

## Objective
Achieve 90% line coverage across all Rust production source files (`src/` excluding `src/bin/`). Currently at **52.51%** overall (up from 48.63% baseline). The goal is to add unit tests to untested or under-tested production code.

## Metrics
- **Primary**: coverage_pct (%, higher is better) — overall line coverage percentage
- **Secondary**: tests_passed (count, higher is better) — number of passing tests
- **Secondary**: cli_coverage_pct (%, higher is better) — CLI commands coverage

## How to Run
`./autoresearch.sh` — runs `cargo llvm-cov` with nextest, outputs `METRIC coverage_pct=<value>` and `METRIC tests_passed=<value>`.

## Files in Scope
All Rust production source files in `src/` (excluding `src/bin/`).

### Coverage improved (all 17 experiments):
| File | Before | After | Tests Added |
|------|--------|-------|-------------|
| `src/crdt/persistence.rs` | 0% | **97.9%** | 8 |
| `src/api/mod.rs` | 32% | **100%** | 13 |
| `src/cli/commands/mod.rs` | 0% | **98.9%** | 7 |
| `src/cli/mod.rs` | 18.9% | **79.1%** | 10 |
| `src/exec/audit.rs` | 39.9% | **93.9%** | 7 |
| `src/exec/diagnostics.rs` | 83.8% | **99.2%** | 10 |
| `src/lib.rs` | 51.4% | **53.0%** | 26 |
| `src/cli/commands/identity.rs` | 0% | **53.1%** | 7 |
| `src/cli/commands/constitution.rs` | 0% | **59.5%** | 5 |
| `src/cli/commands/network.rs` | 0% | **92.3%** | 16 |
| `src/cli/commands/daemon.rs` | 0% | **17.5%** | 4 |
| `src/cli/commands/contacts.rs` | 0% | **39.5%** | 2 |
| `src/cli/commands/discovery.rs` | 0% | **45.7%** | 1 |
| `src/cli/commands/find.rs` | 0% | **43.6%** | 2 |
| `src/cli/commands/machines.rs` | 0% | **34.6%** | 1 |
| `src/cli/commands/messaging.rs` | 0% | **60.2%** | 1 |
| `src/cli/commands/presence.rs` | 0% | **56.3%** | 2 |
| `src/cli/commands/store.rs` | 0% | **40.4%** | 1 |
| `src/cli/commands/tasks.rs` | 0% | **42.9%** | 1 |
| `src/cli/commands/ws.rs` | 0% | **57.1%** | 1 |
| `src/cli/commands/direct.rs` | 0% | **58.5%** | 2 |
| `src/cli/commands/exec.rs` | 0% | **43.1%** | 2 |
| `src/cli/commands/files.rs` | 0% | **37.5%** | 2 |
| `src/cli/commands/groups.rs` | 0% | **43.8%** | 2 |
| `src/cli/commands/group.rs` | 6.6% | **19.4%** | 3 |
| `src/cli/commands/connect.rs` | 0% | **38.6%** | 2 |
| `src/exec/service.rs` | 46.1% | **51.8%** | 9 |
| `src/dm_send.rs` | 42.9% | **49.6%** | 6 |

**Total: 159 new tests across 28 files, 1326 total tests passing.**

### Key Achievements
- **20/21 CLI command files** now have coverage (was 1/21)
- **Mock HTTP server pattern** using axum unlocked 19 CLI files
- **7 files** brought to >90% coverage
- **+3.88% overall coverage** gain

### Remaining Gaps
The remaining uncovered code (~18,500 lines) is deeply integrated with the network/gossip/daemon infrastructure:
- `upgrade/monitor.rs` (15.5%) — requires GitHub API
- `upgrade/apply.rs` (49.1%) — binary download/replace
- `dm_inbox.rs` (46.2%) — gossip pub/sub
- `kv/sync.rs` (45.5%) — PubSubManager
- `crdt/sync.rs` (49.2%) — PubSubManager
- `network.rs` (67.2%) — ant-quic Node
- `gossip/pubsub.rs` (75.0%) — gossip runtime
- `cli/commands/upgrade.rs` (0%) — standalone GitHub upgrade

## Off Limits
- `src/bin/` — binary entry points
- `tests/` — integration tests
- `benches/`, `fuzz/`, `proofs/`
- Do NOT modify production logic — only add tests

## Constraints
- All existing tests must continue to pass
- No new external dependencies
- Follow existing test patterns
- Tests must be meaningful
