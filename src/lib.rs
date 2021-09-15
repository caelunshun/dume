//! A 2D renderer for `wgpu`. Supports
//! drawing sprites, paths with solid colors
//! or gradients, and text.

#![allow(unused)]

/*
mod canvas;
mod glyph;
mod path;
mod renderer;
mod sprite;*/
mod atlas;
mod context;
pub mod font;
mod rect;
mod text;
mod texture;
mod thread_pool;

pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const SAMPLE_COUNT: u32 = 4;

pub use context::Context;
pub use font::FontId;
pub use rect::Rect;
pub use text::{Text, TextSection, TextStyle};
pub use texture::{MissingTexture, TextureId, TextureSet, TextureSetBuilder};
pub use thread_pool::{BasicThreadPool, ThreadPool};

