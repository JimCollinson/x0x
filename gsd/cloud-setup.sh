#!/usr/bin/env bash
# GSD plugin bootstrap for Claude Code cloud sessions on this repo.
#
# Paste this whole file into the x0x cloud environment's "Setup script"
# field (code.claude.com -> environment configuration). It runs as root
# in the container BEFORE Claude Code starts, on every branch.
#
# Requires: a GITHUB_TOKEN environment variable (set in the same
# environment configuration) with read access to the marketplace repo
# below. Without it the script skips quietly so sessions still start.
#
# What it does:
#   1. Clones the marketplace repo and reads .claude-plugin/marketplace.json,
#      so marketplace/plugin names are never hardcoded here.
#   2. Writes ~/.claude/settings.json with extraKnownMarketplaces +
#      enabledPlugins for every plugin in the marketplace (full-plugin
#      route; undocumented for cloud user scope -- verify once).
#   3. Copies each plugin's skills/ into ~/.claude/skills/ (proven route:
#      container-local user skills are loaded by cloud sessions).

set -u

MARKETPLACE_REPO="JimCollinson/jim-ops"
CLONE_DIR="$HOME/.gsd-marketplace"

if [ -z "${GITHUB_TOKEN:-}" ]; then
  echo "WARNING: GITHUB_TOKEN not set — skipping GSD plugin bootstrap." >&2
  exit 0
fi

rm -rf "$CLONE_DIR"
if ! git clone --quiet --depth 1 \
    "https://x-access-token:${GITHUB_TOKEN}@github.com/${MARKETPLACE_REPO}.git" \
    "$CLONE_DIR"; then
  echo "ERROR: could not clone ${MARKETPLACE_REPO} — GSD plugin will be unavailable." >&2
  exit 1
fi

python3 - "$CLONE_DIR" "$MARKETPLACE_REPO" <<'PY'
import json, os, shutil, sys

clone_dir, repo = sys.argv[1], sys.argv[2]
with open(os.path.join(clone_dir, ".claude-plugin", "marketplace.json")) as f:
    mp = json.load(f)

mp_name = mp["name"]
plugins = mp.get("plugins", [])
print(f"Marketplace '{mp_name}' with plugins: {[p['name'] for p in plugins]}")

# 1) Register marketplace + enable its plugins in container user settings.
settings_path = os.path.expanduser("~/.claude/settings.json")
os.makedirs(os.path.dirname(settings_path), exist_ok=True)
settings = {}
if os.path.exists(settings_path):
    with open(settings_path) as f:
        settings = json.load(f)
settings.setdefault("extraKnownMarketplaces", {})[mp_name] = {
    "source": {"source": "github", "repo": repo}
}
enabled = settings.setdefault("enabledPlugins", {})
for p in plugins:
    enabled[f"{p['name']}@{mp_name}"] = True
with open(settings_path, "w") as f:
    json.dump(settings, f, indent=2)
print(f"Wrote {settings_path}")

# 2) Proven fallback: install each plugin's skills as user-level skills.
skills_root = os.path.expanduser("~/.claude/skills")
os.makedirs(skills_root, exist_ok=True)
for p in plugins:
    src = p.get("source")
    if not isinstance(src, str):
        continue  # only plugins vendored inside the marketplace repo
    skills_dir = os.path.join(clone_dir, os.path.normpath(src), "skills")
    if not os.path.isdir(skills_dir):
        continue
    for skill in os.listdir(skills_dir):
        s = os.path.join(skills_dir, skill)
        if os.path.isdir(s) and os.path.exists(os.path.join(s, "SKILL.md")):
            dest = os.path.join(skills_root, skill)
            shutil.rmtree(dest, ignore_errors=True)
            shutil.copytree(s, dest)
            print(f"Installed skill: {skill}")
PY
