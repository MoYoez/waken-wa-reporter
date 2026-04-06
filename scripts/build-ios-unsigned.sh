#!/bin/sh

set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
PROJECT_DIR="$ROOT_DIR/src-tauri/gen/apple"
DERIVED_DATA_DIR="$PROJECT_DIR/build"
OUTPUT_DIR="$PROJECT_DIR/output"
APP_BUILD_DIR="$OUTPUT_DIR/Release-iphoneos"
PAYLOAD_DIR="$OUTPUT_DIR/Payload"
IPA_PATH="$OUTPUT_DIR/waken-wa-preview-ui-unsigned.ipa"
SCHEME="waken-wa-preview-ui_iOS"
PROJECT_PATH="$PROJECT_DIR/waken-wa-preview-ui.xcodeproj"

if [ ! -d "$PROJECT_DIR" ]; then
  echo "iOS project has not been generated. Run 'pnpm tauri ios init --ci' first." >&2
  exit 1
fi

rm -rf "$DERIVED_DATA_DIR" "$OUTPUT_DIR"
mkdir -p "$APP_BUILD_DIR" "$PAYLOAD_DIR"

xcodebuild \
  -project "$PROJECT_PATH" \
  -scheme "$SCHEME" \
  -configuration Release \
  -sdk iphoneos \
  -derivedDataPath "$DERIVED_DATA_DIR" \
  CONFIGURATION_BUILD_DIR="$APP_BUILD_DIR" \
  CODE_SIGNING_ALLOWED=NO \
  CODE_SIGNING_REQUIRED=NO \
  CODE_SIGN_IDENTITY="" \
  DEVELOPMENT_TEAM="" \
  PROVISIONING_PROFILE_SPECIFIER="" \
  build

APP_PATH="$(find "$APP_BUILD_DIR" -maxdepth 1 -type d -name '*.app' | head -n 1)"

if [ -z "$APP_PATH" ]; then
  echo "Unsigned iOS app bundle was not produced." >&2
  exit 1
fi

cp -R "$APP_PATH" "$PAYLOAD_DIR/"

(
  cd "$OUTPUT_DIR"
  /usr/bin/zip -qry "$(basename "$IPA_PATH")" Payload
)

echo "Unsigned app: $APP_PATH"
echo "Unsigned IPA: $IPA_PATH"
