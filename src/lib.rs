//! A 2D renderer for `wgpu`. Supports
//! drawing sprites, paths with solid colors
//! or gradients, and text.

mod atlas;
mod canvas;
mod context;
pub mod font;
mod path;
mod rect;
mod renderer;
mod text;
mod texture;
mod thread_pool;

pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const SAMPLE_COUNT: u32 = 4;

pub use canvas::Canvas;
pub use context::Context;
pub use font::FontId;
pub use rect::Rect;
pub use text::{
    layout::{TextBlob, TextOptions},
    Text, TextSection, TextStyle,
};
pub use texture::{MissingTexture, TextureId, TextureSet, TextureSetBuilder, TextureSetId};
pub use thread_pool::{BasicThreadPool, ThreadPool};

/// Utility function to convert RGBA to BGRA data in place.
pub fn convert_rgba_to_bgra(data: &mut [u8]) {
    for chunk in data.chunks_exact_mut(4) {
        let r = chunk[0];
        let b = chunk[2];
        chunk[0] = b;
        chunk[2] = r;
    }
}
