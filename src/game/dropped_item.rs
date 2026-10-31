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
    pub rotation: f32,
    pub age: f32,
    pub pickup_delay: f32,
}

pub struct DroppedItemManager {
    pub items: Vec<DroppedItem>,
}

impl DroppedItemManager {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn spawn(&mut self, block: BlockType, pos: Vec3) {
        if block == BlockType::Air {
            return;
        }

        // Pseudo-random initial velocity popping out of the block
        let seed = (pos.x.abs() * 100.0 + pos.z.abs() * 37.0 + self.items.len() as f32 * 17.0) as u32;
        let angle = (seed % 360) as f32 * std::f32::consts::PI / 180.0;
        let speed = 1.2;

        self.items.push(DroppedItem {
            position: pos,
            velocity: Vec3::new(angle.cos() * speed, 3.2, angle.sin() * speed),
            block,
            rotation: angle,
            age: 0.0,
            pickup_delay: 0.45,
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
        let mut i = 0;

        while i < self.items.len() {
            let item = &mut self.items[i];
            item.age += dt;
            item.rotation += 2.4 * dt;

            if item.pickup_delay > 0.0 {
                item.pickup_delay -= dt;
            }

            // Despawn if older than 5 minutes
            if item.age > 300.0 {
                self.items.swap_remove(i);
                continue;
            }

            // Check if player is already within pickup range
            let dx = item.position.x - player_feet.x;
            let dz = item.position.z - player_feet.z;
            let horiz_dist = (dx * dx + dz * dz).sqrt();
            let dy = item.position.y - player_feet.y;

            if item.pickup_delay <= 0.0 && horiz_dist < 1.75 && dy >= -0.8 && dy <= 2.2 {
                if inventory.add_item(item.block, 1) {
                    self.items.swap_remove(i);
                    continue;
                }
            }

            // Magnet attraction towards player center when nearby
            let to_center = player_center - item.position;
            let dist_to_center = to_center.length();

            if item.pickup_delay <= 0.0 && dist_to_center < 3.5 {
                let pull_dir = to_center.normalize();
                item.velocity += pull_dir * 32.0 * dt;
                let cur_spd = item.velocity.length();
                if cur_spd > 12.0 {
                    item.velocity = item.velocity * (12.0 / cur_spd);
                }
            } else {
                // Regular physics when not pulled by magnet
                item.velocity.y -= 22.0 * dt;
                item.velocity.x *= (0.2_f32).powf(dt);
                item.velocity.z *= (0.2_f32).powf(dt);
            }

            // Vertical movement and collision
            let next_y = item.position.y + item.velocity.y * dt;
            if item.velocity.y < 0.0 {
                let test_y = next_y - 0.15;
                let bx = item.position.x.floor() as i32;
                let by = test_y.floor() as i32;
                let bz = item.position.z.floor() as i32;
                let block = world.get_block(bx, by, bz);
                if block.is_solid() {
                    let top_y = by as f32 + 1.0 + 0.15;
                    item.position.y = top_y;
                    item.velocity.y = 0.0;
                } else {
                    item.position.y = next_y;
                }
            } else if item.velocity.y > 0.0 {
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
            let test_x = if item.velocity.x >= 0.0 { next_x + 0.15 } else { next_x - 0.15 };
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
            let test_z = if item.velocity.z >= 0.0 { next_z + 0.15 } else { next_z - 0.15 };
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

            if item.pickup_delay <= 0.0 && horiz_dist_post < 1.75 && dy_post >= -0.8 && dy_post <= 2.2 {
                if inventory.add_item(item.block, 1) {
                    self.items.swap_remove(i);
                    continue;
                }
            }

            i += 1;
        }
    }

    pub fn generate_mesh(&self) -> (Vec<BlockVertex>, Vec<u32>) {
        let mut vertices = Vec::with_capacity(self.items.len() * 24);
        let mut indices = Vec::with_capacity(self.items.len() * 36);

        let half_size = 0.125; // 0.25 total width

        for item in &self.items {
            let bobbing_y = (item.age * 3.5).sin() * 0.06;
            let center = item.position + Vec3::new(0.0, bobbing_y, 0.0);
            let rot = item.rotation;
            let cos_r = rot.cos();
            let sin_r = rot.sin();

            // Rotates point around Y axis relative to center
            let rotate_pt = |dx: f32, dy: f32, dz: f32| -> [f32; 3] {
                let rx = dx * cos_r - dz * sin_r;
                let rz = dx * sin_r + dz * cos_r;
                [center.x + rx, center.y + dy, center.z + rz]
            };

            let s = half_size;

            // Generate 6 faces for the mini-cube
            // Face 0: +X
            {
                let tl = item.block.texture_layer(0);
                let base = vertices.len() as u32;
                vertices.extend_from_slice(&[
                    BlockVertex { position: rotate_pt(s, -s, s), normal_idx: 0, uv: [0.0, 1.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(s, -s, -s), normal_idx: 0, uv: [1.0, 1.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(s, s, -s), normal_idx: 0, uv: [1.0, 0.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(s, s, s), normal_idx: 0, uv: [0.0, 0.0], tex_layer: tl, ao: 1.0 },
                ]);
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
            // Face 1: -X
            {
                let tl = item.block.texture_layer(1);
                let base = vertices.len() as u32;
                vertices.extend_from_slice(&[
                    BlockVertex { position: rotate_pt(-s, -s, -s), normal_idx: 1, uv: [0.0, 1.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(-s, -s, s), normal_idx: 1, uv: [1.0, 1.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(-s, s, s), normal_idx: 1, uv: [1.0, 0.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(-s, s, -s), normal_idx: 1, uv: [0.0, 0.0], tex_layer: tl, ao: 1.0 },
                ]);
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
            // Face 2: +Y
            {
                let tl = item.block.texture_layer(2);
                let base = vertices.len() as u32;
                vertices.extend_from_slice(&[
                    BlockVertex { position: rotate_pt(-s, s, s), normal_idx: 2, uv: [0.0, 1.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(s, s, s), normal_idx: 2, uv: [1.0, 1.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(s, s, -s), normal_idx: 2, uv: [1.0, 0.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(-s, s, -s), normal_idx: 2, uv: [0.0, 0.0], tex_layer: tl, ao: 1.0 },
                ]);
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
            // Face 3: -Y
            {
                let tl = item.block.texture_layer(3);
                let base = vertices.len() as u32;
                vertices.extend_from_slice(&[
                    BlockVertex { position: rotate_pt(-s, -s, -s), normal_idx: 3, uv: [0.0, 1.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(s, -s, -s), normal_idx: 3, uv: [1.0, 1.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(s, -s, s), normal_idx: 3, uv: [1.0, 0.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(-s, -s, s), normal_idx: 3, uv: [0.0, 0.0], tex_layer: tl, ao: 1.0 },
                ]);
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
            // Face 4: +Z
            {
                let tl = item.block.texture_layer(4);
                let base = vertices.len() as u32;
                vertices.extend_from_slice(&[
                    BlockVertex { position: rotate_pt(-s, -s, s), normal_idx: 4, uv: [0.0, 1.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(s, -s, s), normal_idx: 4, uv: [1.0, 1.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(s, s, s), normal_idx: 4, uv: [1.0, 0.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(-s, s, s), normal_idx: 4, uv: [0.0, 0.0], tex_layer: tl, ao: 1.0 },
                ]);
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
            // Face 5: -Z
            {
                let tl = item.block.texture_layer(5);
                let base = vertices.len() as u32;
                vertices.extend_from_slice(&[
                    BlockVertex { position: rotate_pt(s, -s, -s), normal_idx: 5, uv: [0.0, 1.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(-s, -s, -s), normal_idx: 5, uv: [1.0, 1.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(-s, s, -s), normal_idx: 5, uv: [1.0, 0.0], tex_layer: tl, ao: 1.0 },
                    BlockVertex { position: rotate_pt(s, s, -s), normal_idx: 5, uv: [0.0, 0.0], tex_layer: tl, ao: 1.0 },
                ]);
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }

        (vertices, indices)
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
}
