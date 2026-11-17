#!/usr/bin/env bash
# Build release binaries and stage the Linux file tree, then make .deb and .rpm with nfpm:
#   dist/release/scriptcraft-<version>-linux-<arch>.{deb,rpm}
# Release builds embed craft-fonts and craft-i18n: set CRAFT_FONTS_DIR and CRAFT_I18N_DIR.
set -euo pipefail
. "$(dirname "$0")/../env.sh"
export CRAFT_FONTS_REQUIRED=1 CRAFT_I18N_REQUIRED=1
ARCH="$(uname -m)"
case "$ARCH" in x86_64) NFPM_ARCH=amd64 ;; aarch64) NFPM_ARCH=arm64 ;; *) NFPM_ARCH="$ARCH" ;; esac
export NFPM_ARCH
cargo build --release -p scriptcraft -p scriptcraft-cli --manifest-path "$ROOT/Cargo.toml"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
TARGET="${CARGO_TARGET_DIR:-$ROOT/target}/release"
install -Dm755 "$TARGET/scriptcraft" "$STAGE/usr/bin/scriptcraft"
install -Dm755 "$TARGET/scriptcraft-cli" "$STAGE/usr/bin/scriptcraft-cli"
install -Dm644 "$ROOT/packaging/linux/ai.storyteller.scriptcraft.desktop" "$STAGE/usr/share/applications/ai.storyteller.scriptcraft.desktop"
cp -r "$ROOT/assets/app-icon/hicolor" "$STAGE/usr/share/icons/" 2>/dev/null || { mkdir -p "$STAGE/usr/share/icons"; cp -r "$ROOT/assets/app-icon/hicolor" "$STAGE/usr/share/icons/"; }
CONFIG="$(mktemp)"
sed "s|\${STAGE}|$STAGE|g" "$ROOT/packaging/linux/nfpm.yaml" > "$CONFIG"
for format in deb rpm; do
  nfpm package -f "$CONFIG" -p "$format" -t "$DIST/scriptcraft-$VERSION-linux-$ARCH.$format"
done
rm -f "$CONFIG"
