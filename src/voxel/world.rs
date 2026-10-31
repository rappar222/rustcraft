use super::block::BlockType;
use super::chunk::{Chunk, ChunkPos, CHUNK_DEPTH, CHUNK_HEIGHT, CHUNK_WIDTH};
use glam::Vec3;
use noise::{Fbm, NoiseFn, Perlin};
use rayon::prelude::*;
use rustc_hash::FxHashMap;

pub const SEA_LEVEL: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Plains,
    Forest,
    Desert,
    SnowyTaiga,
    Mountains,
}

impl Biome {
    pub fn name(self) -> &'static str {
        match self {
            Biome::Plains => "Plains",
            Biome::Forest => "Forest",
            Biome::Desert => "Desert",
            Biome::SnowyTaiga => "Snowy Taiga",
            Biome::Mountains => "Mountains",
        }
    }
}

pub struct World {
    pub seed: u32,
    pub chunks: FxHashMap<ChunkPos, Chunk>,
    pub modified_blocks: FxHashMap<(i32, i32, i32), BlockType>,
    noise: Fbm<Perlin>,
    biome_noise: Perlin,
    cave_noise: Perlin,
}

impl World {
    pub fn new(seed: u32) -> Self {
        let mut noise = Fbm::<Perlin>::new(seed);
        noise.octaves = 4;
        noise.frequency = 0.008;
        noise.persistence = 0.5;

        let biome_noise = Perlin::new(seed.wrapping_add(500));
        let cave_noise = Perlin::new(seed.wrapping_add(1000));

        Self {
            seed,
            chunks: FxHashMap::default(),
            modified_blocks: FxHashMap::default(),
            noise,
            biome_noise,
            cave_noise,
        }
    }

    pub fn biome_at(&self, gx: i32, gz: i32) -> Biome {
        Self::sample_biome(&self.biome_noise, &self.noise, gx, gz)
    }

    fn sample_biome(biome_noise: &Perlin, terrain_noise: &Fbm<Perlin>, gx: i32, gz: i32) -> Biome {
        let temp = biome_noise.get([gx as f64 * 0.0035, gz as f64 * 0.0035]);
        let elev = terrain_noise.get([gx as f64, gz as f64]);
        if elev > 0.52 {
            Biome::Mountains
        } else if temp < -0.26 {
            Biome::SnowyTaiga
        } else if temp > 0.30 {
            Biome::Desert
        } else if temp > 0.05 {
            Biome::Forest
        } else {
            Biome::Plains
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

    pub fn get_surface_height(&self, x: i32, z: i32) -> i32 {
        for y in (0..CHUNK_HEIGHT as i32).rev() {
            let b = self.get_block(x, y, z);
            if b.is_solid() && b != BlockType::Leaves && b != BlockType::Cactus {
                return y;
            }
        }
        42
    }

    /// Find a safe dry spawn coordinate near (0, 0)
    pub fn find_safe_spawn(&self) -> Vec3 {
        for radius in 0..16 {
            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    let gx = dx * 2;
                    let gz = dz * 2;
                    let gy = self.get_surface_height(gx, gz);
                    let surface_block = self.get_block(gx, gy, gz);
                    let above1 = self.get_block(gx, gy + 1, gz);
                    let above2 = self.get_block(gx, gy + 2, gz);
                    if gy >= SEA_LEVEL as i32
                        && surface_block != BlockType::Cactus
                        && !above1.is_fluid()
                        && !above1.is_solid()
                        && !above2.is_solid()
                    {
                        return Vec3::new(gx as f32 + 0.5, gy as f32 + 1.001, gz as f32 + 0.5);
                    }
                }
            }
        }
        let gy = self.get_surface_height(0, 0);
        Vec3::new(0.5, gy as f32 + 1.001, 0.5)
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockType) -> bool {
        if !self.set_block_raw(x, y, z, block) {
            return false;
        }
        self.modified_blocks.insert((x, y, z), block);
        self.update_local_physics(x, y, z);
        true
    }

    pub fn set_block_raw(&mut self, x: i32, y: i32, z: i32, block: BlockType) -> bool {
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return false;
        }

        let chunk_pos = ChunkPos::from_block_coords(x, z);
        let lx = x.rem_euclid(CHUNK_WIDTH as i32) as usize;
        let lz = z.rem_euclid(CHUNK_DEPTH as i32) as usize;

        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.set_block(lx, y as usize, lz, block);

