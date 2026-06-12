#!/usr/bin/env bash
# GSD plugin bootstrap for Claude Code cloud sessions on this repo.
#
# Paste this whole file into the x0x cloud environment's "Setup script"
# field (code.claude.com -> environment configuration). It runs in the
# container BEFORE Claude Code starts, on every branch, and installs the
# plugin via the official CLI.
#
# jim-ops is private: set a GITHUB_TOKEN environment variable (read
# access to the marketplace repo) in the same environment configuration.

set -u

MARKETPLACE_REPO="JimCollinson/jim-ops"
# <plugin name>@<marketplace 'name' field in .claude-plugin/marketplace.json>
# — correct this line if your names differ.
PLUGIN_SPEC="gsd@jim-ops"

if ! claude plugin marketplace add "$MARKETPLACE_REPO"; then
  # Plain add failed (private repo without ambient credentials) — retry
  # with the token embedded in the clone URL.
  if [ -n "${GITHUB_TOKEN:-}" ]; then
    claude plugin marketplace add \
      "https://x-access-token:${GITHUB_TOKEN}@github.com/${MARKETPLACE_REPO}.git" || {
      echo "ERROR: could not add marketplace ${MARKETPLACE_REPO}" >&2
      exit 1
    }
  else
    echo "ERROR: could not add marketplace ${MARKETPLACE_REPO} — private repo and GITHUB_TOKEN not set." >&2
    exit 1
  fi
fi

claude plugin install "$PLUGIN_SPEC" || {
  echo "ERROR: could not install ${PLUGIN_SPEC} — check the name with 'claude plugin marketplace list'." >&2
  exit 1
}

claude plugin list
