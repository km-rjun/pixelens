#!/bin/bash
set -euo pipefail

INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/pixelens"
CACHE_DIR="$HOME/.cache/pixelens"

echo "Installing Pixelens..."
echo ""

# Create directories if they don't exist
mkdir -p "$INSTALL_DIR" "$CONFIG_DIR" "$CACHE_DIR"

# Build release binaries
echo "Building release binaries..."
cargo build --release

# Copy binaries
echo "Installing binaries to $INSTALL_DIR..."
cp target/release/pixelens "$INSTALL_DIR/"
cp target/release/pixelensd "$INSTALL_DIR/"

# Verify installation
if [ -f "$INSTALL_DIR/pixelens" ] && [ -f "$INSTALL_DIR/pixelensd" ]; then
    echo ""
    echo "Installation complete!"
    echo ""
    echo "Installed:"
    echo "  $INSTALL_DIR/pixelens"
    echo "  $INSTALL_DIR/pixelensd"
    echo ""
    echo "Config directory: $CONFIG_DIR"
    echo "Cache directory:  $CACHE_DIR"
else
    echo ""
    echo "Installation failed. Binaries not found in $INSTALL_DIR"
    exit 1
fi

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "WARNING: $INSTALL_DIR is not in your PATH."
    echo "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    echo ""
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
fi

echo ""
echo "Next steps:"
echo "  1. Ensure $INSTALL_DIR is in your PATH"
echo "  2. Start the daemon: pixelensd &"
echo "  3. Use pixelens: pixelens grab"
echo ""
echo "Optional: Install as systemd user service:"
echo "  cp packaging/systemd/pixelensd.service ~/.config/systemd/user/"
echo "  systemctl --user daemon-reload"
echo "  systemctl --user enable --now pixelensd"
