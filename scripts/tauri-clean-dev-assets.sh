#!/usr/bin/env sh
set -eu

PROJECT_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"

clean_codegen_assets() {
  for profile in debug release; do
    build_dir="$PROJECT_ROOT/target/$profile/build"
    if [ ! -d "$build_dir" ]; then
      continue
    fi

    find "$build_dir" -path '*/out/tauri-codegen-assets' -type d -prune -exec rm -rf {} +
  done
}

clean_codegen_assets
