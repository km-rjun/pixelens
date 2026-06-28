#!/bin/bash
set -euo pipefail

INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/pixelens"
CACHE_DIR="$HOME/.cache/pixelens"
SERVICE_DIR="$HOME/.config/systemd/user"
SERVICE_FILE="pixelensd.service"

echo "Installing Pixelens..."
echo ""

# Create directories if they don't exist
mkdir -p "$INSTALL_DIR" "$CONFIG_DIR" "$CACHE_DIR"

# Check if pixelensd is running and stop it
STOPPED_DAEMON=false
if pgrep -x pixelensd > /dev/null 2>&1; then
    echo "Stopping running pixelensd..."
    if [ -f "$SERVICE_DIR/$SERVICE_FILE" ]; then
        systemctl --user stop pixelensd 2>/dev/null || killall pixelensd 2>/dev/null || true
    else
        killall pixelensd 2>/dev/null || true
    fi
    STOPPED_DAEMON=true
    sleep 1
fi

# Build release binaries with layer-shell support
echo "Building release binaries with layer-shell support..."
echo "  cargo build --release --workspace --features layer-shell"
cargo build --release --workspace --features layer-shell

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
    echo "Built with layer-shell support: yes"
    echo ""
    echo "Config directory: $CONFIG_DIR"
    echo "Cache directory:  $CACHE_DIR"
else
    echo ""
    echo "Installation failed. Binaries not found in $INSTALL_DIR"
    exit 1
fi

# Restart daemon if it was running before
if [ "$STOPPED_DAEMON" = true ]; then
    echo ""
    echo "Restarting pixelensd..."
    if [ -f "$SERVICE_DIR/$SERVICE_FILE" ]; then
        systemctl --user start pixelensd 2>/dev/null || true
    else
        "$INSTALL_DIR/pixelensd" &
    fi
    echo "pixelensd restarted."
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
echo "  2. Start the daemon: pixelens daemon start"
echo "  3. Use pixelens: pixelens grab"
echo ""
echo "Optional: Install as systemd user service:"
echo "  cp packaging/systemd/pixelensd.service ~/.config/systemd/user/"
echo "  systemctl --user daemon-reload"
echo "  systemctl --user enable --now pixelensd"
