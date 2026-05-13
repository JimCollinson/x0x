# Autoresearch Ideas — 90% Test Coverage

## Ideas Not Yet Pursued

- **Mock-based CLI command tests**: Create a mock HTTP server (using `axum` test helpers or `wiremock`) that the CLI commands can talk to. This would let us test all 20 CLI command files without a real daemon. The `DaemonClient` already accepts an `api_override` URL, so we just need a test server that returns appropriate JSON responses.

- **Property-based tests for CRDT convergence**: Add `proptest` tests for `crdt/sync.rs` and `kv/sync.rs` that generate random sequences of operations and verify convergence properties. These already exist in `tests/proptest_crdt.rs` but could be extended.

- **Integration tests for exec/service.rs**: The `ExecService` tests in `src/exec/service.rs` already create a real `Agent` with tempfile keys. These could be extended to test more code paths (denial scenarios, capability limits, concurrent sessions).

- **Test upgrade/apply.rs with tempfile**: The `AutoApplyUpgrader` could be tested by creating a mock binary, a mock archive, and verifying the backup/restore logic. The download step could be skipped by pre-placing the archive.

- **Test upgrade/monitor.rs with mock HTTP**: The `UpgradeMonitor` makes HTTP requests to GitHub. A test could use a local HTTP server to return mock release data.

- **Test dm_send.rs and dm_inbox.rs**: These require a running gossip network. Could be tested via the existing integration test infrastructure in `tests/harness/`.

- **Test network.rs ConnectionPool more thoroughly**: The `ConnectionPool` already has tests for eviction and capacity. Could add tests for concurrent access patterns and edge cases.

- **Test gossip/pubsub.rs more thoroughly**: Already at 75.1%. Could add tests for subscription management, message filtering, and error handling.

## Dead Ends

- **Unit testing individual CLI command files**: Each file is an `async fn` that takes `&DaemonClient` and makes HTTP calls. Without a mock HTTP layer, these can't be unit tested. The `DaemonClient` uses `reqwest::Client` directly with no trait abstraction.
- **Testing network.rs without ant-quic**: The `NetworkNode` wraps `ant_quic::Node` which requires real QUIC connections. No way to mock this without changing production code.
- **Testing gossip modules without gossip runtime**: All gossip modules depend on `saorsa-gossip-*` crates which require a running gossip runtime.
