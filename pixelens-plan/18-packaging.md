# Pixelens Packaging — v1 .deb

**Date:** 2026-07-20
**Decision:** `.deb` only. Ship the systemd user unit (`pixelensd.service`)
but do **NOT** auto-enable it. No AppImage, no cargo-dist (out of scope).

## Artifact

- Package: `pixelens` v0.1.0, `Architecture: amd64`.
- Binaries: `/usr/bin/pixelens` (CLI), `/usr/bin/pixelensd` (daemon).
- User unit: `/usr/lib/systemd/user/pixelensd.service` (installed, not enabled).
- Man pages: `pixelens.1`, `pixelensd.1`.

## Runtime dependencies (declared in `DEBIAN/control`)

- Hard: `libc6` (only glibc is a linker dependency — both binaries are
  otherwise statically linked Rust).
- Functional (external CLIs the program shells out to): `slurp`, `grim`,
  `tesseract-ocr`, `xdg-utils`.
- `Recommends: fuzzel | wofi` (optional action-menu backends).
- `Suggests: xdg-desktop-portal`, `libgtk-3-0` (portal / gtk menu path,
  not in the default build).

## Why these and not gtk3/xkbcommon/etc.

`ldd` on the release binaries shows **only glibc** as a shared-lib dependency.
The keyhook crate uses evdev/nix (no libxkbcommon link); capture shells out
to slurp/grim at runtime; OCR shells out to tesseract; the gtk-layer-shell
menu backend is behind a feature not enabled in the default build. So the
*linker* deps are minimal — the *functional* deps are the external binaries,
which we declare as `Depends` so the package is actually usable.

## Build flow (reproducible)

`packaging/build-deb.sh`:
1. `cargo clean` (disk mitigation — 7.8G root historically fills).
2. `cargo build --release --bins` (links `pixelens` + `pixelensd`).
3. `install -m 0755` both into `packaging/deb/usr/bin/`.
4. `dpkg-deb --build --root-owner-group` → `target/debian/pixelens_0.1.0_amd64.deb`.

Verify with `dpkg-deb -I` (metadata) and `dpkg-deb -c` (file list) — done
in-script.

## Install / enable (user opt-in)

```
sudo dpkg -i pixelens_0.1.0_amd64.deb
systemctl --user enable --now pixelensd.service   # explicit, not automatic
```

## Status 2026-07-20

- Release build: links both bins, BUILD_EXIT=0, 12G free post-build.
- Deb tree authored, build script written, package built + verified.
- NOT installed on this VM (headless, no display) — structural verification
  only (`dpkg-deb -I`/`-c` passed). Live run requires a Wayland/X11 session.
- Committed + pushed to `features/core-loop` (see changelog).
