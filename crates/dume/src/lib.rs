//! A 2D renderer for `wgpu`. Supports
//! drawing sprites, paths with solid colors
//! or gradients, and text.

#![allow(clippy::derive_hash_xor_eq, clippy::too_many_arguments)]
#![allow(dead_code)]

mod atlas;
mod canvas;
mod context;
pub mod font;
mod glyph;
mod layer;
mod rect;
mod renderer;
mod scissor;
mod text;
mod texture;
pub mod yuv;

// NB: the data stored in these textures is sRGB, but we handle
// the sRGB conversions ourselves.
const INTERMEDIATE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Uint;
pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

pub use canvas::Canvas;
pub use context::Context;
pub use font::{FontId, Style, Weight};
pub use layer::Layer;
pub use rect::Rect;
pub use renderer::StrokeCap;
pub use scissor::Scissor;
use smartstring::LazyCompact;
pub use text::{
    layout::{Align, Baseline, TextBlob, TextOptions},
    Text, TextSection, TextStyle,
};
pub use texture::{MissingTexture, TextureId, TextureSet, TextureSetBuilder, TextureSetId};
pub use yuv::YuvTexture;

pub use palette::Srgba;
pub use dume_markup::markup;

pub type SmartString = smartstring::SmartString<LazyCompact>;

#[macro_export]
macro_rules! text {
    ($markup:literal $(,)? $($fmt_arg:expr),* $(,)?) => {
        $crate::Text::from_sections($crate::markup!($markup, $($fmt_arg),*))
    }
}

/// Utility function to convert RGBA to BGRA data in place.
pub fn convert_rgba_to_bgra(data: &mut [u8]) {
    for chunk in data.chunks_exact_mut(4) {
        let r = chunk[0];
        let b = chunk[2];
        chunk[0] = b;
        chunk[2] = r;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SpriteRotate {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}