            // Mark adjacent chunks dirty if block modification is on or near the border (for AO/light)
            if lx <= 1 {
                if let Some(c) = self.chunks.get_mut(&ChunkPos::new(chunk_pos.x - 1, chunk_pos.z)) {
                    c.is_dirty = true;
                }
            }
            if lx >= CHUNK_WIDTH - 2 {
                if let Some(c) = self.chunks.get_mut(&ChunkPos::new(chunk_pos.x + 1, chunk_pos.z)) {
                    c.is_dirty = true;
                }
            }
            if lz <= 1 {
                if let Some(c) = self.chunks.get_mut(&ChunkPos::new(chunk_pos.x, chunk_pos.z - 1)) {
                    c.is_dirty = true;
                }
            }
            if lz >= CHUNK_DEPTH - 2 {
                if let Some(c) = self.chunks.get_mut(&ChunkPos::new(chunk_pos.x, chunk_pos.z + 1)) {
                    c.is_dirty = true;
                }
            }

            true
        } else {
            false
        }
    }

    /// Resolve immediate block physics: falling Sand/Gravel, Water+Lava -> Obsidian, and downward Water flow
    fn update_local_physics(&mut self, cx: i32, cy: i32, cz: i32) {
        // 1. Settle gravity columns (Sand, Gravel) at and above (cx, cy, cz)
        for check_y in cy..(cy + 16).min(CHUNK_HEIGHT as i32 - 1) {
            let b = self.get_block(cx, check_y, cz);
            if b.is_gravity_block() {
                let mut target_y = check_y;
                while target_y > 1 {
                    let below = self.get_block(cx, target_y - 1, cz);
                    if below == BlockType::Air || below == BlockType::Water {
                        target_y -= 1;
                    } else {
                        break;
                    }
                }
                if target_y != check_y {
                    self.set_block_raw(cx, check_y, cz, BlockType::Air);
                    self.modified_blocks.insert((cx, check_y, cz), BlockType::Air);
                    self.set_block_raw(cx, target_y, cz, b);
                    self.modified_blocks.insert((cx, target_y, cz), b);
                }
            }
        }

        // 2. Water / Lava reactions & fluid flow around (cx, cy, cz)
        let neighbors = [
            (cx, cy, cz),
            (cx + 1, cy, cz),
            (cx - 1, cy, cz),
            (cx, cy + 1, cz),
            (cx, cy - 1, cz),
            (cx, cy, cz + 1),
            (cx, cy, cz - 1),
        ];

        for &(nx, ny, nz) in &neighbors {
            let b = self.get_block(nx, ny, nz);
            if b == BlockType::Lava {
                // Check if any neighbor of Lava is Water -> turn into Obsidian!
                for (dx, dy, dz) in [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
                    if self.get_block(nx + dx, ny + dy, nz + dz) == BlockType::Water {
                        self.set_block_raw(nx, ny, nz, BlockType::Obsidian);
                        self.modified_blocks.insert((nx, ny, nz), BlockType::Obsidian);
                        break;
                    }
                }
            } else if b == BlockType::Water {
                // Flow downward into Air below
                if ny > 1 && self.get_block(nx, ny - 1, nz) == BlockType::Air {
                    self.set_block_raw(nx, ny - 1, nz, BlockType::Water);
                    self.modified_blocks.insert((nx, ny - 1, nz), BlockType::Water);
                }
            }
        }
    }

    /// Trigger a 3D spherical explosion (for TNT or Creeper), destroying blocks and returning drops
    pub fn explode(&mut self, cx: i32, cy: i32, cz: i32, radius: f32) -> Vec<(BlockType, Vec3)> {
        let mut drops = Vec::new();
        let r = radius.ceil() as i32;
        let r_sq = radius * radius;

        for dy in -r..=r {
            for dz in -r..=r {
                for dx in -r..=r {
                    let dist_sq = (dx * dx + dy * dy + dz * dz) as f32;
                    if dist_sq <= r_sq {
                        let bx = cx + dx;
                        let by = cy + dy;
                        let bz = cz + dz;
                        let block = self.get_block(bx, by, bz);
                        if block != BlockType::Air
                            && block != BlockType::Bedrock
                            && block != BlockType::Obsidian
                            && !block.is_fluid()
                        {
                            self.set_block_raw(bx, by, bz, BlockType::Air);
                            self.modified_blocks.insert((bx, by, bz), BlockType::Air);
                            if let Some(drop) = block.drop_item() {
                                // Drop 50% of blasted blocks to prevent lag while feeling rewarding
                                if ((bx ^ by ^ bz) & 1) == 0 {
                                    drops.push((
                                        drop,
                                        Vec3::new(bx as f32 + 0.5, by as f32 + 0.5, bz as f32 + 0.5),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        drops
    }

    /// Check if any block within `radius` of `pos` matches `target` (e.g. CraftingTable or Furnace)
    pub fn is_block_nearby(&self, pos: Vec3, radius: i32, target: BlockType) -> bool {
        let px = pos.x.floor() as i32;
        let py = pos.y.floor() as i32;
        let pz = pos.z.floor() as i32;

        for dy in -radius..=radius {
            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    if self.get_block(px + dx, py + dy, pz + dz) == target {
                        return true;
                    }
                }
            }
        }
        false
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
        let biome_noise = &self.biome_noise;
        let cave_noise = &self.cave_noise;

        // Generate terrain chunks in parallel using Rayon!
        let generated: Vec<Chunk> = missing_positions
            .into_par_iter()
            .map(|pos| Self::generate_chunk(pos, noise, biome_noise, cave_noise))
            .collect();

        for mut chunk in generated {
            let p = chunk.pos;
            let origin_x = p.x * CHUNK_WIDTH as i32;
            let origin_z = p.z * CHUNK_DEPTH as i32;

            // Re-apply any saved block modifications belonging to this chunk
            for ly in 0..CHUNK_HEIGHT {
                for lz in 0..CHUNK_DEPTH {
                    for lx in 0..CHUNK_WIDTH {
                        let gx = origin_x + lx as i32;
                        let gy = ly as i32;
                        let gz = origin_z + lz as i32;
                        if let Some(&mod_block) = self.modified_blocks.get(&(gx, gy, gz)) {
                            chunk.set_block(lx, ly, lz, mod_block);
                        }
                    }
                }
            }

            self.chunks.insert(p, chunk);

            // Mark adjacent existing chunks dirty so boundary faces update seamlessly
            for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let neighbor_pos = ChunkPos::new(p.x + dx, p.z + dz);
                if let Some(neighbor) = self.chunks.get_mut(&neighbor_pos) {
                    neighbor.is_dirty = true;
                }
            }
        }
    }

    fn generate_chunk(
        pos: ChunkPos,
        noise: &Fbm<Perlin>,
        biome_noise: &Perlin,
        cave_noise: &Perlin,
    ) -> Chunk {
        let mut chunk = Chunk::new(pos);
        let origin_x = pos.x * CHUNK_WIDTH as i32;
        let origin_z = pos.z * CHUNK_DEPTH as i32;

        let base_height = 38.0;
        let height_variation = 26.0;

        let mut flora_to_plant = Vec::new();

        for lz in 0..CHUNK_DEPTH {
            for lx in 0..CHUNK_WIDTH {
                let gx = origin_x + lx as i32;
                let gz = origin_z + lz as i32;

                let biome = Self::sample_biome(biome_noise, noise, gx, gz);
                let n_val = noise.get([gx as f64, gz as f64]);
                let height = (base_height + n_val * height_variation)
                    .clamp(6.0, (CHUNK_HEIGHT - 12) as f64) as usize;

                for y in 0..CHUNK_HEIGHT {
                    if y == 0 {
                        chunk.set_block(lx, y, lz, BlockType::Bedrock);
                    } else if y < height.saturating_sub(3) {
                        // 3D cave check
                        let cave_val = cave_noise.get([gx as f64 * 0.04, y as f64 * 0.05, gz as f64 * 0.04]);
                        if cave_val >= 0.64 {
                            if y <= 7 {
                                chunk.set_block(lx, y, lz, BlockType::Lava);
                            }
                        } else {
                            // Underground Ore Vein Distribution
                            let ore_hash = ((gx.wrapping_mul(73856093))
                                ^ ((y as i32).wrapping_mul(19349663))
                                ^ (gz.wrapping_mul(83492791))) as u32;
                            let block = if y <= 15 && ore_hash % 190 == 0 {
                                BlockType::DiamondOre
                            } else if y <= 30 && ore_hash % 110 == 1 {
                                BlockType::GoldOre
                            } else if y <= 58 && ore_hash % 48 == 2 {
                                BlockType::IronOre
                            } else if y <= 78 && ore_hash % 28 == 3 {
                                BlockType::CoalOre
                            } else if y <= 45 && ore_hash % 65 == 4 {
                                BlockType::Gravel
                            } else {
                                BlockType::Stone
                            };
                            chunk.set_block(lx, y, lz, block);
                        }
                    } else if y < height {
                        let sub_block = match biome {
                            Biome::Desert => BlockType::Sand,
                            Biome::Mountains if height > 56 => BlockType::Stone,
                            _ if height <= SEA_LEVEL + 1 => BlockType::Sand,
                            _ => BlockType::Dirt,
                        };
                        chunk.set_block(lx, y, lz, sub_block);
                    } else if y == height {
                        if height < SEA_LEVEL {
                            // Underwater floor
                            let floor_block = if (gx + gz) % 5 == 0 {
                                BlockType::Gravel
                            } else {
                                BlockType::Sand
                            };
                            chunk.set_block(lx, y, lz, floor_block);
                        } else if height <= SEA_LEVEL + 1 && biome != Biome::SnowyTaiga {
                            // Sandy beach shoreline
                            chunk.set_block(lx, y, lz, BlockType::Sand);
                        } else {
                            let top_block = match biome {
                                Biome::Desert => BlockType::Sand,
                                Biome::SnowyTaiga => BlockType::Snow,
                                Biome::Mountains if height > 58 => BlockType::Snow,
                                Biome::Mountains if height > 52 => BlockType::Stone,
                                _ => BlockType::Grass,
                            };
                            chunk.set_block(lx, y, lz, top_block);

                            // Tree / Cactus placement
                            let tree_hash = ((gx.wrapping_mul(374761393)) ^ (gz.wrapping_mul(668265263))) as u32;
                            if lx >= 2 && lx <= CHUNK_WIDTH - 3 && lz >= 2 && lz <= CHUNK_DEPTH - 3 {
                                match biome {
                                    Biome::Forest if tree_hash % 36 == 0 => {
                                        let log = if tree_hash % 72 == 0 {
                                            BlockType::BirchWood
                                        } else {
                                            BlockType::Wood
                                        };
                                        flora_to_plant.push((lx, height + 1, lz, log));
                                    }
                                    Biome::Plains if tree_hash % 85 == 0 => {
                                        flora_to_plant.push((lx, height + 1, lz, BlockType::Wood));
                                    }
                                    Biome::SnowyTaiga if tree_hash % 48 == 0 => {
                                        flora_to_plant.push((lx, height + 1, lz, BlockType::SpruceWood));
                                    }
                                    Biome::Desert if tree_hash % 95 == 0 => {
                                        flora_to_plant.push((lx, height + 1, lz, BlockType::Cactus));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } else if y <= SEA_LEVEL && height < SEA_LEVEL {
                        if y == SEA_LEVEL && biome == Biome::SnowyTaiga {
                            chunk.set_block(lx, y, lz, BlockType::Ice);
                        } else {
                            chunk.set_block(lx, y, lz, BlockType::Water);
                        }
                    }
                }
            }
        }

        // Plant trees and cacti
        for (tx, ty, tz, kind) in flora_to_plant {
            if kind == BlockType::Cactus {
                for dy in 0..3 {
                    if ty + dy < CHUNK_HEIGHT {
                        chunk.set_block(tx, ty + dy, tz, BlockType::Cactus);
                    }
                }
            } else {
                let trunk_height = 5;
                if ty + trunk_height + 2 < CHUNK_HEIGHT {
                    for dy in 0..trunk_height {
                        chunk.set_block(tx, ty + dy, tz, kind);
                    }
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
        }

        chunk.is_dirty = true;
        chunk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_biomes_gravity_and_explosion() {
        let mut world = World::new(42);
        world.generate_chunks_around(ChunkPos::new(0, 0), 1);

        // Verify safe spawn is above sea level and solid underneath
        let spawn = world.find_safe_spawn();
        assert!(spawn.y >= SEA_LEVEL as f32);

        // Test gravity settling for Sand
        world.set_block(2, 60, 2, BlockType::Air);
        world.set_block(2, 61, 2, BlockType::Air);
        world.set_block(2, 62, 2, BlockType::Sand);
        // Sand should settle down when unsupported
        assert_ne!(world.get_block(2, 62, 2), BlockType::Sand);

        // Test Water + Lava -> Obsidian reaction
        world.set_block(5, 50, 5, BlockType::Lava);
        world.set_block(6, 50, 5, BlockType::Water);
        assert_eq!(world.get_block(5, 50, 5), BlockType::Obsidian);

        // Test TNT / Creeper explosion destroys stone but not bedrock
        world.set_block(8, 50, 8, BlockType::Stone);
        let _drops = world.explode(8, 50, 8, 2.5);
        assert_eq!(world.get_block(8, 50, 8), BlockType::Air);
        assert_eq!(world.get_block(8, 0, 8), BlockType::Bedrock);
    }
}
