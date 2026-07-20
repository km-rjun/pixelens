//! Search, reverse-image, and image-upload logic for Pixelens.
//!
//! Ported from the divergent `origin/main` `crates/pixelens-core/src/{actions,search,upload}`
//! under Strategy C: the *logic* is carried over, but the `ActionHandler` trait harness from
//! `origin/main` is intentionally dropped — this crate exposes plain functions/structs that the
//! `pixelens-daemon` wires into its `Command` dispatch.

pub mod error;
pub mod google_lens;
pub mod reverse_image;
pub mod search;
pub mod upload;

// Re-export the daemon-facing API so callers can use
// `pixelens_search::{build_search_url, ReverseImageSearcher}` directly.
pub use reverse_image::ReverseImageSearcher;
pub use search::build_search_url;
