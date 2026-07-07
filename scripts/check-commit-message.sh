#!/usr/bin/env bash
set -euo pipefail

allowed_pattern='^(feat|fix|docs|chore): .+'

check_subject() {
  local subject="$1"

  if [[ "$subject" =~ $allowed_pattern ]]; then
    return 0
  fi

  echo "::warning::Commit message should start with feat:, fix:, docs:, or chore: ${subject}"
  return 1
}

if [[ "$#" -eq 0 ]]; then
  echo "Usage: $0 <commit-message-file|commit-range>" >&2
  exit 2
fi

target="$1"
failed=0

if [[ -f "$target" ]]; then
  subject="$(sed -n '1p' "$target")"
  check_subject "$subject" || failed=1
else
  while IFS= read -r subject; do
    [[ -n "$subject" ]] || continue
    check_subject "$subject" || failed=1
  done < <(git log --format=%s "$target")
fi

exit "$failed"
