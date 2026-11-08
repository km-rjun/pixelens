# 04 — Crate: pixelens-core

**Role**: lowest layer of the dependency graph. Defines shared types, errors,
and the core traits every other crate depends on. No sibling-crate deps.

**Path**: `pixelens-core/src/{lib,error,geometry,traits}.rs`

**Exposes** (`lib.rs`):
- `error` → `CaptureError`, `CaptureResult`, `PixelensError`, `PixelensResult`
- `geometry` → `Point`, `Rect`, `Size`
- `traits` → `CaptureImage`, `CaptureProvider`, `CaptureRequest`, `OcrEngine`,
  `OcrError`, `RawCapture`

**State**: ✅ Stable foundation. The traits (`CaptureProvider`, `OcrEngine`)
are declared here intentionally ahead of concrete backends (M2+). The
`CaptureProvider::capture` takes a `CaptureRequest` and returns `RawCapture`.

**Notes**:
- `PixelensError::NoDisplayServer` is what `detector` returns on unknown env;
  the daemon maps this to a fatal startup error.
- This crate compiles independently; it is the safest place to add shared types.

**Work remaining**: none required for M1–M2. Future milestones add concrete
trait impls in sibling crates, not here (unless a new shared type is needed).
