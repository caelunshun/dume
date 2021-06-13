//! 2D rendering library on top of wgpu and lyon.

#![allow(unused)]

mod atlas;
mod canvas;
pub mod font;
mod renderer;
mod sprite;
mod rect;
mod text;
mod glyph;

pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

pub use canvas::{Canvas, SpriteData, SpriteDescriptor};
pub use sprite::SpriteId;
pub use text::{Text, TextSection, TextStyle};
