#!/usr/bin/env bash
# build-deb.sh — produce a reproducible pixelens .deb from a release build.
#
# Usage:  packaging/build-deb.sh
# Requires: cargo, dpkg-deb, a prior `cargo build --release`.
#
# The script copies the two release binaries into the deb tree, sets
# permissions, and runs `dpkg-deb --build`. It does NOT auto-enable the
# shipped systemd user unit (per packaging decision 2026-07-20).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEB_ROOT="$REPO_ROOT/packaging/deb"
STAGE_BIN="$DEB_ROOT/usr/bin"
OUT_DIR="$REPO_ROOT/target/debian"

VERSION="$(grep -m1 '^version' "$REPO_ROOT/Cargo.toml" | sed -E 's/version *= *"([^"]+)"/\1/')"
PKG="pixelens_${VERSION}_amd64.deb"

echo "==> staging binaries"
mkdir -p "$STAGE_BIN"
install -m 0755 "$REPO_ROOT/target/release/pixelens"  "$STAGE_BIN/pixelens"
install -m 0755 "$REPO_ROOT/target/release/pixelensd" "$STAGE_BIN/pixelensd"

echo "==> building $PKG"
mkdir -p "$OUT_DIR"
# fakeroot not required for a user-service unit; build as current user.
dpkg-deb --build --root-owner-group "$DEB_ROOT" "$OUT_DIR/$PKG"

echo "==> verifying"
dpkg-deb -I "$OUT_DIR/$PKG"
dpkg-deb -c "$OUT_DIR/$PKG"
echo "==> wrote $OUT_DIR/$PKG"
