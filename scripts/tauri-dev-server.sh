#!/usr/bin/env sh
set -eu

PORT="${AI_LIMITS_TAURI_DEV_PORT:-1420}"
HOST="${AI_LIMITS_TAURI_DEV_HOST:-127.0.0.1}"
PROJECT_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"

stop_project_server() {
  if ! command -v lsof >/dev/null 2>&1; then
    return 0
  fi

  pids="$(lsof -nP -tiTCP:"$PORT" -sTCP:LISTEN 2>/dev/null || true)"
  if [ -z "$pids" ]; then
    return 0
  fi

  for pid in $pids; do
    command_line="$(ps -p "$pid" -o command= 2>/dev/null || true)"
    command_line_lc="$(printf '%s' "$command_line" | tr '[:upper:]' '[:lower:]')"
    cwd_line="$(lsof -nP -a -p "$pid" -d cwd 2>/dev/null | awk 'NR==2 {print $NF}' || true)"

    case "$command_line_lc" in
      *"python"*"-m http.server"*"$PORT"*"--directory frontend"*|*"python"*"-m http.server"*"$PORT"*"--directory"*"${PROJECT_ROOT}/frontend"*)
        kill "$pid" 2>/dev/null || true
        ;;
      *"python"*"-m http.server"*"$PORT"*)
        if [ "$cwd_line" = "$PROJECT_ROOT" ]; then
          kill "$pid" 2>/dev/null || true
        fi
        ;;
    esac
  done

  sleep 0.2
}

stop_project_server

exec python3 -m http.server "$PORT" --bind "$HOST" --directory "$PROJECT_ROOT/frontend"
