# ROADMAP.md — Pixelens

## Project Goal

Build a Linux-native visual search and OCR utility that lets users select screen content and perform actions on it.

## MVP (Complete)

- [x] Screen capture via grim/slurp
- [x] OCR via Tesseract
- [x] Action menu after capture (copy, search, AI, translate)
- [x] Clipboard integration via wl-copy
- [x] Search via xdg-open
- [x] AI integration with vision model support
- [x] Daemon with IPC
- [x] CLI with selection-first commands
- [x] Menu backends (fuzzel, wofi, stdin)
- [x] Compositor keybinding examples (Hyprland, Niri, Sway)
- [x] Reverse image search: save PNG locally, open Google Lens upload page

## Post-MVP (In Progress)

- [x] Custom upload provider support for reverse image search
- [ ] Ask AI with screenshot input (partial: vision model detection done, needs verification)
- [ ] Translate with screenshot context (currently text-only)
- [ ] Improve action menu UX (currently functional)

## Later / Exploratory

- [ ] Optional provider integrations (SerpAPI, Google Vision, Imgur)
- [ ] X11 backend (grim/slurp alternative)
- [ ] System tray integration
- [ ] Daemon-managed global hotkeys (compositor keybindings recommended instead)
- [ ] Packaging (AUR, DEB, etc.)
- [ ] macOS support

## Non-Goals

- Web frontend / React / TypeScript
- Generic text-processing CLI
- Platform-specific UI frameworks
- Complex configuration UI

## Current Next Milestone

Polish Ask AI behavior: improve prompt construction, add default prompts, and verify image input works with vision models.
