#!/usr/bin/env bash
set -euo pipefail

APP_PATH="target/release/bundle/macos/AI Limits.app"

npm exec tauri -- build --bundles app
scripts/embed-macos-widget.sh unsigned "$APP_PATH"

echo "Unsigned local macOS app with Widget Extension: $APP_PATH"
