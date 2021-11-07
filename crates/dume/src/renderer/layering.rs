use glam::{ivec2, IVec2, Vec2};
use smallvec::SmallVec;

use crate::Rect;

use super::BatchId;

/// Responsible for splitting up batches to maintain correct draw order.
///
/// All items in one batch are drawn together. A problem arises when two
/// batches contain overlapping elements. For example, consider the following
/// sequence of draw commands given to the canvas:
/// 1. Draw image A at (0, 0). The renderer assigns the image to batch 1.
/// 2. Draw some text at (0, 0). The renderer assigns the text to batch 2.
/// 3. Draw image B at (0, 0).
/// Without splitting up batches, image B would be added to batch 1, which
/// causes the text to be layered over image B - when it should be underneath
/// according to draw order.
///
/// The layering engine prevents the above situation by splitting the window
/// into tiles. Whenever a draw command could affect a tile, that tile is marked
/// as affected by that command's batch. If the tile is already marked by a different batch
/// _as well as the current batch_, then we have to start a new batch.
#[derive(Default)]
pub struct LayeringEngine {
    tiles: Box<[Tile]>,
    width: u32,
    height: u32,
}

impl LayeringEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_window_size(&mut self, size: Vec2) {
        let size = pixel_to_tile_coords(size);

        if size.x != self.width as i32 || size.y != self.height as i32 {
            self.tiles =
                vec![Tile::default(); size.x as usize * size.y as usize].into_boxed_slice();
        }

        self.width = size.x as u32;
        self.height = size.y as u32;
    }

    pub(super) fn layer(
        &mut self,
        affected_region: Rect,
        potential_batch: BatchId,
    ) -> LayeringResult {
        let min = pixel_to_tile_coords(affected_region.pos).max(IVec2::ZERO);
        let max = pixel_to_tile_coords(affected_region.pos + affected_region.size)
            .min(ivec2(self.width as i32, self.height as i32));

        for x in min.x..=max.x {
            for y in min.y..=max.y {
                if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
                    let tile = &mut self.tiles[x as usize + (y as usize * self.width as usize)];

                    if let Some(pos) = tile
                        .affected_by_batches
                        .iter()
                        .position(|b| *b == potential_batch)
                    {
                        // If this batch isn't at the front, we have to make a new batch.
                        if pos != tile.affected_by_batches.len() - 1 {
                            return LayeringResult::CreateNewBatch;
                        }
                    } else if tile
                        .affected_by_batches
                        .iter()
                        .any(|&b| b > potential_batch)
                    {
                        return LayeringResult::CreateNewBatch;
                    } else {
                        tile.affected_by_batches.push(potential_batch);
                    }
                }
            }
        }

        LayeringResult::UseCurrentBatch
    }

    pub fn reset(&mut self) {
        for tile in &mut *self.tiles {
            tile.affected_by_batches.clear();
        }
    }
}

/// The size of each square tile in logical pixels.
const TILE_SIZE: f32 = 10.;

fn pixel_to_tile_coords(px: Vec2) -> IVec2 {
    ivec2(
        (px.x / TILE_SIZE).floor() as i32,
        (px.y / TILE_SIZE).floor() as i32,
    )
}

#[derive(Debug, Copy, Clone)]
pub enum LayeringResult {
    UseCurrentBatch,
    CreateNewBatch,
}

#[derive(Default, Debug, Clone)]
struct Tile {
    affected_by_batches: SmallVec<[BatchId; 1]>,
}
