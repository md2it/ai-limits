#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: scripts/embed-macos-widget.sh <unsigned|signed> <path-to.app> [signing-identity]"
}

if [[ $# -lt 2 || $# -gt 3 ]]; then
  usage >&2
  exit 1
fi

MODE="$1"
APP_PATH="$2"
SIGNING_IDENTITY="${3:-}"
PROJECT_PATH="src-macos-widgets/AI Limits Widgets.xcodeproj"
DERIVED_DATA="target/macos-widgets"
WIDGET_PATH="$DERIVED_DATA/Build/Products/Release/AI Limits Widgets.appex"
EMBEDDED_PATH="$APP_PATH/Contents/PlugIns/AI Limits Widgets.appex"
WIDGET_ARCHS="${WIDGET_ARCHS:-$(uname -m)}"

if [[ "$MODE" != "unsigned" && "$MODE" != "signed" ]]; then
  usage >&2
  exit 1
fi

if [[ ! -d "$APP_PATH" ]]; then
  echo "App bundle does not exist: $APP_PATH" >&2
  exit 1
fi

BUILD_SETTINGS=(
  -project "$PROJECT_PATH"
  -scheme "AI Limits Widgets"
  -configuration Release
  -derivedDataPath "$DERIVED_DATA"
  ARCHS="$WIDGET_ARCHS"
  ONLY_ACTIVE_ARCH=NO
)

if [[ "$MODE" == "unsigned" ]]; then
  BUILD_SETTINGS+=(CODE_SIGNING_ALLOWED=NO)
else
  if [[ -z "$SIGNING_IDENTITY" ]]; then
    echo "Signing identity is required in signed mode." >&2
    exit 1
  fi
  if [[ -z "${APPLE_TEAM_ID:-}" || -z "${WIDGET_PROVISIONING_PROFILE_SPECIFIER:-}" ]]; then
    echo "APPLE_TEAM_ID and WIDGET_PROVISIONING_PROFILE_SPECIFIER are required in signed mode." >&2
    exit 1
  fi
  BUILD_SETTINGS+=(CODE_SIGN_STYLE=Manual CODE_SIGN_IDENTITY="$SIGNING_IDENTITY" DEVELOPMENT_TEAM="$APPLE_TEAM_ID" PROVISIONING_PROFILE_SPECIFIER="$WIDGET_PROVISIONING_PROFILE_SPECIFIER" OTHER_CODE_SIGN_FLAGS="--timestamp --options runtime")
fi

xcodebuild "${BUILD_SETTINGS[@]}" build
mkdir -p "$(dirname "$EMBEDDED_PATH")"
ditto "$WIDGET_PATH" "$EMBEDDED_PATH"

if [[ "$MODE" == "signed" ]]; then
  codesign --force --timestamp --options runtime --entitlements src-tauri/Entitlements.plist --sign "$SIGNING_IDENTITY" "$APP_PATH"
fi

echo "Embedded Widget Extension: $EMBEDDED_PATH"
