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
        player_pos: Vec3,
        inventory: &mut Inventory,
        dt: f32,
    ) {
        let mut i = 0;
        while i < self.items.len() {
            let item = &mut self.items[i];
            item.age += dt;
            item.rotation += 2.4 * dt;

            if item.pickup_delay > 0.0 {
                item.pickup_delay -= dt;
            }

            // Gravity & friction
            item.velocity.y -= 18.0 * dt;
            item.velocity.x *= (0.35_f32).powf(dt);
            item.velocity.z *= (0.35_f32).powf(dt);

            let next_pos = item.position + item.velocity * dt;

            // Simple terrain collision check
            let block_under = world.get_block(
                next_pos.x.floor() as i32,
                (next_pos.y - 0.15).floor() as i32,
                next_pos.z.floor() as i32,
            );

            if block_under.is_solid() {
                item.velocity.y = 0.0;
                let ground_y = (next_pos.y - 0.15).floor() + 1.15;
                if item.position.y < ground_y {
                    item.position.y = ground_y;
                }
            } else {
                item.position.y = next_pos.y;
            }

            // Horizontal movement if not blocked by wall
            let wall_block = world.get_block(
                next_pos.x.floor() as i32,
                item.position.y.floor() as i32,
                next_pos.z.floor() as i32,
            );
            if !wall_block.is_solid() {
                item.position.x = next_pos.x;
                item.position.z = next_pos.z;
            }

            // Player magnet / pickup
            let to_player = player_pos - item.position;
            let dist = to_player.length();

            if item.pickup_delay <= 0.0 && dist < 2.2 {
                // Attract to player
                item.velocity += to_player.normalize() * 12.0 * dt;

                if dist < 0.85 {
                    // Collect item!
                    if inventory.add_item(item.block, 1) {
                        self.items.swap_remove(i);
                        continue;
                    }
                }
            }

            // Despawn if older than 5 minutes
            if item.age > 300.0 {
                self.items.swap_remove(i);
                continue;
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
