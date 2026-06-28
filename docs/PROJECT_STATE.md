# PROJECT_STATE.md — Pixelens

## Product Purpose

Pixelens lets the user select content visible on screen and immediately copy, search, translate, ask AI about it, or perform another contextual action.

## Current Architecture

- `pixelens`: user-facing CLI and daemon controller
- `pixelensd`: background daemon
- `pixelens-core`: capture, OCR, actions, configuration, IPC, menu, upload, search, and shared logic

## Current Working Functionality

- Wayland region selection through `slurp`
- Screenshot capture through `grim`
- OCR through Tesseract
- CLI-to-daemon IPC over Unix domain sockets
- Clean capture cancellation
- `pixelens grab` showing built-in action bar after OCR
- `pixelens copy` copying OCR text via `wl-copy`
- `pixelens search` returning search URL and opening browser
- `pixelens ai` sending text and image to configured AI provider (if model supports vision)
- `pixelens translate` translating OCR text
- `pixelens image` saving image locally and opening Google Lens upload page
- Custom upload provider support for reverse image search
- Built-in action bar with keyboard shortcuts (default)
- External menu backends: fuzzel, wofi, stdin (optional fallback)
- Daemon start and status behavior verified
- Vision model detection for AI image input
- Quota exhaustion error reporting (insufficient_quota)
- CI passing on GitHub Actions

## Current Command Semantics

All commands are selection-first. Typed positional text is not the normal input model.

| Command | Behavior |
|---------|----------|
| `pixelens grab` | Select region, OCR, show action bar |
| `pixelens copy` | Select region, OCR, copy to clipboard |
| `pixelens search` | Select region, OCR, return search URL |
| `pixelens ai` | Select region, OCR, send text+image to AI (if model supports vision) |
| `pixelens translate` | Select region, OCR, translate (optional `--to`) |
| `pixelens image` | Select region, save image, open Google Lens (upload if configured) |
| `pixelens daemon start` | Start pixelensd if not running |
| `pixelens daemon status` | Check daemon via IPC |
| `pixelens daemon stop` | Graceful shutdown via IPC |
| `pixelens config` | Show or set configuration |
| `pixelens version` | Show version |

## Built-in Action Bar

`pixelens grab` shows a built-in action bar with keyboard shortcuts:
- `[C] Copy` - Copy text to clipboard
- `[S] Search` - Search the web
- `[A] Ask AI` - Send to AI
- `[T] Translate` - Translate text
- `[Esc] Cancel` - Exit without action

The action bar uses `crossterm` for terminal-based interaction, requiring no external dependencies.

## Compositor Keybindings

Recommended keybindings for Wayland compositors:

**Hyprland** (`~/.config/hypr/hyprland.conf`):
```
bind = SUPER SHIFT, S, exec, pixelens grab
bind = SUPER SHIFT, C, exec, pixelens copy
bind = SUPER SHIFT, F, exec, pixelens search
bind = SUPER SHIFT, A, exec, pixelens ai
bind = SUPER SHIFT, T, exec, pixelens translate
bind = SUPER SHIFT, I, exec, pixelens image
```

**Niri** (`~/.config/niri/config.kdl`):
```kdl
binds {
    Mod+Shift+S { spawn "pixelens" "grab"; }
    Mod+Shift+C { spawn "pixelens" "copy"; }
    Mod+Shift+F { spawn "pixelens" "search"; }
    Mod+Shift+A { spawn "pixelens" "ai"; }
    Mod+Shift+T { spawn "pixelens" "translate"; }
    Mod+Shift+I { spawn "pixelens" "image"; }
}
```

**Sway** (`~/.config/sway/config`):
```
bindsym $mod+Shift+s exec pixelens grab
bindsym $mod+Shift+c exec pixelens copy
bindsym $mod+Shift+f exec pixelens search
bindsym $mod+Shift+a exec pixelens ai
bindsym $mod+Shift+t exec pixelens translate
bindsym $mod+Shift+i exec pixelens image
```

## Known Limitations

- Automatic reverse image search requires custom upload provider configuration
- OCR may preserve layout artifacts from Tesseract

## Next Milestone

Polish Ask AI behavior: improve prompt construction, add default prompts, and verify image input works with vision models.

See `docs/ROADMAP.md` for full roadmap.

## Non-Goals for Next Milestone

- No new frontend framework
- No broad OCR tuning
- No macOS implementation
- No unrelated refactoring
