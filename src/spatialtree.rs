use std::collections::HashMap;

use bevy::{prelude::*};

pub const SPATIAL_TREE_SEARCH_RADIUS: f32 = 100.0;

#[derive(Resource)]
pub struct SpatialTree {
    tile_size: f32,
    grid: HashMap<(isize, isize), Vec<Entity>>
}

impl SpatialTree {
    pub fn new(tile_size: f32) -> Self {
        Self {
            grid: HashMap::new(),
            tile_size
        }
    }

    fn loc_to_tile(&self, loc: f32) -> isize {
        (loc / self.tile_size) as isize
    }

    fn get_affected_tiles(&self, x: f32, y: f32, size: f32) -> (isize, isize, isize, isize) {
        (
            self.loc_to_tile(x),
            self.loc_to_tile(y - size),
            self.loc_to_tile(x + size),
            self.loc_to_tile(y),
        )
    }

    pub fn insert(&mut self, x: f32, y: f32, size: f32, e: Entity) {
        let (gx_min, gy_min, gx_max, gy_max) = self.get_affected_tiles(x, y, size);
        for gx in gx_min..=gx_max {
            for gy in gy_min..=gy_max {
                self.grid
                    .entry((gx, gy))
                    .and_modify(|entries| entries.push(e))
                    .or_insert(vec![e]);
            }
        }
    }

    pub fn remove(&mut self, x: f32, y: f32, size: f32, e: Entity) {
        let (gx_min, gy_min, gx_max, gy_max) = self.get_affected_tiles(x, y, size);
        for gx in gx_min..=gx_max {
            for gy in gy_min..=gy_max {
                let Some(entities) = self.grid.get_mut(&(gx, gy)) else {
                    warn!("Attempting to extract entity at pos [{},{}], but no entry is found in grid - this could be bug", gx, gy);
                    continue;
                };

                entities.retain(|&entity| entity != e);
            }
        }
    }

    pub fn reposition(&mut self, from: Vec2, to: Vec2, size: f32, e: Entity) {
        self.remove(from.x, from.y, size, e);
        self.insert(to.x, to.y, size, e);
    }

    pub fn get_at(&self, x: f32, y: f32, size: f32) -> Vec<Entity> {
        let (gx_min, gy_min, gx_max, gy_max) = self.get_affected_tiles(x, y, size);
        let mut result = vec![];
        for gx in gx_min..=gx_max {
            for gy in gy_min..=gy_max {
                let Some(entries) = self.grid.get(&(gx, gy)) else {
                    continue;
                };
                result.extend(entries)
            }
        }
        result
    }

    pub fn cleanup_y(&mut self, y: f32) {
        let gy = self.loc_to_tile(y);
        self.grid.retain(|k, _v| {
            gy <= k.0
        });
    }
}

