use crate::voxel::block::BlockType;
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

#[derive(Debug, Clone, Copy, Default)]
pub struct EnvironmentContact {
    pub on_ground: bool,
    pub in_water: bool,
    pub in_lava: bool,
    pub touching_cactus: bool,
}

pub struct Physics;

impl Physics {
    pub const PLAYER_WIDTH: f32 = 0.6;
    pub const PLAYER_HEIGHT_STAND: f32 = 1.8;
    pub const PLAYER_HEIGHT_CROUCH: f32 = 1.45;
    pub const EYE_HEIGHT_STAND: f32 = 1.62;
    pub const EYE_HEIGHT_CROUCH: f32 = 1.28;

    /// Check if standing up to full height at `pos` would intersect any solid ceiling block
    pub fn can_stand_up(pos: Vec3, world: &World) -> bool {
        let half_w = Self::PLAYER_WIDTH * 0.5 - 0.001;
        let stand_box = Aabb::from_player_pos(pos, half_w, Self::PLAYER_HEIGHT_STAND);
        for b_box in Self::get_surrounding_block_aabbs(pos, half_w, Self::PLAYER_HEIGHT_STAND, world) {
            if stand_box.intersects(&b_box) {
                return false;
            }
        }
        true
    }

    pub fn check_environment(pos: Vec3, height: f32, world: &World) -> (bool, bool, bool) {
        let half_w = Self::PLAYER_WIDTH * 0.5 + 0.05;
        let min_x = (pos.x - half_w).floor() as i32;
        let max_x = (pos.x + half_w).floor() as i32;
        let min_y = pos.y.floor() as i32;
        let max_y = (pos.y + height - 0.1).floor() as i32;
        let min_z = (pos.z - half_w).floor() as i32;
        let max_z = (pos.z + half_w).floor() as i32;

        let mut in_water = false;
        let mut in_lava = false;
        let mut touching_cactus = false;

        for y in min_y..=max_y {
            for z in min_z..=max_z {
                for x in min_x..=max_x {
                    match world.get_block(x, y, z) {
                        BlockType::Water => in_water = true,
                        BlockType::Lava => in_lava = true,
                        BlockType::Cactus => touching_cactus = true,
                        _ => {}
                    }
                }
            }
        }
        (in_water, in_lava, touching_cactus)
    }

    pub fn update_player(
        pos: &mut Vec3,
        vel: &mut Vec3,
        world: &World,
        dt: f32,
        flying: bool,
        crouching: bool,
        was_on_ground: bool,
    ) -> EnvironmentContact {
        let half_w = Self::PLAYER_WIDTH * 0.5;
        let player_h = if crouching && !flying {
            Self::PLAYER_HEIGHT_CROUCH
        } else {
            Self::PLAYER_HEIGHT_STAND
        };
        let mut on_ground = false;

        let (in_water, in_lava, _) = Self::check_environment(*pos, player_h, world);
        let in_fluid = in_water || in_lava;

        let damping = if in_fluid && !flying {
            (-16.0 * dt).exp()
        } else {
            (-12.0 * dt).exp()
        };

        if !flying {
            if in_fluid {
                // Gentle buoyancy / fluid sinking speed
                vel.y = (vel.y - 7.5 * dt).clamp(-4.5, 6.0);
            } else {
                // Standard gravity
                vel.y = (vel.y - 26.0 * dt).max(-60.0);
            }
            vel.x *= damping;
            vel.z *= damping;
        } else {
            vel.x *= damping;
            vel.y *= damping;
            vel.z *= damping;
        }

        let mut delta = *vel * dt;

        // Crouch sneak: Ledge fall prevention (only when on ground!)
        if crouching && !flying && was_on_ground && Self::has_ground_below(*pos, half_w, world) {
            let test_x_pos = Vec3::new(pos.x + delta.x, pos.y, pos.z);
            if !Self::has_ground_below(test_x_pos, half_w, world) {
                delta.x = 0.0;
                vel.x = 0.0;
            }
            let test_z_pos = Vec3::new(pos.x, pos.y, pos.z + delta.z);
            if !Self::has_ground_below(test_z_pos, half_w, world) {
                delta.z = 0.0;
                vel.z = 0.0;
            }
            if delta.x != 0.0 && delta.z != 0.0 {
                let test_xz_pos = Vec3::new(pos.x + delta.x, pos.y, pos.z + delta.z);
                if !Self::has_ground_below(test_xz_pos, half_w, world) {
                    delta.x = 0.0;
                    delta.z = 0.0;
                    vel.x = 0.0;
                    vel.z = 0.0;
                }
            }
        }

        // Move and collide on Y first so floor alignment is exact before X/Z wall checks
        pos.y += delta.y;
        let mut p_box = Aabb::from_player_pos(*pos, half_w, player_h);
        for b_box in Self::get_surrounding_block_aabbs(*pos, half_w, player_h, world) {
            if p_box.intersects(&b_box) {
                if delta.y > 0.0 {
                    pos.y = b_box.min.y - player_h - 0.0001;
                } else if delta.y < 0.0 {
                    pos.y = b_box.max.y + 0.0001;
                    on_ground = true;
                }
                vel.y = 0.0;
                p_box = Aabb::from_player_pos(*pos, half_w, player_h);
            }
        }

        // Move and collide on X
        pos.x += delta.x;
        p_box = Aabb::from_player_pos(*pos, half_w, player_h);
        for b_box in Self::get_surrounding_block_aabbs(*pos, half_w, player_h, world) {
            if p_box.intersects(&b_box) {
                if delta.x > 0.0 {
                    pos.x = b_box.min.x - half_w - 0.0001;
                } else if delta.x < 0.0 {
                    pos.x = b_box.max.x + half_w + 0.0001;
                }
                vel.x = 0.0;
                p_box = Aabb::from_player_pos(*pos, half_w, player_h);
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
                p_box = Aabb::from_player_pos(*pos, half_w, player_h);
            }
        }

        let (post_water, post_lava, touching_cactus) = Self::check_environment(*pos, player_h, world);

        EnvironmentContact {
            on_ground,
            in_water: in_water || post_water,
            in_lava: in_lava || post_lava,
            touching_cactus,
        }
    }

    pub fn has_ground_below(pos: Vec3, half_w: f32, world: &World) -> bool {
        let eps = 0.001;
        let min_x = (pos.x - half_w + eps).floor() as i32;
        let max_x = (pos.x + half_w - eps).floor() as i32;
        let min_z = (pos.z - half_w + eps).floor() as i32;
        let max_z = (pos.z + half_w - eps).floor() as i32;
        let check_y = (pos.y - 0.08).floor() as i32;

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
