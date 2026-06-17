# Integration tests

Cross-crate integration tests live here. Each file should target a
specific milestone once that milestone's deliverables are landed.

| Test file | Covers |
|---|---|
| (none yet) | — |

## Conventions

- Tests must not require a real display server. The display detection
  module is exercised with a unit test; capture / OCR tests will use
  in-memory fixtures until the real backends are wired in M3–M5.
- Tests must not modify the host's `~/.config/pixelens`. The config
  loader must accept an explicit path override (M8).
- Tests must finish in <5 s so CI stays snappy.
