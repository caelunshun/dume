//! A 2D renderer for `wgpu`. Supports
//! drawing sprites, paths with solid colors
//! or gradients, and text.

#![allow(unused)]

mod atlas;
mod canvas;
pub mod font;
mod glyph;
mod path;
mod rect;
mod renderer;
mod sprite;
mod text;

pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const SAMPLE_COUNT: u32 = 4;

pub use canvas::{Canvas, SpriteData, SpriteDescriptor};
pub use rect::Rect;
pub use sprite::SpriteId;
pub use text::{
    layout::{Align, Baseline, Paragraph, TextLayout},
    markup, Text, TextSection, TextStyle,
};
