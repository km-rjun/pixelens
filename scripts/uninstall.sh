#!/bin/bash
set -euo pipefail

INSTALL_DIR="$HOME/.local/bin"

echo "Uninstalling Pixelens..."
echo ""

# Remove binaries
if [ -f "$INSTALL_DIR/pixelens" ]; then
    rm "$INSTALL_DIR/pixelens"
    echo "Removed: $INSTALL_DIR/pixelens"
else
    echo "Not found: $INSTALL_DIR/pixelens"
fi

if [ -f "$INSTALL_DIR/pixelensd" ]; then
    rm "$INSTALL_DIR/pixelensd"
    echo "Removed: $INSTALL_DIR/pixelensd"
else
    echo "Not found: $INSTALL_DIR/pixelensd"
fi

echo ""
echo "Uninstall complete."
echo ""
echo "Note: Config and cache directories were not removed."
echo "  Config: ~/.config/pixelens"
echo "  Cache:  ~/.cache/pixelens"
echo ""
echo "To remove them manually:"
echo "  rm -rf ~/.config/pixelens ~/.cache/pixelens"
