//! 2D rendering library on top of wgpu and lyon.

#![allow(unused)]

mod atlas;
mod canvas;
pub mod font;
mod glyph;
mod rect;
mod renderer;
mod sprite;
mod text;

pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

pub use canvas::{Canvas, SpriteData, SpriteDescriptor};
pub use sprite::SpriteId;
pub use text::{
    layout::{Align, Baseline, Paragraph, TextLayout},
    markup, Text, TextSection, TextStyle,
};
