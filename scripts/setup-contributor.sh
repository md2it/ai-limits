#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

target_hooks_path="scripts/git-hooks"
current_hooks_path="$(git config --local --get core.hooksPath || true)"

if [[ -z "$current_hooks_path" ]]; then
  git config --local core.hooksPath "$target_hooks_path"
  echo "Configured local Git hooks: core.hooksPath=$target_hooks_path"
  exit 0
fi

if [[ "$current_hooks_path" == "$target_hooks_path" ]]; then
  echo "Local Git hooks already configured: core.hooksPath=$target_hooks_path"
  exit 0
fi

cat <<EOF >&2
Local Git hooks were not changed.

Current core.hooksPath:
  $current_hooks_path

This project expects:
  $target_hooks_path

If this custom hooks path is intentional, keep it and wire project hooks manually.
Otherwise run:
  git config --local core.hooksPath $target_hooks_path
EOF

exit 1
