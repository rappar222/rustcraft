use super::block::BlockType;
use super::chunk::{Chunk, ChunkPos, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};
use noise::{Fbm, NoiseFn, Perlin};
use rayon::prelude::*;
use rustc_hash::FxHashMap;

pub struct World {
    pub chunks: FxHashMap<ChunkPos, Chunk>,
    noise: Fbm<Perlin>,
    cave_noise: Perlin,
}

impl World {
    pub fn new(seed: u32) -> Self {
        let mut noise = Fbm::<Perlin>::new(seed);
        noise.octaves = 4;
        noise.frequency = 0.008;
        noise.persistence = 0.5;

        let cave_noise = Perlin::new(seed.wrapping_add(1000));

        Self {
            chunks: FxHashMap::default(),
            noise,
            cave_noise,
        }
    }

    #[inline(always)]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> BlockType {
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return BlockType::Air;
        }

        let chunk_pos = ChunkPos::from_block_coords(x, z);
        if let Some(chunk) = self.chunks.get(&chunk_pos) {
            let lx = x.rem_euclid(CHUNK_WIDTH as i32) as usize;
            let lz = z.rem_euclid(CHUNK_DEPTH as i32) as usize;
            chunk.get_block(lx, y as usize, lz)
        } else {
            BlockType::Air
        }
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockType) -> bool {
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return false;
        }

        let chunk_pos = ChunkPos::from_block_coords(x, z);
        let lx = x.rem_euclid(CHUNK_WIDTH as i32) as usize;
        let lz = z.rem_euclid(CHUNK_DEPTH as i32) as usize;

        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.set_block(lx, y as usize, lz, block);

            // Mark adjacent chunks dirty if block modification is on the border
            if lx == 0 {
                if let Some(c) = self.chunks.get_mut(&ChunkPos::new(chunk_pos.x - 1, chunk_pos.z)) {
                    c.is_dirty = true;
                }
            } else if lx == CHUNK_WIDTH - 1 {
                if let Some(c) = self.chunks.get_mut(&ChunkPos::new(chunk_pos.x + 1, chunk_pos.z)) {
                    c.is_dirty = true;
                }
            }

            if lz == 0 {
                if let Some(c) = self.chunks.get_mut(&ChunkPos::new(chunk_pos.x, chunk_pos.z - 1)) {
                    c.is_dirty = true;
                }
            } else if lz == CHUNK_DEPTH - 1 {
                if let Some(c) = self.chunks.get_mut(&ChunkPos::new(chunk_pos.x, chunk_pos.z + 1)) {
                    c.is_dirty = true;
                }
            }

            true
        } else {
            false
        }
    }

    pub fn generate_chunks_around(&mut self, center_chunk: ChunkPos, radius: i32) {
        let mut missing_positions = Vec::new();
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                if dx * dx + dz * dz <= radius * radius {
                    let pos = ChunkPos::new(center_chunk.x + dx, center_chunk.z + dz);
                    if !self.chunks.contains_key(&pos) {
                        missing_positions.push(pos);
                    }
                }
            }
        }

        if missing_positions.is_empty() {
            return;
        }

        let noise = &self.noise;
        let cave_noise = &self.cave_noise;

        // Generate terrain chunks in parallel using Rayon!
        let generated: Vec<Chunk> = missing_positions
            .into_par_iter()
            .map(|pos| Self::generate_chunk(pos, noise, cave_noise))
            .collect();

        for chunk in generated {
            self.chunks.insert(chunk.pos, chunk);
        }
    }

    fn generate_chunk(pos: ChunkPos, noise: &Fbm<Perlin>, cave_noise: &Perlin) -> Chunk {
        let mut chunk = Chunk::new(pos);
        let origin_x = pos.x * CHUNK_WIDTH as i32;
        let origin_z = pos.z * CHUNK_DEPTH as i32;

        let base_height = 40.0;
        let height_variation = 24.0;

        let mut trees_to_plant = Vec::new();

        for lz in 0..CHUNK_DEPTH {
            for lx in 0..CHUNK_WIDTH {
                let gx = origin_x + lx as i32;
                let gz = origin_z + lz as i32;

                // 2D terrain height
                let n_val = noise.get([gx as f64, gz as f64]);
                let height = (base_height + n_val * height_variation).clamp(4.0, (CHUNK_HEIGHT - 10) as f64) as usize;

                for y in 0..CHUNK_HEIGHT {
                    if y == 0 {
                        chunk.set_block(lx, y, lz, BlockType::Bedrock);
                    } else if y < height.saturating_sub(4) {
                        // 3D cave check
                        let cave_val = cave_noise.get([gx as f64 * 0.04, y as f64 * 0.05, gz as f64 * 0.04]);
                        if cave_val < 0.65 {
                            chunk.set_block(lx, y, lz, BlockType::Stone);
                        }
                    } else if y < height {
                        chunk.set_block(lx, y, lz, BlockType::Dirt);
                    } else if y == height {
                        if height < 34 {
                            chunk.set_block(lx, y, lz, BlockType::Sand);
                        } else {
                            chunk.set_block(lx, y, lz, BlockType::Grass);

                            // Pseudo-random tree placement
                            let tree_hash = ((gx.wrapping_mul(374761393)) ^ (gz.wrapping_mul(668265263))) as u32;
                            if tree_hash % 60 == 0 && lx >= 2 && lx <= CHUNK_WIDTH - 3 && lz >= 2 && lz <= CHUNK_DEPTH - 3 {
                                trees_to_plant.push((lx, height + 1, lz));
                            }
                        }
                    }
                }
            }
        }

        // Plant oak trees
        for (tx, ty, tz) in trees_to_plant {
            let trunk_height = 5;
            if ty + trunk_height + 2 < CHUNK_HEIGHT {
                // Trunk
                for dy in 0..trunk_height {
                    chunk.set_block(tx, ty + dy, tz, BlockType::Wood);
                }
                // Leaves canopy
                let leaf_start = ty + trunk_height - 2;
                for ly in leaf_start..=(ty + trunk_height + 1) {
                    let r: i32 = if ly >= ty + trunk_height { 1 } else { 2 };
                    for dx in -r..=r {
                        for dz in -r..=r {
                            if dx == 0 && dz == 0 && ly < ty + trunk_height {
                                continue;
                            }
                            if dx.abs() == r && dz.abs() == r && (ly == ty + trunk_height + 1 || (dx + dz) % 2 == 0) {
                                continue;
                            }
                            let leaf_x = (tx as i32 + dx) as usize;
                            let leaf_z = (tz as i32 + dz) as usize;
                            if chunk.get_block(leaf_x, ly, leaf_z) == BlockType::Air {
                                chunk.set_block(leaf_x, ly, leaf_z, BlockType::Leaves);
                            }
                        }
                    }
                }
            }
        }

        chunk.is_dirty = true;
        chunk
    }
}
