//! Shared types, errors, and traits for the Pixelens workspace.
//!
//! This crate is the lowest layer of the dependency graph. It defines the
//! error enum used by every other crate, the data types that flow between
//! components (capture results, OCR results, geometry), and the core traits
//! (`CaptureProvider`, `OcrEngine`) that v1 implementations satisfy.
//!
//! M1 note: traits are declared here so other crates can depend on them
//! before any concrete backend lands (M2+). Concrete implementations are
//! introduced in later milestones.

pub mod error;
pub mod geometry;
pub mod traits;

pub use error::{CaptureError, CaptureResult, PixelensError, PixelensResult};
pub use geometry::{Point, Rect, Size};
pub use traits::{
    CaptureImage, CaptureProvider, CaptureRequest, OcrEngine, OcrError, RawCapture,
};
