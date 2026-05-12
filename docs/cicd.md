# CI/CD

Five workflows in `.github/workflows/`:

- **ci.yml**: fmt, clippy, nextest, doc (all jobs symlink `ant-quic` and `saorsa-gossip` from `.deps/`)
- **security.yml**: `cargo audit`
- **release.yml**: Multi-platform builds (7 targets), macOS code signing, publishes to crates.io. Also generates `release-manifest.json` and signature for the self-update system (see [`upgrade-system.md`](upgrade-system.md)).
- **build.yml**: PR validation
- **sign-skill.yml**: GPG-signs `SKILL.md`
