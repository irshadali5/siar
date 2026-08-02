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
SOURCE_RES="$ROOT_DIR/crates/android/kotlin/dev/irshad/siar/src/main/res"
if [[ ! -d "$GENERATED_RES" || ! -d "$SOURCE_RES" ]]; then
  echo "Android resource directory missing after dx build" >&2
  exit 3
fi

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

cat <<'EOF'
Android target build complete. APK is available at siar-android-release.apk
EOF
