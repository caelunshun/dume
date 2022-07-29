//! Utility for using `zylo` with `winit`.
//!
//! Supports the `tiny-skia` and `wgpu` backends.

use zylo::ErasedBackend;
use zylo_tiny_skia::TinySkiaBackend;

mod app;
mod presenter;

pub use app::{run, Render};
pub use presenter::Presenter;

pub extern crate winit;

/// The rendering backend in use.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BackendType {
    #[cfg(feature = "backend-tiny-skia")]
    TinySkia,
    Other,
}

/// Selects the optimal available `Backend`.
///
/// # Panics
/// Panics if no backend is available.
pub fn select_optimal_backend() -> Box<dyn ErasedBackend> {
    if cfg!(feature = "backend-tiny-skia") {
        Box::new(TinySkiaBackend::new())
    } else {
        panic!("no working rendering backend is available")
    }
}
