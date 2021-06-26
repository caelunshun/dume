#![allow(clippy::clippy::derive_hash_xor_eq)]

use std::hash::{Hash, Hasher};

use glam::Vec2;
use lru::LruCache;
use lyon::{
    geom::Point,
    lyon_tessellation::{
        Count, FillGeometryBuilder, FillOptions, FillTessellator, FillVertex, GeometryBuilder,
        GeometryBuilderError, StrokeGeometryBuilder, StrokeOptions, StrokeTessellator,
        StrokeVertex, VertexId,
    },
    math::{Angle, Vector},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Path {
    pub segments: Vec<PathSegment>,
}

impl Path {
    pub fn to_lyon(&self, close: bool) -> lyon::path::Path {
        let mut builder = lyon::path::Path::builder().with_svg();
        for (i, segment) in self.segments.iter().enumerate() {
            match segment {
                PathSegment::MoveTo(p) => {
                    builder.move_to(point(*p));
                }
                PathSegment::LineTo(p) => {
                    builder.line_to(point(*p));
                }
                PathSegment::QuadTo(c, p) => {
                    builder.quadratic_bezier_to(point(*c), point(*p));
                }
                PathSegment::CubicTo(c1, c2, p) => {
                    builder.cubic_bezier_to(point(*c1), point(*c2), point(*p));
                }
                PathSegment::Arc(center, radius, start_angle, end_angle) => builder.arc(
                    point(*center),
                    Vector::splat(*radius),
                    Angle::radians(start_angle - end_angle),
                    Angle::radians(*start_angle),
                ),
            };
        }
        if close {
            builder.close();
        }
        builder.build()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    MoveTo(Vec2),
    LineTo(Vec2),
    QuadTo(Vec2, Vec2),
    CubicTo(Vec2, Vec2, Vec2),
    Arc(Vec2, f32, f32, f32),
}

impl Eq for PathSegment {}

impl Hash for PathSegment {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            PathSegment::MoveTo(p) => hash_vec(state, *p),
            PathSegment::LineTo(p) => hash_vec(state, *p),
            PathSegment::QuadTo(c, p) => {
                hash_vec(state, *c);
                hash_vec(state, *p);
            }
            PathSegment::CubicTo(c1, c2, p) => {
                hash_vec(state, *c1);
                hash_vec(state, *c2);
                hash_vec(state, *p);
            }
            PathSegment::Arc(center, radius, start_angle, end_angle) => {
                hash_vec(state, *center);
                radius.to_bits().hash(state);
                start_angle.to_bits().hash(state);
                end_angle.to_bits().hash(state);
            }
        }
    }
}

fn hash_vec(state: &mut impl Hasher, v: Vec2) {
    v.x.to_bits().hash(state);
    v.y.to_bits().hash(state);
}

fn point(v: Vec2) -> Point<f32> {
    Point::new(v.x, v.y)
}

const CACHE_CAPACITY: usize = 16384;

#[derive(Debug, Clone, Default)]
pub struct TesselatedPath {
    pub vertices: Vec<Vec2>,
    pub indices: Vec<u32>,
}

impl GeometryBuilder for TesselatedPath {
    fn begin_geometry(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    fn end_geometry(&mut self) -> Count {
        Count {
            vertices: self.vertices.len() as u32,
            indices: self.indices.len() as u32,
        }
    }

    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        self.indices.push(a.0);
        self.indices.push(b.0);
        self.indices.push(c.0);
    }

    fn abort_geometry(&mut self) {}
}

impl FillGeometryBuilder for TesselatedPath {
    fn add_fill_vertex(&mut self, vertex: FillVertex) -> Result<VertexId, GeometryBuilderError> {
        self.vertices
            .push(Vec2::new(vertex.position().x, vertex.position().y));
        Ok(VertexId::from_usize(self.vertices.len() - 1))
    }
}

impl StrokeGeometryBuilder for TesselatedPath {
    fn add_stroke_vertex(
        &mut self,
        vertex: StrokeVertex,
    ) -> Result<VertexId, GeometryBuilderError> {
        self.vertices
            .push(Vec2::new(vertex.position().x, vertex.position().y));
        Ok(VertexId::from_usize(self.vertices.len() - 1))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TesselateKind {
    Fill,
    Stroke { width: u32 },
}

/// A cache of tesselated paths.
#[allow(clippy::clippy::new_without_default)]
pub struct PathCache {
    paths: LruCache<(Path, TesselateKind), TesselatedPath, ahash::RandomState>,
    fill_tesselator: FillTessellator,
    stroke_tesselator: StrokeTessellator,
}

impl PathCache {
    pub fn new() -> Self {
        Self {
            paths: LruCache::with_hasher(CACHE_CAPACITY, ahash::RandomState::new()),
            fill_tesselator: FillTessellator::new(),
            stroke_tesselator: StrokeTessellator::new(),
        }
    }

    pub fn with_tesselated_path(
        &mut self,
        path: &(Path, TesselateKind),
        callback: impl FnOnce(&TesselatedPath),
    ) {
        if let Some(p) = self.paths.get(path) {
            callback(p);
        } else {
            let lyon_path = path.0.to_lyon(path.1 == TesselateKind::Fill);
            let mut tesselated = TesselatedPath::default();
            match path.1 {
                TesselateKind::Fill => {
                    self.fill_tesselator.tessellate_path(
                        &lyon_path,
                        &FillOptions::default(),
                        &mut tesselated,
                    );
                }
                TesselateKind::Stroke { width } => {
                    let width = width as f32 / 100.0;
                    self.stroke_tesselator.tessellate_path(
                        &lyon_path,
                        &StrokeOptions::default().with_line_width(width),
                        &mut tesselated,
                    );
                }
            }

            callback(&tesselated);

            self.paths.put(path.clone(), tesselated);
        }
    }
}
