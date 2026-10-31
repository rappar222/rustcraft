use crate::game::inventory::Inventory;
use crate::render::vertex::BlockVertex;
use crate::voxel::block::BlockType;
use crate::voxel::world::World;
use glam::Vec3;

#[derive(Debug, Clone)]
pub struct DroppedItem {
    pub position: Vec3,
    pub velocity: Vec3,
    pub block: BlockType,
    pub count: u32,
    pub rotation: f32,
    pub age: f32,
    pub pickup_delay: f32,
}

pub struct DroppedItemManager {
    pub items: Vec<DroppedItem>,
    spawn_counter: u32,
}

impl DroppedItemManager {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            spawn_counter: 0,
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn spawn(&mut self, block: BlockType, pos: Vec3) {
        self.spawn_stack(block, 1, pos);
    }

    pub fn spawn_stack(&mut self, block: BlockType, count: u32, pos: Vec3) {
        if block == BlockType::Air || count == 0 {
            return;
        }

        self.spawn_counter = self.spawn_counter.wrapping_add(1);
        let hash = (pos.x.to_bits().wrapping_mul(73856093))
            ^ (pos.y.to_bits().wrapping_mul(19349663))
            ^ (pos.z.to_bits().wrapping_mul(83492791))
            ^ (self.spawn_counter.wrapping_mul(2654435761));
        let angle = (hash % 360) as f32 * std::f32::consts::PI / 180.0;
        let speed = 1.15;

        self.items.push(DroppedItem {
            position: pos,
            velocity: Vec3::new(angle.cos() * speed, 2.8, angle.sin() * speed),
            block,
            count,
            rotation: angle,
            age: 0.0,
            pickup_delay: 0.25,
        });
    }

