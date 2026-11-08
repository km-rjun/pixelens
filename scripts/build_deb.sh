#!/bin/bash
# Build .deb package for pixelens
# Usage: ./scripts/build_deb.sh [VERSION]
# If VERSION not provided, uses "dev"

set -euo pipefail

VERSION="${1:-dev}"
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEB_DIR="${WORKSPACE_ROOT}/target/debian"
STAGE_DIR="${DEB_DIR}/pixelens_${VERSION}_amd64"

echo "Building .deb for pixelens ${VERSION}"

# Clean and create staging directory
rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}/DEBIAN"
mkdir -p "${STAGE_DIR}/usr/bin"
mkdir -p "${STAGE_DIR}/usr/lib/systemd/user"
mkdir -p "${STAGE_DIR}/usr/share/man/man1"

# Build release binaries
cd "${WORKSPACE_ROOT}"
cargo build --workspace --release --locked

# Copy binaries
cp target/release/pixelens "${STAGE_DIR}/usr/bin/"
cp target/release/pixelensd "${STAGE_DIR}/usr/bin/"

# Copy systemd user unit files
cat > "${STAGE_DIR}/usr/lib/systemd/user/pixelens.service" <<'EOF'
[Unit]
Description=Pixelens OCR daemon
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/bin/pixelensd
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=graphical-session.target
EOF

cat > "${STAGE_DIR}/usr/lib/systemd/user/pixelens-keyhook.service" <<'EOF'
[Unit]
Description=Pixelens global hotkey listener
After=pixelens.service
PartOf=pixelens.service

[Service]
Type=simple
ExecStart=/usr/bin/pixelens-keyhook
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
EOF

# Copy man pages (generate from --help if they don't exist)
if [ -f "${WORKSPACE_ROOT}/docs/pixelens.1" ]; then
    cp "${WORKSPACE_ROOT}/docs/pixelens.1" "${STAGE_DIR}/usr/share/man/man1/"
    cp "${WORKSPACE_ROOT}/docs/pixelensd.1" "${STAGE_DIR}/usr/share/man/man1/"
else
    # Generate basic man pages
    cat > "${STAGE_DIR}/usr/share/man/man1/pixelens.1" <<'MANEOF'
.TH PIXELENS 1 "2024" "Pixelens 0.1" "User Commands"
.SH NAME
pixelens \- CLI client for pixelens OCR daemon
.SH SYNOPSIS
.B pixelens
[\fIcommand\fR] [\fIargs\fR...]
.SH DESCRIPTION
Pixelens is a visual text extraction utility. Press a hotkey, select a screen
region, and the text is copied to clipboard via OCR.
.SH COMMANDS
.TP
.B grab
Select an area and copy text to clipboard
.TP
.B copy
Alias for grab
.TP
.B status
Show daemon status
.TP
.B stop
Stop daemon
.TP
.B install
Install systemd user service (Linux) or scheduled task (Windows)
.TP
.B hotkey
Manage global hotkey (enable|disable|status)
.TP
.B autostart
Manage XDG autostart .desktop (enable|disable|status)
.TP
.B config
Manage configuration (list|get|set)
.TP
.B version
Show version
.TP
.B help
Show help
.SH AUTHOR
km-rjun <km-rjun@users.noreply.github.com>
MANEOF
    cat > "${STAGE_DIR}/usr/share/man/man1/pixelensd.1" <<'MANEOF'
.TH PIXELENSD 1 "2024" "Pixelens 0.1" "User Commands"
.SH NAME
pixelensd \- Pixelens OCR daemon
.SH SYNOPSIS
.B pixelensd
.SH DESCRIPTION
Background daemon that handles screen capture, OCR, AI, and action-menu
dispatch. Communicates with the CLI client over a Unix socket.
.SH AUTHOR
km-rjun <km-rjun@users.noreply.github.com>
MANEOF
fi

# Create control file
cat > "${STAGE_DIR}/DEBIAN/control" <<EOF
Package: pixelens
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: amd64
Depends: libc6, slurp, grim, tesseract-ocr, xdg-utils
Recommends: fuzzel | wofi
Suggests: xdg-desktop-portal, libgtk-3-0
Maintainer: km-rjun <km-rjun@users.noreply.github.com>
Homepage: https://github.com/km-rjun/pixelens
Description: Linux-native visual text extraction utility
 Press a hotkey, select a screen region, and Pixelens extracts the text
 (OCR) and copies it to the clipboard. A background daemon (pixelensd)
 handles capture, OCR, AI, and action-menu dispatch. Ships CLI (pixelens)
 and daemon (pixelensd). No display-server integration is bundled; the
 default capture path uses slurp/grim.
EOF

# Create postinst
cat > "${STAGE_DIR}/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
systemctl --user daemon-reload >/dev/null 2>&1 || true
# Do NOT auto-enable; user must run: systemctl --user enable --now pixelens.service pixelens-keyhook.service
exit 0
EOF
chmod 755 "${STAGE_DIR}/DEBIAN/postinst"

# Create postrm
cat > "${STAGE_DIR}/DEBIAN/postrm" <<'EOF'
#!/bin/sh
set -e
systemctl --user daemon-reload >/dev/null 2>&1 || true
exit 0
EOF
chmod 755 "${STAGE_DIR}/DEBIAN/postrm"

# Build .deb
cd "${DEB_DIR}"
dpkg-deb --build "pixelens_${VERSION}_amd64"

echo "Built: ${DEB_DIR}/pixelens_${VERSION}_amd64.deb"
ls -la "${DEB_DIR}/pixelens_${VERSION}_amd64.deb"