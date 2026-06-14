# Branch globs to gate (space-separated).
GSD_GATE_BRANCHES="feat/* exp/*"

# Ordered fast-gate commands; run in order, first non-zero blocks the push.
# Keep FAST (local tripwire). CI runs the full/slow/cross-platform set.
GSD_GATE_COMMANDS=(
  "cargo fmt --all -- --check"
  "cargo clippy --all-targets --all-features -- -D warnings"
)
