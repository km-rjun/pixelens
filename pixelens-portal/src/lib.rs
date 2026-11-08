//! `pixelens-portal`: xdg-desktop-portal native screen capture backend.
//!
//! UM5 introduces a portal-native capture path that talks directly to
//! `org.freedesktop.portal.ScreenCast` instead of shelling out to
//! `slurp` + `grim`. When the `portal` feature is enabled, [`PortalBackend`]
//! tries the portal path first and transparently falls back to the
//! existing `slurp`/`grim` workflow when the portal is unavailable.
//!
//! The portal I/O is abstracted behind the [`PortalSession`] trait so the
//! core capture logic is unit-testable without a live DBus/pipewire
//! session (see [`MockPortalSession`]).
//!
//! When the `portal` feature is disabled (the default), this crate still
//! compiles and links cleanly: only the [`portal`] module, which contains
//! the real [`PortalBackend`] implementation, is gated behind the feature.

pub mod portal;

pub use portal::{PortalBackend, PortalSession};
