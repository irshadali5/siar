#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

export ANDROID_NDK_ROOT="${ANDROID_NDK_ROOT:-/opt/android-ndk}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-/opt/android-ndk}"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="${CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER:-/opt/android-ndk/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android34-clang}"
export GRADLE_USER_HOME="${GRADLE_USER_HOME:-$ROOT_DIR/target/gradle-user-home}"

dx build -p siar --platform android --release --target aarch64-linux-android

GENERATED_APP="$ROOT_DIR/target/dx/siar/release/android/app"
GENERATED_RES="$GENERATED_APP/app/src/main/res"
GENERATED_KOTLIN="$GENERATED_APP/app/src/main/kotlin"
SOURCE_RES="$ROOT_DIR/crates/android/kotlin/dev/irshad/siar/src/main/res"
SOURCE_KOTLIN="$ROOT_DIR/crates/android/kotlin"
MANIFEST_SNIPPET="$SOURCE_KOTLIN/AndroidManifest.snippet.xml"
VERSION_CODE_FILE="$ROOT_DIR/crates/android/version-code.txt"
if [[ ! -d "$GENERATED_RES" || ! -d "$SOURCE_RES" || ! -d "$GENERATED_KOTLIN" || ! -d "$SOURCE_KOTLIN" || ! -f "$MANIFEST_SNIPPET" ]]; then
  echo "Android resource directory missing after dx build" >&2
  exit 3
fi

VERSION_CODE="$(tr -d '[:space:]' < "$VERSION_CODE_FILE")"
if [[ ! "$VERSION_CODE" =~ ^[1-9][0-9]*$ ]]; then
  echo "Invalid Android version code: $VERSION_CODE" >&2
  exit 3
fi
sed -i -E "s/versionCode = [0-9]+/versionCode = $VERSION_CODE/" \
  "$GENERATED_APP/app/build.gradle.kts"

# Overlay the Activity bridge and foreground-service implementation after
# Dioxus generates its host project. MainActivity intentionally replaces the
# two-line generated subclass while all other Wry host files remain untouched.
cp -a "$SOURCE_KOTLIN/." "$GENERATED_KOTLIN/"

GENERATED_MANIFEST="$GENERATED_APP/app/src/main/AndroidManifest.xml"
# Dioxus' raw-manifest hook is inserted at the manifest root, where Android
# rejects service/receiver elements. Merge our component-only snippet into
# the generated <application> instead, then rebuild the generated project.
MANIFEST_TEMP="$GENERATED_MANIFEST.siar.tmp"
if ! awk -v snippet="$MANIFEST_SNIPPET" '
  /^[[:space:]]*<\/application>/ && !inserted {
    while ((getline line < snippet) > 0) print line
    close(snippet)
    inserted = 1
  }
  { print }
  END { if (!inserted) exit 42 }
' "$GENERATED_MANIFEST" > "$MANIFEST_TEMP"; then
  rm -f "$MANIFEST_TEMP"
  echo "Could not merge Android component manifest" >&2
  exit 3
fi
mv "$MANIFEST_TEMP" "$GENERATED_MANIFEST"

for component in \
  dev.irshad.siar.CallForegroundService \
  dev.irshad.siar.RelayForegroundService \
  dev.irshad.siar.BootCompletedReceiver; do
  if ! grep -Fq "$component" "$GENERATED_MANIFEST"; then
    echo "Android manifest is missing required component: $component" >&2
    exit 3
  fi
done

# Dioxus already derives density-correct legacy WebP launchers from the
# square 1024px bundle icon. Overlay only our adaptive layers; copying the
# source PNG legacy set as well would create duplicate Android resources.
install -Dm0644 "$SOURCE_RES/drawable-nodpi/siar_launcher_foreground.png" \
  "$GENERATED_RES/drawable-nodpi/siar_launcher_foreground.png"
install -Dm0644 "$SOURCE_RES/values/ic_launcher_background.xml" \
  "$GENERATED_RES/values/ic_launcher_background.xml"
install -Dm0644 "$SOURCE_RES/mipmap-anydpi-v26/ic_launcher.xml" \
  "$GENERATED_RES/mipmap-anydpi-v26/ic_launcher.xml"
install -Dm0644 "$SOURCE_RES/mipmap-anydpi-v26/ic_launcher_round.xml" \
  "$GENERATED_RES/mipmap-anydpi-v26/ic_launcher_round.xml"

cd "$GENERATED_APP"
./gradlew assembleRelease

APK_PATH="$GENERATED_APP/app/build/outputs/apk/release/app-release.apk"
UNSIGNED_APK_PATH="$GENERATED_APP/app/build/outputs/apk/release/app-release-unsigned.apk"
OUTPUT_APK="$ROOT_DIR/siar-android-release.apk"
if [[ -f "$APK_PATH" ]]; then
  install -m0644 "$APK_PATH" "$OUTPUT_APK"
elif [[ -f "$UNSIGNED_APK_PATH" ]]; then
  SDK_ROOT="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-/opt/android-sdk}}"
  APKSIGNER="$SDK_ROOT/build-tools/34.0.0/apksigner"
  if [[ ! -x "$APKSIGNER" || ! -f "$ROOT_DIR/keystore.jks" ]]; then
    echo "Release signing tools or keystore are missing" >&2
    exit 4
  fi
  "$APKSIGNER" sign \
    --ks "$ROOT_DIR/keystore.jks" \
    --ks-key-alias siar-key \
    --ks-pass pass:password \
    --key-pass pass:password \
    --out "$OUTPUT_APK" \
    "$UNSIGNED_APK_PATH"
else
  echo "Release APK was not produced" >&2
  exit 4
fi

SDK_ROOT="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-/opt/android-sdk}}"
APKSIGNER="$SDK_ROOT/build-tools/34.0.0/apksigner"
AAPT2="$SDK_ROOT/build-tools/34.0.0/aapt2"
EXPECTED_CERT_SHA256="faab39baebe74b9c3dc6fe366948f6feee8eb860549ad72515d8b5fce9efd2d5"
ACTUAL_CERT_SHA256="$($APKSIGNER verify --print-certs "$OUTPUT_APK" | sed -n 's/^Signer #1 certificate SHA-256 digest: //p')"
if [[ "$ACTUAL_CERT_SHA256" != "$EXPECTED_CERT_SHA256" ]]; then
  echo "Refusing APK signed by unexpected certificate: $ACTUAL_CERT_SHA256" >&2
  exit 5
fi
ACTUAL_VERSION_CODE="$($AAPT2 dump badging "$OUTPUT_APK" | sed -n "s/.*versionCode='\([0-9]*\)'.*/\1/p" | head -1)"
if [[ "$ACTUAL_VERSION_CODE" != "$VERSION_CODE" ]]; then
  echo "APK versionCode $ACTUAL_VERSION_CODE does not match expected $VERSION_CODE" >&2
  exit 5
fi
APK_MANIFEST_XMLTREE="$($AAPT2 dump xmltree "$OUTPUT_APK" --file AndroidManifest.xml)"
for component in \
  dev.irshad.siar.CallForegroundService \
  dev.irshad.siar.RelayForegroundService \
  dev.irshad.siar.BootCompletedReceiver; do
  if ! grep -Fq "$component" <<< "$APK_MANIFEST_XMLTREE"; then
    echo "Release APK is missing required component: $component" >&2
    exit 5
  fi
done

cat <<'EOF'
Android target build complete. APK is available at siar-android-release.apk
EOF