    pub fn update(
        &mut self,
        world: &World,
        player_feet: Vec3,
        inventory: &mut Inventory,
        dt: f32,
    ) {
        let player_center = player_feet + Vec3::new(0.0, 0.85, 0.0);

        // Merge nearby matching item stacks on the ground
        let mut a = 0;
        while a < self.items.len() {
            let mut b = a + 1;
            while b < self.items.len() {
                if self.items[a].block == self.items[b].block
                    && self.items[a].count + self.items[b].count <= 64
                    && self.items[a].age > 0.35
                    && self.items[b].age > 0.35
                    && (self.items[a].position - self.items[b].position).length_squared() < 0.45
                {
                    let add_cnt = self.items[b].count;
                    self.items[a].count += add_cnt;
                    self.items.swap_remove(b);
                } else {
                    b += 1;
                }
            }
            a += 1;
        }

        let mut i = 0;
        while i < self.items.len() {
            let item = &mut self.items[i];
            item.age += dt;
            item.rotation += 2.4 * dt;

            if item.pickup_delay > 0.0 {
                item.pickup_delay -= dt;
            }

            // Despawn if older than 5 minutes or fell into deep void
            if item.age > 300.0 || item.position.y < -30.0 {
                self.items.swap_remove(i);
                continue;
            }

            // Check if player is within pickup range
            let dx = item.position.x - player_feet.x;
            let dz = item.position.z - player_feet.z;
            let horiz_dist = (dx * dx + dz * dz).sqrt();
            let dy = item.position.y - player_feet.y;

            if item.pickup_delay <= 0.0 && horiz_dist < 1.85 && dy >= -0.9 && dy <= 2.3 {
                let added = inventory.add_item_partial(item.block, item.count);
                if added >= item.count {
                    self.items.swap_remove(i);
                    continue;
                } else if added > 0 {
                    item.count -= added;
                }
            }

            // Magnet attraction towards player center when nearby
            let to_center = player_center - item.position;
            let dist_to_center = to_center.length();

            if item.pickup_delay <= 0.0 && dist_to_center < 3.6 && dist_to_center > 0.001 {
                let pull_dir = to_center / dist_to_center;
                item.velocity += pull_dir * 34.0 * dt;
                let cur_spd = item.velocity.length();
                if cur_spd > 13.0 {
                    item.velocity *= 13.0 / cur_spd;
                }
            } else {
                // Regular gravity & air/ground friction
                item.velocity.y = (item.velocity.y - 22.0 * dt).max(-40.0);
                let drag = (-3.5 * dt).exp();
                item.velocity.x *= drag;
                item.velocity.z *= drag;
            }

            // Vertical movement and collision
            let next_y = item.position.y + item.velocity.y * dt;
            if item.velocity.y <= 0.0 {
                let test_y = next_y - 0.15;
                let bx = item.position.x.floor() as i32;
                let by = test_y.floor() as i32;
                let bz = item.position.z.floor() as i32;
                let block = world.get_block(bx, by, bz);
                if block.is_solid() {
                    let top_y = by as f32 + 1.0 + 0.15;
                    item.position.y = top_y;
                    item.velocity.y = 0.0;
                    // Extra ground friction when resting on surface
                    let ground_drag = (-8.0 * dt).exp();
                    item.velocity.x *= ground_drag;
                    item.velocity.z *= ground_drag;
                } else {
                    item.position.y = next_y;
                }
            } else {
                let test_y = next_y + 0.15;
                let bx = item.position.x.floor() as i32;
                let by = test_y.floor() as i32;
                let bz = item.position.z.floor() as i32;
                let block = world.get_block(bx, by, bz);
                if block.is_solid() {
                    item.velocity.y = 0.0;
                } else {
                    item.position.y = next_y;
                }
            }

            // Horizontal X movement & collision
            let next_x = item.position.x + item.velocity.x * dt;
            let test_x = if item.velocity.x >= 0.0 {
                next_x + 0.15
            } else {
                next_x - 0.15
            };
            let block_x = world.get_block(
                test_x.floor() as i32,
                item.position.y.floor() as i32,
                item.position.z.floor() as i32,
            );
            if block_x.is_solid() {
                item.velocity.x = 0.0;
            } else {
                item.position.x = next_x;
            }

            // Horizontal Z movement & collision
            let next_z = item.position.z + item.velocity.z * dt;
            let test_z = if item.velocity.z >= 0.0 {
                next_z + 0.15
            } else {
                next_z - 0.15
            };
            let block_z = world.get_block(
                item.position.x.floor() as i32,
                item.position.y.floor() as i32,
                test_z.floor() as i32,
            );
            if block_z.is_solid() {
                item.velocity.z = 0.0;
            } else {
                item.position.z = next_z;
            }

            // Check pickup again after movement
            let dx_post = item.position.x - player_feet.x;
            let dz_post = item.position.z - player_feet.z;
            let horiz_dist_post = (dx_post * dx_post + dz_post * dz_post).sqrt();
            let dy_post = item.position.y - player_feet.y;

            if item.pickup_delay <= 0.0 && horiz_dist_post < 1.85 && dy_post >= -0.9 && dy_post <= 2.3 {
                let added = inventory.add_item_partial(item.block, item.count);
                if added >= item.count {
                    self.items.swap_remove(i);
                    continue;
                } else if added > 0 {
                    item.count -= added;
                }
            }

            i += 1;
        }
    }

    pub fn generate_mesh_with_highlight(
        &self,
        target_block: Option<(i32, i32, i32)>,
        break_progress: f32,
    ) -> (Vec<BlockVertex>, Vec<u32>) {
        let mut vertices = Vec::with_capacity(self.items.len() * 48 + 288);
        let mut indices = Vec::with_capacity(self.items.len() * 72 + 432);

        let half_size = 0.125; // 0.25 total width

        for item in &self.items {
            let bobbing_y = (item.age * 3.5).sin() * 0.06;
            let center = item.position + Vec3::new(0.0, bobbing_y, 0.0);
            Self::append_rotated_cube(&mut vertices, &mut indices, center, half_size, item.rotation, item.block, 1.0);

            // If stack has multiple items, render a second slightly offset mini-cube (classic Minecraft style!)
            if item.count > 1 {
                let offset = Vec3::new(
                    item.rotation.cos() * 0.055,
                    0.035,
                    item.rotation.sin() * 0.055,
                );
                Self::append_rotated_cube(
                    &mut vertices,
                    &mut indices,
                    center + offset,
                    half_size,
                    item.rotation,
                    item.block,
                    0.92,
                );
            }
        }

        // Render 3D Block Selection Wireframe Box around targeted voxel
        if let Some((bx, by, bz)) = target_block {
            Self::append_selection_box(&mut vertices, &mut indices, bx, by, bz, break_progress);
        }

        (vertices, indices)
    }

    #[allow(dead_code)]
    pub fn generate_mesh(&self) -> (Vec<BlockVertex>, Vec<u32>) {
        self.generate_mesh_with_highlight(None, 0.0)
    }

