#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Verify a signed macOS app bundle or release zip.

Usage:
  scripts/verify-macos-app.sh [--notarization MODE] <path-to.app-or.zip>

Modes:
  full         (default) expect stapled notarization ticket
  submit-only  signed; notarization may still be in progress at Apple
  sign-only    signed only; Gatekeeper may warn until notarized

Examples:
  scripts/verify-macos-app.sh "AI Limits.app"
  scripts/verify-macos-app.sh --notarization full "AI Limits.app.zip"
EOF
}

NOTARIZATION_MODE="full"
TARGET=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --notarization)
      if [[ $# -lt 2 ]]; then
        echo "Missing value for --notarization" >&2
        usage >&2
        exit 1
      fi
      NOTARIZATION_MODE="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      if [[ -n "$TARGET" ]]; then
        echo "Unexpected argument: $1" >&2
        usage >&2
        exit 1
      fi
      TARGET="$1"
      shift
      ;;
  esac
done

if [[ -z "$TARGET" ]]; then
  usage >&2
  exit 1
fi

case "$NOTARIZATION_MODE" in
  full|submit-only|sign-only) ;;
  *)
    echo "Unknown notarization mode: $NOTARIZATION_MODE" >&2
    exit 1
    ;;
esac

if [[ ! -e "$TARGET" ]]; then
  echo "Path does not exist: $TARGET" >&2
  exit 1
fi

EXTRACT_DIR=""
cleanup() {
  if [[ -n "$EXTRACT_DIR" && -d "$EXTRACT_DIR" ]]; then
    rm -rf "$EXTRACT_DIR"
  fi
}
trap cleanup EXIT

if [[ "$TARGET" == *.zip ]]; then
  EXTRACT_DIR="$(mktemp -d)"
  ditto -x -k "$TARGET" "$EXTRACT_DIR"
  APP_PATH="$(find "$EXTRACT_DIR" -maxdepth 1 -name '*.app' -print -quit)"
  if [[ -z "$APP_PATH" ]]; then
    echo "No .app bundle found in zip: $TARGET" >&2
    exit 1
  fi
elif [[ "$TARGET" == *.app ]]; then
  APP_PATH="$TARGET"
else
  echo "Expected a .app bundle or .zip archive: $TARGET" >&2
  exit 1
fi

echo "Verifying macOS app: $APP_PATH"

codesign -dv "$APP_PATH"
codesign -d --entitlements - "$APP_PATH"
codesign --verify --deep --strict --verbose=4 "$APP_PATH"

case "$NOTARIZATION_MODE" in
  full)
    spctl --assess --type execute -vv "$APP_PATH"
    xcrun stapler validate "$APP_PATH"
    echo "macOS app is signed, notarized, and stapled."
    ;;
  submit-only)
    echo "Signed app verified. Notarization was submitted without waiting."
    echo "Check status with:"
    echo "  xcrun notarytool history --key ... --key-id ... --issuer ..."
    echo "After Accepted, staple locally or rerun workflow with macos_notarization=full:"
    echo "  xcrun stapler staple \"$APP_PATH\""
    ;;
  sign-only)
    echo "Signed only. Gatekeeper may warn until the app is notarized."
    ;;
esac
