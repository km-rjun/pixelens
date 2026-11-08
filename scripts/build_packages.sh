#!/usr/bin/env bash
# Build all packaging artifacts for a release.
# Usage: ./scripts/build_packages.sh <version>

set -euo pipefail

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
    echo "Usage: $0 <version>"
    exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PACKAGING_DIR="$ROOT_DIR/packaging"
DEB_PATH="$ROOT_DIR/target/debian/pixelens_${VERSION}_amd64.deb"
WIN_ZIP_PATH="$ROOT_DIR/target/debian/pixelens-${VERSION}-windows-x64.zip"

echo "Building packages for pixelens v${VERSION}"

# Verify artifacts exist
if [[ ! -f "$DEB_PATH" ]]; then
    echo "ERROR: .deb not found at $DEB_PATH"
    exit 1
fi
if [[ ! -f "$WIN_ZIP_PATH" ]]; then
    echo "ERROR: Windows zip not found at $WIN_ZIP_PATH"
    exit 1
fi

# Compute checksums
DEB_SHA256=$(sha256sum "$DEB_PATH" | cut -d' ' -f1)
WIN_SHA256=$(sha256sum "$WIN_ZIP_PATH" | cut -d' ' -f1)

echo "DEB SHA256: $DEB_SHA256"
echo "ZIP SHA256: $WIN_SHA256"

# Update AUR PKGBUILD
AUR_PKGBUILD="$PACKAGING_DIR/aur/PKGBUILD"
sed -i "s/pkgver=.*/pkgver=${VERSION}/" "$AUR_PKGBUILD"
sed -i "s|sha256sums=('SKIP')|sha256sums=('$DEB_SHA256')|" "$AUR_PKGBUILD"
echo "Updated $AUR_PKGBUILD"

# Update Chocolatey nuspec
CHOCO_NUSPEC="$PACKAGING_DIR/chocolatey/pixelens.nuspec"
sed -i "s|<version>.*</version>|<version>${VERSION}</version>|" "$CHOCO_NUSPEC"
sed -i "s|v[0-9.]\+/pixelens-[0-9.]\+-windows-x64\.zip|v${VERSION}/pixelens-${VERSION}-windows-x64.zip|" "$CHOCO_NUSPEC"
echo "Updated $CHOCO_NUSPEC"

# Update Chocolatey install script
CHOCO_INSTALL="$PACKAGING_DIR/chocolatey/tools/chocolateyinstall.ps1"
sed -i "s|v[0-9.]\+/pixelens-[0-9.]\+-windows-x64\.zip|v${VERSION}/pixelens-${VERSION}-windows-x64.zip|" "$CHOCO_INSTALL"
sed -i "s|\$checksum = ''|\$checksum = '${WIN_SHA256}'|" "$CHOCO_INSTALL"
echo "Updated $CHOCO_INSTALL"

# Update Scoop manifest
SCOOP_MANIFEST="$PACKAGING_DIR/scoop/pixelens.json"
# Use jq for proper JSON manipulation
jq --arg version "$VERSION" \
   --arg url "https://github.com/km-rjun/pixelens/releases/download/v${VERSION}/pixelens-${VERSION}-windows-x64.zip" \
   --arg hash "$WIN_SHA256" \
   '.version = $version | .architecture."64bit".url = $url | .architecture."64bit".hash = $hash' \
   "$SCOOP_MANIFEST" > "${SCOOP_MANIFEST}.tmp" && mv "${SCOOP_MANIFEST}.tmp" "$SCOOP_MANIFEST"
echo "Updated $SCOOP_MANIFEST"

echo ""
echo "All packaging files updated for v${VERSION}"
echo ""
echo "Next steps:"
echo "  1. AUR: Push updated PKGBUILD to aur.archlinux.org (pixelens-bin)"
echo "  2. Chocolatey: Run 'choco pack' in packaging/chocolatey/ and push with 'choco push'"
echo "  3. Scoop: Add manifest to a bucket repo (e.g. scoop-bucket) or submit PR to main bucket"
echo "  4. Homebrew: Create formula in homebrew-core or a tap"