    fn append_rotated_cube(
        vertices: &mut Vec<BlockVertex>,
        indices: &mut Vec<u32>,
        center: Vec3,
        s: f32,
        rot: f32,
        block: BlockType,
        ao: f32,
    ) {
        let cos_r = rot.cos();
        let sin_r = rot.sin();

        let rotate_pt = |dx: f32, dy: f32, dz: f32| -> [f32; 3] {
            let rx = dx * cos_r - dz * sin_r;
            let rz = dx * sin_r + dz * cos_r;
            [center.x + rx, center.y + dy, center.z + rz]
        };

        // Face 0: +X
        {
            let tl = block.texture_layer(0);
            let base = vertices.len() as u32;
            vertices.extend_from_slice(&[
                BlockVertex { position: rotate_pt(s, -s, s), normal_idx: 0, uv: [0.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(s, -s, -s), normal_idx: 0, uv: [1.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(s, s, -s), normal_idx: 0, uv: [1.0, 0.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(s, s, s), normal_idx: 0, uv: [0.0, 0.0], tex_layer: tl, ao },
            ]);
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        // Face 1: -X
        {
            let tl = block.texture_layer(1);
            let base = vertices.len() as u32;
            vertices.extend_from_slice(&[
                BlockVertex { position: rotate_pt(-s, -s, -s), normal_idx: 1, uv: [0.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(-s, -s, s), normal_idx: 1, uv: [1.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(-s, s, s), normal_idx: 1, uv: [1.0, 0.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(-s, s, -s), normal_idx: 1, uv: [0.0, 0.0], tex_layer: tl, ao },
            ]);
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        // Face 2: +Y
        {
            let tl = block.texture_layer(2);
            let base = vertices.len() as u32;
            vertices.extend_from_slice(&[
                BlockVertex { position: rotate_pt(-s, s, s), normal_idx: 2, uv: [0.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(s, s, s), normal_idx: 2, uv: [1.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(s, s, -s), normal_idx: 2, uv: [1.0, 0.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(-s, s, -s), normal_idx: 2, uv: [0.0, 0.0], tex_layer: tl, ao },
            ]);
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        // Face 3: -Y
        {
            let tl = block.texture_layer(3);
            let base = vertices.len() as u32;
            vertices.extend_from_slice(&[
                BlockVertex { position: rotate_pt(-s, -s, -s), normal_idx: 3, uv: [0.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(s, -s, -s), normal_idx: 3, uv: [1.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(s, -s, s), normal_idx: 3, uv: [1.0, 0.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(-s, -s, s), normal_idx: 3, uv: [0.0, 0.0], tex_layer: tl, ao },
            ]);
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        // Face 4: +Z
        {
            let tl = block.texture_layer(4);
            let base = vertices.len() as u32;
            vertices.extend_from_slice(&[
                BlockVertex { position: rotate_pt(-s, -s, s), normal_idx: 4, uv: [0.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(s, -s, s), normal_idx: 4, uv: [1.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(s, s, s), normal_idx: 4, uv: [1.0, 0.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(-s, s, s), normal_idx: 4, uv: [0.0, 0.0], tex_layer: tl, ao },
            ]);
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        // Face 5: -Z
        {
            let tl = block.texture_layer(5);
            let base = vertices.len() as u32;
            vertices.extend_from_slice(&[
                BlockVertex { position: rotate_pt(s, -s, -s), normal_idx: 5, uv: [0.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(-s, -s, -s), normal_idx: 5, uv: [1.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(-s, s, -s), normal_idx: 5, uv: [1.0, 0.0], tex_layer: tl, ao },
                BlockVertex { position: rotate_pt(s, s, -s), normal_idx: 5, uv: [0.0, 0.0], tex_layer: tl, ao },
            ]);
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    fn append_selection_box(
        vertices: &mut Vec<BlockVertex>,
        indices: &mut Vec<u32>,
        bx: i32,
        by: i32,
        bz: i32,
        break_progress: f32,
    ) {
        let pad = 0.004;
        let x0 = bx as f32 - pad;
        let y0 = by as f32 - pad;
        let z0 = bz as f32 - pad;
        let x1 = bx as f32 + 1.0 + pad;
        let y1 = by as f32 + 1.0 + pad;
        let z1 = bz as f32 + 1.0 + pad;

        let t = 0.012 + break_progress * 0.015;
        let ao = if break_progress > 0.0 { 0.85 } else { 0.15 };
        let tl = BlockType::Bedrock.texture_layer(0);

        // 12 edges of the cube as thin boxes
        let edges = [
            // Bottom 4 edges
            ([x0, y0, z0], [x1, y0 + t, z0 + t]),
            ([x0, y0, z1 - t], [x1, y0 + t, z1]),
            ([x0, y0, z0], [x0 + t, y0 + t, z1]),
            ([x1 - t, y0, z0], [x1, y0 + t, z1]),
            // Top 4 edges
            ([x0, y1 - t, z0], [x1, y1, z0 + t]),
            ([x0, y1 - t, z1 - t], [x1, y1, z1]),
            ([x0, y1 - t, z0], [x0 + t, y1, z1]),
            ([x1 - t, y1 - t, z0], [x1, y1, z1]),
            // Vertical 4 pillars
            ([x0, y0, z0], [x0 + t, y1, z0 + t]),
            ([x1 - t, y0, z0], [x1, y1, z0 + t]),
            ([x0, y0, z1 - t], [x0 + t, y1, z1]),
            ([x1 - t, y0, z1 - t], [x1, y1, z1]),
        ];

        for (min, max) in edges {
            Self::append_axis_box(vertices, indices, min, max, tl, ao);
        }
    }

    fn append_axis_box(
        vertices: &mut Vec<BlockVertex>,
        indices: &mut Vec<u32>,
        min: [f32; 3],
        max: [f32; 3],
        tl: u32,
        ao: f32,
    ) {
        let [x0, y0, z0] = min;
        let [x1, y1, z1] = max;
        let uv = [0.5, 0.5];

        let faces: [([f32; 3], [f32; 3], [f32; 3], [f32; 3], u32); 6] = [
            ([x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1], 0), // +X
            ([x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0], 1), // -X
            ([x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0], 2), // +Y
            ([x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1], 3), // -Y
            ([x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1], 4), // +Z
            ([x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0], 5), // -Z
        ];

        for (p0, p1, p2, p3, n_idx) in faces {
            let base = vertices.len() as u32;
            vertices.extend_from_slice(&[
                BlockVertex { position: p0, normal_idx: n_idx, uv, tex_layer: tl, ao },
                BlockVertex { position: p1, normal_idx: n_idx, uv, tex_layer: tl, ao },
                BlockVertex { position: p2, normal_idx: n_idx, uv, tex_layer: tl, ao },
                BlockVertex { position: p3, normal_idx: n_idx, uv, tex_layer: tl, ao },
            ]);
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dropped_item_pickup_on_ground() {
        let mut manager = DroppedItemManager::new();
        let mut inv = Inventory::new_survival();
        let world = World::new(1234);

        // Spawn 10 dropped items near player
        for i in 0..10 {
            manager.spawn(BlockType::Dirt, Vec3::new(1.0 + (i as f32) * 0.1, 40.15, 1.0));
        }
        assert_eq!(manager.items.len(), 10);

        // Set pickup delay to 0 to simulate time passed
        for item in &mut manager.items {
            item.pickup_delay = 0.0;
        }

        // Player standing at (1.0, 41.0, 1.0) on top of ground block 40
        let player_feet = Vec3::new(1.0, 41.0, 1.0);
        manager.update(&world, player_feet, &mut inv, 0.05);

        // All 10 items should be picked up into the inventory!
        assert_eq!(manager.items.len(), 0);
        assert_eq!(inv.hotbar[0].block, BlockType::Dirt);
        assert_eq!(inv.hotbar[0].count, 10);
    }

    #[test]
    fn test_nearby_items_merge_on_ground() {
        let mut manager = DroppedItemManager::new();
        let mut inv = Inventory::new_survival();
        let world = World::new(1234);

        manager.spawn(BlockType::Cobblestone, Vec3::new(10.0, 40.15, 10.0));
        manager.spawn(BlockType::Cobblestone, Vec3::new(10.1, 40.15, 10.1));
        for item in &mut manager.items {
            item.age = 1.0;
        }

        // Player far away so items merge without being picked up
        manager.update(&world, Vec3::new(100.0, 41.0, 100.0), &mut inv, 0.016);
        assert_eq!(manager.items.len(), 1);
        assert_eq!(manager.items[0].count, 2);
    }
}
