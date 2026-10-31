use crate::voxel::world::World;
use glam::Vec3;

pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_player_pos(pos: Vec3, half_width: f32, height: f32) -> Self {
        Self {
            min: Vec3::new(pos.x - half_width, pos.y, pos.z - half_width),
            max: Vec3::new(pos.x + half_width, pos.y + height, pos.z + half_width),
        }
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }
}

pub struct Physics;

impl Physics {
    pub const PLAYER_WIDTH: f32 = 0.6;
    pub const PLAYER_HEIGHT_STAND: f32 = 1.8;
    pub const PLAYER_HEIGHT_CROUCH: f32 = 1.45;
    pub const EYE_HEIGHT_STAND: f32 = 1.62;
    pub const EYE_HEIGHT_CROUCH: f32 = 1.28;

    pub fn update_player(
        pos: &mut Vec3,
        vel: &mut Vec3,
        world: &World,
        dt: f32,
        flying: bool,
        crouching: bool,
    ) -> bool {
        let half_w = Self::PLAYER_WIDTH * 0.5;
        let player_h = if crouching && !flying {
            Self::PLAYER_HEIGHT_CROUCH
        } else {
            Self::PLAYER_HEIGHT_STAND
        };
        let mut on_ground = false;

        if !flying {
            // Apply gravity
            vel.y -= 26.0 * dt;
            // Air / ground friction
            vel.x *= 0.85;
            vel.z *= 0.85;
        } else {
            vel.x *= 0.85;
            vel.y *= 0.85;
            vel.z *= 0.85;
        }

        let mut delta = *vel * dt;

        // Crouch sneak: Ledge fall prevention (classic Minecraft mechanic!)
        if crouching && !flying {
            // Check if moving on X steps off edge
            let test_x_pos = Vec3::new(pos.x + delta.x, pos.y, pos.z);
            if !Self::has_ground_below(test_x_pos, half_w, world) {
                delta.x = 0.0;
                vel.x = 0.0;
            }
            // Check if moving on Z steps off edge
            let test_z_pos = Vec3::new(pos.x, pos.y, pos.z + delta.z);
            if !Self::has_ground_below(test_z_pos, half_w, world) {
                delta.z = 0.0;
                vel.z = 0.0;
            }
        }

        // Move and collide on X
        pos.x += delta.x;
        let mut p_box = Aabb::from_player_pos(*pos, half_w, player_h);
        for b_box in Self::get_surrounding_block_aabbs(*pos, half_w, player_h, world) {
            if p_box.intersects(&b_box) {
                if delta.x > 0.0 {
                    pos.x = b_box.min.x - half_w - 0.0001;
                } else if delta.x < 0.0 {
                    pos.x = b_box.max.x + half_w + 0.0001;
                }
                vel.x = 0.0;
                break;
            }
        }

        // Move and collide on Y
        pos.y += delta.y;
        p_box = Aabb::from_player_pos(*pos, half_w, player_h);
        for b_box in Self::get_surrounding_block_aabbs(*pos, half_w, player_h, world) {
            if p_box.intersects(&b_box) {
                if delta.y > 0.0 {
                    pos.y = b_box.min.y - player_h - 0.0001;
                } else if delta.y < 0.0 {
                    pos.y = b_box.max.y + 0.0001;
                    on_ground = true;
                }
                vel.y = 0.0;
                break;
            }
        }

        // Move and collide on Z
        pos.z += delta.z;
        p_box = Aabb::from_player_pos(*pos, half_w, player_h);
        for b_box in Self::get_surrounding_block_aabbs(*pos, half_w, player_h, world) {
            if p_box.intersects(&b_box) {
                if delta.z > 0.0 {
                    pos.z = b_box.min.z - half_w - 0.0001;
                } else if delta.z < 0.0 {
                    pos.z = b_box.max.z + half_w + 0.0001;
                }
                vel.z = 0.0;
                break;
            }
        }

        on_ground
    }

    fn has_ground_below(pos: Vec3, half_w: f32, world: &World) -> bool {
        let min_x = (pos.x - half_w).floor() as i32;
        let max_x = (pos.x + half_w).floor() as i32;
        let min_z = (pos.z - half_w).floor() as i32;
        let max_z = (pos.z + half_w).floor() as i32;
        let check_y = (pos.y - 0.2).floor() as i32;

        for z in min_z..=max_z {
            for x in min_x..=max_x {
                if world.get_block(x, check_y, z).is_solid() {
                    return true;
                }
            }
        }
        false
    }

    fn get_surrounding_block_aabbs(
        pos: Vec3,
        half_w: f32,
        height: f32,
        world: &World,
    ) -> Vec<Aabb> {
        let min_x = (pos.x - half_w - 1.0).floor() as i32;
        let max_x = (pos.x + half_w + 1.0).ceil() as i32;
        let min_y = (pos.y - 1.0).floor() as i32;
        let max_y = (pos.y + height + 1.0).ceil() as i32;
        let min_z = (pos.z - half_w - 1.0).floor() as i32;
        let max_z = (pos.z + half_w + 1.0).ceil() as i32;

        let mut boxes = Vec::new();
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                for x in min_x..=max_x {
                    let block = world.get_block(x, y, z);
                    if block.is_solid() {
                        boxes.push(Aabb::new(
                            Vec3::new(x as f32, y as f32, z as f32),
                            Vec3::new(x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0),
                        ));
                    }
                }
            }
        }
        boxes
    }
}
