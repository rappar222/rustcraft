use crate::game::dropped_item::DroppedItemManager;
use crate::game::state::GameMode;
use crate::game::survival::SurvivalState;
use crate::render::texture::TextureManager;
use crate::render::vertex::BlockVertex;
use crate::voxel::block::BlockType;
use crate::voxel::world::{World, SEA_LEVEL};
use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobKind {
    Pig,
    Zombie,
    Creeper,
}

impl MobKind {
    pub fn max_health(self) -> f32 {
        match self {
            MobKind::Pig => 10.0,
            MobKind::Zombie => 20.0,
            MobKind::Creeper => 20.0,
        }
    }

    pub fn move_speed(self) -> f32 {
        match self {
            MobKind::Pig => 2.2,
            MobKind::Zombie => 3.1,
            MobKind::Creeper => 2.9,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mob {
    pub kind: MobKind,
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub health: f32,
    pub on_ground: bool,
    pub hurt_timer: f32,
    pub walk_phase: f32,
    pub ai_timer: f32,
    pub fuse_timer: f32, // Used by Creeper before exploding
    pub panic_timer: f32, // Used by Pig when hit
}

pub struct MobManager {
    pub mobs: Vec<Mob>,
    spawn_cooldown: f32,
    rng_state: u32,
}

impl MobManager {
    pub fn new() -> Self {
        Self {
            mobs: Vec::new(),
            spawn_cooldown: 2.0,
            rng_state: 0x9E37_79B9,
        }
    }

    pub fn clear(&mut self) {
        self.mobs.clear();
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng_state = x;
        x
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u32() % 10000) as f32 / 10000.0
    }

    pub fn spawn_mob(&mut self, kind: MobKind, position: Vec3) {
        self.mobs.push(Mob {
            kind,
            position,
            velocity: Vec3::ZERO,
            yaw: 0.0,
            health: kind.max_health(),
            on_ground: false,
            hurt_timer: 0.0,
            walk_phase: 0.0,
            ai_timer: 1.0,
            fuse_timer: 0.0,
            panic_timer: 0.0,
        });
    }

    /// Raycast from player's camera to check if a mob is hit within `max_reach` (4.0m).
    /// Returns true if a mob was struck!
    pub fn try_attack_mob(
        &mut self,
        ray_origin: Vec3,
        ray_dir: Vec3,
        max_reach: f32,
        damage: f32,
        dropped_items: &mut DroppedItemManager,
    ) -> bool {
        let dir = ray_dir.normalize_or_zero();
        let mut best_idx = None;
        let mut best_dist = max_reach;

        for (idx, mob) in self.mobs.iter().enumerate() {
            let half_w = 0.45;
            let height = if mob.kind == MobKind::Pig { 1.1 } else { 1.85 };
            let min = Vec3::new(mob.position.x - half_w, mob.position.y, mob.position.z - half_w);
            let max = Vec3::new(mob.position.x + half_w, mob.position.y + height, mob.position.z + half_w);

            if let Some(t) = Self::ray_aabb(ray_origin, dir, min, max) {
                if t >= 0.0 && t < best_dist {
                    best_dist = t;
                    best_idx = Some(idx);
                }
            }
        }

        if let Some(idx) = best_idx {
            let mob = &mut self.mobs[idx];
            if mob.hurt_timer <= 0.1 {
                mob.health -= damage;
                mob.hurt_timer = 0.38;
                // Knockback away from attacker
                let kb_dir = Vec3::new(dir.x, 0.0, dir.z).normalize_or_zero();
                mob.velocity.x = kb_dir.x * 6.5;
                mob.velocity.z = kb_dir.z * 6.5;
                mob.velocity.y = 4.2;
                mob.on_ground = false;
                if mob.kind == MobKind::Pig {
                    mob.panic_timer = 4.0;
                    mob.yaw = kb_dir.z.atan2(kb_dir.x);
                }
            }

            if mob.health <= 0.0 {
                let dead = self.mobs.swap_remove(idx);
                let drop_pos = dead.position + Vec3::new(0.0, 0.5, 0.0);
                match dead.kind {
                    MobKind::Pig => {
                        dropped_items.spawn_stack(BlockType::RawPorkchop, 2, drop_pos);
                    }
                    MobKind::Zombie => {
                        dropped_items.spawn_stack(BlockType::RottenFlesh, 2, drop_pos);
                        dropped_items.spawn(BlockType::IronIngot, drop_pos);
                    }
                    MobKind::Creeper => {
                        dropped_items.spawn_stack(BlockType::Coal, 2, drop_pos);
                        dropped_items.spawn(BlockType::TNT, drop_pos);
                    }
                }
            }
            true
        } else {
            false
        }
    }

    fn ray_aabb(origin: Vec3, dir: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
        let mut tmin = f32::NEG_INFINITY;
        let mut tmax = f32::INFINITY;

        for axis in 0..3 {
            let o = origin[axis];
            let d = dir[axis];
            let bmin = min[axis];
            let bmax = max[axis];

            if d.abs() < 1e-6 {
                if o < bmin || o > bmax {
                    return None;
                }
            } else {
                let mut t1 = (bmin - o) / d;
                let mut t2 = (bmax - o) / d;
                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                }
                tmin = tmin.max(t1);
                tmax = tmax.min(t2);
                if tmin > tmax {
                    return None;
                }
            }
        }

        if tmax >= 0.0 {
            Some(tmin.max(0.0))
        } else {
            None
        }
    }

    pub fn update(
        &mut self,
        world: &mut World,
        player_pos: Vec3,
        player_vel: &mut Vec3,
        game_mode: GameMode,
        survival: &mut SurvivalState,
        dropped_items: &mut DroppedItemManager,
        daylight: f32,
        dt: f32,
    ) {
        // Periodic natural mob spawning around player
        self.spawn_cooldown -= dt;
        if self.spawn_cooldown <= 0.0 {
            self.spawn_cooldown = 3.5;
            if self.mobs.len() < 14 {
                let angle = self.next_f32() * std::f32::consts::TAU;
                let dist = 16.0 + self.next_f32() * 18.0;
                let sx = (player_pos.x + angle.cos() * dist).floor() as i32;
                let sz = (player_pos.z + angle.sin() * dist).floor() as i32;
                let sy = world.get_surface_height(sx, sz);
                let ground = world.get_block(sx, sy, sz);

                if sy >= SEA_LEVEL as i32 && ground.is_solid() && ground != BlockType::Cactus {
                    let kind = if daylight > 0.55 {
                        MobKind::Pig
                    } else if self.next_u32() % 3 == 0 {
                        MobKind::Creeper
                    } else {
                        MobKind::Zombie
                    };
                    self.spawn_mob(kind, Vec3::new(sx as f32 + 0.5, sy as f32 + 1.01, sz as f32 + 0.5));
                }
            }
        }

        let mut i = 0;
        while i < self.mobs.len() {
            let to_player = player_pos - self.mobs[i].position;
            let dist_sq = to_player.length_squared();

            // Despawn if too far or in void
            if dist_sq > 68.0 * 68.0 || self.mobs[i].position.y < -25.0 {
                self.mobs.swap_remove(i);
                continue;
            }

            let dist = dist_sq.sqrt();
            let mob = &mut self.mobs[i];

            if mob.hurt_timer > 0.0 {
                mob.hurt_timer = (mob.hurt_timer - dt).max(0.0);
            }
            if mob.panic_timer > 0.0 {
                mob.panic_timer = (mob.panic_timer - dt).max(0.0);
            }

            // AI Behavior
            let mut desired_speed = 0.0;
            match mob.kind {
                MobKind::Pig => {
                    mob.ai_timer -= dt;
                    if mob.panic_timer > 0.0 {
                        desired_speed = mob.kind.move_speed() * 1.65;
                    } else if mob.ai_timer <= 0.0 {
                        let r = ((mob.position.x.to_bits() ^ mob.position.z.to_bits()).wrapping_add((dt * 1000.0) as u32)) % 100;
                        mob.yaw += (r as f32 - 50.0) * 0.04;
                        mob.ai_timer = 2.0 + (r % 3) as f32;
                    } else if (mob.ai_timer * 2.0) as i32 % 2 == 0 {
                        desired_speed = mob.kind.move_speed() * 0.55;
                    }
                }
                MobKind::Zombie => {
                    // Zombies burn in bright sunlight unless in water
                    let in_water = world.get_block(
                        mob.position.x.floor() as i32,
                        mob.position.y.floor() as i32,
                        mob.position.z.floor() as i32,
                    ) == BlockType::Water;
                    if daylight > 0.82 && !in_water && mob.position.y >= SEA_LEVEL as f32 {
                        mob.health -= 1.5 * dt;
                        if mob.hurt_timer <= 0.0 {
                            mob.hurt_timer = 0.2;
                        }
                    }

                    if game_mode == GameMode::Survival && dist < 20.0 && dist > 0.3 {
                        mob.yaw = to_player.z.atan2(to_player.x);
                        desired_speed = mob.kind.move_speed();

                        // Melee attack when close to player
                        if dist < 1.35 && (to_player.y).abs() < 1.8 {
                            if survival.take_damage(3.0) {
                                let kb = Vec3::new(to_player.x, 0.0, to_player.z).normalize_or_zero();
                                player_vel.x += kb.x * 5.5;
                                player_vel.z += kb.z * 5.5;
                                player_vel.y = 4.0;
                            }
                        }
                    }
                }
                MobKind::Creeper => {
                    if game_mode == GameMode::Survival && dist < 20.0 && dist > 0.3 {
                        mob.yaw = to_player.z.atan2(to_player.x);
                        if dist < 2.8 {
                            mob.fuse_timer += dt;
                            desired_speed = 0.4;
                        } else {
                            mob.fuse_timer = (mob.fuse_timer - dt * 1.5).max(0.0);
                            desired_speed = mob.kind.move_speed();
                        }
                    } else {
                        mob.fuse_timer = (mob.fuse_timer - dt * 1.5).max(0.0);
                    }
                }
            }

            // Check if zombie died from sunlight or creeper fuse completed
            if mob.health <= 0.0 {
                let dead = self.mobs.swap_remove(i);
                dropped_items.spawn(BlockType::RottenFlesh, dead.position + Vec3::new(0.0, 0.4, 0.0));
                continue;
            }

            if mob.kind == MobKind::Creeper && mob.fuse_timer >= 1.4 {
                let creeper = self.mobs.swap_remove(i);
                let cpos = creeper.position;
                let blast_drops = world.explode(
                    cpos.x.floor() as i32,
                    cpos.y.floor() as i32,
                    cpos.z.floor() as i32,
                    3.2,
                );
                for (b, p) in blast_drops {
                    dropped_items.spawn(b, p);
                }
                if dist < 5.0 && game_mode == GameMode::Survival {
                    let blast_dmg = ((5.0 - dist) * 3.2).max(1.0);
                    survival.take_damage(blast_dmg);
                    let kb = (player_pos - cpos).normalize_or_zero();
                    player_vel.x += kb.x * 9.0;
                    player_vel.z += kb.z * 9.0;
                    player_vel.y = 6.0;
                }
                continue;
            }

            // Apply horizontal velocity and physics
            let dir_x = mob.yaw.cos();
            let dir_z = mob.yaw.sin();
            if mob.hurt_timer < 0.2 {
                mob.velocity.x = dir_x * desired_speed;
                mob.velocity.z = dir_z * desired_speed;
            } else {
                let drag = (-6.0 * dt).exp();
                mob.velocity.x *= drag;
                mob.velocity.z *= drag;
            }

            if desired_speed > 0.1 && mob.on_ground {
                mob.walk_phase += dt * desired_speed * 2.8;
            }

            mob.velocity.y = (mob.velocity.y - 24.0 * dt).max(-45.0);

            // Vertical floor collision
            let next_y = mob.position.y + mob.velocity.y * dt;
            let foot_block = world.get_block(
                mob.position.x.floor() as i32,
                (next_y - 0.05).floor() as i32,
                mob.position.z.floor() as i32,
            );
            if mob.velocity.y <= 0.0 && foot_block.is_solid() {
                mob.position.y = (next_y - 0.05).floor() + 1.001;
                mob.velocity.y = 0.0;
                mob.on_ground = true;
            } else {
                mob.position.y = next_y;
                mob.on_ground = false;
            }

            // Horizontal movement + automatic 1-block step-up jump!
            let next_x = mob.position.x + mob.velocity.x * dt;
            let next_z = mob.position.z + mob.velocity.z * dt;
            let front_block = world.get_block(
                (next_x + dir_x * 0.35).floor() as i32,
                mob.position.y.floor() as i32,
                (next_z + dir_z * 0.35).floor() as i32,
            );
            let above_front = world.get_block(
                (next_x + dir_x * 0.35).floor() as i32,
                (mob.position.y + 1.0).floor() as i32,
                (next_z + dir_z * 0.35).floor() as i32,
            );

            if front_block.is_solid() {
                if mob.on_ground && !above_front.is_solid() {
                    mob.velocity.y = 7.8; // Jump up 1-block ledge!
                    mob.on_ground = false;
                }
            } else {
                mob.position.x = next_x;
                mob.position.z = next_z;
            }

            i += 1;
        }
    }

    pub fn append_mob_and_hand_meshes(
        &self,
        vertices: &mut Vec<BlockVertex>,
        indices: &mut Vec<u32>,
        cam_pos: Vec3,
        cam_front: Vec3,
        cam_right: Vec3,
        cam_up: Vec3,
        held_item: BlockType,
        swing_timer: f32,
        walk_bob: f32,
    ) {
        // 1. Render all living 3D articulated Mobs
        for mob in &self.mobs {
            let ao = if mob.hurt_timer > 0.0
                || (mob.kind == MobKind::Creeper && (mob.fuse_timer * 10.0) as i32 % 2 == 1)
            {
                1.55
            } else {
                0.95
            };
            let leg_swing = mob.walk_phase.sin() * 0.22;

            match mob.kind {
                MobKind::Pig => {
                    let tl_skin = TextureManager::LAYER_PIG_SKIN;
                    let tl_face = TextureManager::LAYER_PIG_FACE;
                    // Body
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.0, 0.46, 0.0], [0.40, 0.22, 0.25], tl_skin, tl_skin, ao);
                    // Head with Pig Face on front (+X)
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.42, 0.60, 0.0], [0.20, 0.20, 0.20], tl_face, tl_skin, ao);
                    // 3D Protruding Pig Snout
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.64, 0.54, 0.0], [0.05, 0.07, 0.09], tl_face, tl_skin, ao * 1.04);
                    // 4 Animated Legs
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.25 + leg_swing, 0.14, 0.15], [0.09, 0.14, 0.09], tl_skin, tl_skin, ao * 0.88);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.25 - leg_swing, 0.14, -0.15], [0.09, 0.14, 0.09], tl_skin, tl_skin, ao * 0.88);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [-0.25 - leg_swing, 0.14, 0.15], [0.09, 0.14, 0.09], tl_skin, tl_skin, ao * 0.88);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [-0.25 + leg_swing, 0.14, -0.15], [0.09, 0.14, 0.09], tl_skin, tl_skin, ao * 0.88);
                }
                MobKind::Zombie => {
                    let tl_skin = TextureManager::LAYER_ZOMBIE_SKIN;
                    let tl_face = TextureManager::LAYER_ZOMBIE_FACE;
                    let tl_shirt = TextureManager::LAYER_ZOMBIE_SHIRT;
                    let tl_pants = TextureManager::LAYER_ZOMBIE_PANTS;
                    // Legs
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [leg_swing * 0.7, 0.36, 0.11], [0.11, 0.36, 0.11], tl_pants, tl_pants, ao * 0.85);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [-leg_swing * 0.7, 0.36, -0.11], [0.11, 0.36, 0.11], tl_pants, tl_pants, ao * 0.85);
                    // Torso
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.0, 1.06, 0.0], [0.13, 0.36, 0.22], tl_shirt, tl_shirt, ao);
                    // Outstretched Zombie Arms
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.26, 1.28, 0.32], [0.28, 0.10, 0.10], tl_skin, tl_skin, ao);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.26, 1.28, -0.32], [0.28, 0.10, 0.10], tl_skin, tl_skin, ao);
                    // Head with Zombie Face on front (+X)
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.0, 1.62, 0.0], [0.21, 0.21, 0.21], tl_face, tl_skin, ao);
                }
                MobKind::Creeper => {
                    let tl_skin = TextureManager::LAYER_CREEPER_SKIN;
                    let tl_face = TextureManager::LAYER_CREEPER_FACE;
                    // 4 Short Creeper Feet
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.18 + leg_swing, 0.18, 0.14], [0.11, 0.18, 0.11], tl_skin, tl_skin, ao * 0.85);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.18 - leg_swing, 0.18, -0.14], [0.11, 0.18, 0.11], tl_skin, tl_skin, ao * 0.85);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [-0.18 - leg_swing, 0.18, 0.14], [0.11, 0.18, 0.11], tl_skin, tl_skin, ao * 0.85);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [-0.18 + leg_swing, 0.18, -0.14], [0.11, 0.18, 0.11], tl_skin, tl_skin, ao * 0.85);
                    // Tall Torso
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.0, 0.80, 0.0], [0.15, 0.45, 0.19], tl_skin, tl_skin, ao);
                    // Head with Iconic Creeper Face on front (+X)
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.0, 1.45, 0.0], [0.21, 0.21, 0.21], tl_face, tl_skin, ao);
                }
            }
        }

        // 2. Render 3D First-Person Right Hand / Held Block / Held Tool in camera view space!
        let bob_x = (walk_bob * 0.5).cos() * 0.016;
        let bob_y = (walk_bob).sin().abs() * 0.020;
        let swing_angle = (swing_timer * std::f32::consts::PI).sin();

        let hand_pos = cam_pos
            + cam_front * (0.50 - swing_angle * 0.10)
            + cam_right * (0.33 + bob_x - swing_angle * 0.08)
            - cam_up * (0.24 - bob_y + swing_angle * 0.12);

        if held_item == BlockType::Air {
            // 3D First-person right forearm angled forward into view (right-handed basis: ax x ay = az)
            let arm_forward = (cam_front * 0.82 - cam_right * 0.35 + cam_up * (0.42 - swing_angle * 0.65)).normalize_or_zero();
            let arm_right = arm_forward.cross(cam_up).normalize_or_zero();
            let arm_up = arm_forward.cross(arm_right).normalize_or_zero();
            Self::append_basis_box(
                vertices,
                indices,
                hand_pos - arm_forward * 0.06,
                arm_right * 0.055,
                arm_up * 0.055,
                arm_forward * 0.16,
                TextureManager::LAYER_PLAYER_SKIN,
                1.0,
            );
        } else if held_item.is_placeable() && held_item != BlockType::Torch {
            // Held 3D mini-block rotated 45 deg in camera space so it follows pitch & yaw
            let s = 0.105;
            let c45 = 0.70710677;
            let raw_x = (cam_right * c45 + cam_front * c45).normalize_or_zero();
            let raw_y = (cam_up * (1.0 - swing_angle * 0.25) - cam_front * (swing_angle * 0.25)).normalize_or_zero();
            let dir_z = raw_x.cross(raw_y).normalize_or_zero();
            let ortho_y = dir_z.cross(raw_x).normalize_or_zero();
            Self::append_basis_block(
                vertices,
                indices,
                hand_pos,
                raw_x * s,
                ortho_y * s,
                dir_z * s,
                held_item,
                1.05,
            );
        } else {
            // Held 3D pixel-extruded Tool / Weapon / Torch / Food in right hand!
            // Sprite +X/+Y diagonal points forward and up; sprite plane faces inward-left
            let pitch_swing = swing_angle * 0.75;
            let dir_x = (cam_front * (0.78 + pitch_swing * 0.35) - cam_right * 0.32 - cam_up * (0.20 + pitch_swing * 0.55)).normalize_or_zero();
            let dir_y = (cam_up * (0.85 - pitch_swing * 0.45) + cam_front * (0.38 - pitch_swing * 0.45) - cam_right * 0.15).normalize_or_zero();
            let dir_z = dir_x.cross(dir_y).normalize_or_zero();
            let ortho_y = dir_z.cross(dir_x).normalize_or_zero();

            let scale = 0.165;
            let half_thick = 0.016;
            DroppedItemManager::append_extruded_sprite(
                vertices,
                indices,
                hand_pos + cam_up * 0.03,
                dir_x * scale,
                ortho_y * scale,
                dir_z * half_thick,
                held_item.texture_layer(0),
                1.12,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn append_oriented_part(
        vertices: &mut Vec<BlockVertex>,
        indices: &mut Vec<u32>,
        origin: Vec3,
        yaw: f32,
        local_offset: [f32; 3],
        half_extents: [f32; 3],
        front_tl: u32,
        side_tl: u32,
        ao: f32,
    ) {
        let cos_y = yaw.cos();
        let sin_y = yaw.sin();
        let [ox, oy, oz] = local_offset;
        let [hx, hy, hz] = half_extents;

        let xform = |lx: f32, ly: f32, lz: f32| -> [f32; 3] {
            let px = ox + lx;
            let py = oy + ly;
            let pz = oz + lz;
            [
                origin.x + px * cos_y - pz * sin_y,
                origin.y + py,
                origin.z + px * sin_y + pz * cos_y,
            ]
        };

        let faces: [([f32; 3], [f32; 3], [f32; 3], [f32; 3], u32, u32); 6] = [
            (xform(hx, -hy, hz), xform(hx, -hy, -hz), xform(hx, hy, -hz), xform(hx, hy, hz), 0, front_tl),
            (xform(-hx, -hy, -hz), xform(-hx, -hy, hz), xform(-hx, hy, hz), xform(-hx, hy, -hz), 1, side_tl),
            (xform(-hx, hy, hz), xform(hx, hy, hz), xform(hx, hy, -hz), xform(-hx, hy, -hz), 2, side_tl),
            (xform(-hx, -hy, -hz), xform(hx, -hy, -hz), xform(hx, -hy, hz), xform(-hx, -hy, hz), 3, side_tl),
            (xform(-hx, -hy, hz), xform(hx, -hy, hz), xform(hx, hy, hz), xform(-hx, hy, hz), 4, side_tl),
            (xform(hx, -hy, -hz), xform(-hx, -hy, -hz), xform(-hx, hy, -hz), xform(hx, hy, -hz), 5, side_tl),
        ];

        for (p0, p1, p2, p3, n_idx, tl) in faces {
            let base = vertices.len() as u32;
            vertices.extend_from_slice(&[
                BlockVertex { position: p0, normal_idx: n_idx, uv: [0.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: p1, normal_idx: n_idx, uv: [1.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: p2, normal_idx: n_idx, uv: [1.0, 0.0], tex_layer: tl, ao },
                BlockVertex { position: p3, normal_idx: n_idx, uv: [0.0, 0.0], tex_layer: tl, ao },
            ]);
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    fn append_basis_box(
        vertices: &mut Vec<BlockVertex>,
        indices: &mut Vec<u32>,
        origin: Vec3,
        ax: Vec3,
        ay: Vec3,
        az: Vec3,
        tl: u32,
        ao: f32,
    ) {
        let pt = |x: f32, y: f32, z: f32| -> [f32; 3] {
            (origin + ax * x + ay * y + az * z).to_array()
        };

        let faces: [([f32; 3], [f32; 3], [f32; 3], [f32; 3], u32); 6] = [
            (pt(1.0, -1.0, 1.0), pt(1.0, -1.0, -1.0), pt(1.0, 1.0, -1.0), pt(1.0, 1.0, 1.0), 0),
            (pt(-1.0, -1.0, -1.0), pt(-1.0, -1.0, 1.0), pt(-1.0, 1.0, 1.0), pt(-1.0, 1.0, -1.0), 1),
            (pt(-1.0, 1.0, 1.0), pt(1.0, 1.0, 1.0), pt(1.0, 1.0, -1.0), pt(-1.0, 1.0, -1.0), 2),
            (pt(-1.0, -1.0, -1.0), pt(1.0, -1.0, -1.0), pt(1.0, -1.0, 1.0), pt(-1.0, -1.0, 1.0), 3),
            (pt(-1.0, -1.0, 1.0), pt(1.0, -1.0, 1.0), pt(1.0, 1.0, 1.0), pt(-1.0, 1.0, 1.0), 4),
            (pt(1.0, -1.0, -1.0), pt(-1.0, -1.0, -1.0), pt(-1.0, 1.0, -1.0), pt(1.0, 1.0, -1.0), 5),
        ];

        for (p0, p1, p2, p3, n_idx) in faces {
            let base = vertices.len() as u32;
            vertices.extend_from_slice(&[
                BlockVertex { position: p0, normal_idx: n_idx, uv: [0.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: p1, normal_idx: n_idx, uv: [1.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: p2, normal_idx: n_idx, uv: [1.0, 0.0], tex_layer: tl, ao },
                BlockVertex { position: p3, normal_idx: n_idx, uv: [0.0, 0.0], tex_layer: tl, ao },
            ]);
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn append_basis_block(
        vertices: &mut Vec<BlockVertex>,
        indices: &mut Vec<u32>,
        origin: Vec3,
        ax: Vec3,
        ay: Vec3,
        az: Vec3,
        block: BlockType,
        ao: f32,
    ) {
        let pt = |x: f32, y: f32, z: f32| -> [f32; 3] {
            (origin + ax * x + ay * y + az * z).to_array()
        };

        let faces: [([f32; 3], [f32; 3], [f32; 3], [f32; 3], usize, u32); 6] = [
            (pt(1.0, -1.0, 1.0), pt(1.0, -1.0, -1.0), pt(1.0, 1.0, -1.0), pt(1.0, 1.0, 1.0), 0, 0),
            (pt(-1.0, -1.0, -1.0), pt(-1.0, -1.0, 1.0), pt(-1.0, 1.0, 1.0), pt(-1.0, 1.0, -1.0), 1, 1),
            (pt(-1.0, 1.0, 1.0), pt(1.0, 1.0, 1.0), pt(1.0, 1.0, -1.0), pt(-1.0, 1.0, -1.0), 2, 2),
            (pt(-1.0, -1.0, -1.0), pt(1.0, -1.0, -1.0), pt(1.0, -1.0, 1.0), pt(-1.0, -1.0, 1.0), 3, 3),
            (pt(-1.0, -1.0, 1.0), pt(1.0, -1.0, 1.0), pt(1.0, 1.0, 1.0), pt(-1.0, 1.0, 1.0), 4, 4),
            (pt(1.0, -1.0, -1.0), pt(-1.0, -1.0, -1.0), pt(-1.0, 1.0, -1.0), pt(1.0, 1.0, -1.0), 5, 5),
        ];

        for (p0, p1, p2, p3, face_idx, n_idx) in faces {
            let tl = block.texture_layer(face_idx);
            let base = vertices.len() as u32;
            vertices.extend_from_slice(&[
                BlockVertex { position: p0, normal_idx: n_idx, uv: [0.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: p1, normal_idx: n_idx, uv: [1.0, 1.0], tex_layer: tl, ao },
                BlockVertex { position: p2, normal_idx: n_idx, uv: [1.0, 0.0], tex_layer: tl, ao },
                BlockVertex { position: p3, normal_idx: n_idx, uv: [0.0, 0.0], tex_layer: tl, ao },
            ]);
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mob_melee_combat_and_drops() {
        let mut mobs = MobManager::new();
        let mut drops = DroppedItemManager::new();
        mobs.spawn_mob(MobKind::Pig, Vec3::new(0.0, 40.0, -2.5));
        assert_eq!(mobs.mobs.len(), 1);

        // Attack pig twice with a Diamond Sword (8.0 damage each -> kills 10 HP Pig)
        let origin = Vec3::new(0.0, 40.5, 0.0);
        let dir = Vec3::new(0.0, 0.0, -1.0);
        assert!(mobs.try_attack_mob(origin, dir, 4.2, 8.0, &mut drops));
        assert_eq!(mobs.mobs.len(), 1);

        // Clear hurt_timer so second swing hits immediately
        mobs.mobs[0].hurt_timer = 0.0;
        assert!(mobs.try_attack_mob(origin, dir, 4.2, 8.0, &mut drops));
        assert_eq!(mobs.mobs.len(), 0);
        assert!(!drops.items.is_empty());
        assert_eq!(drops.items[0].block, BlockType::RawPorkchop);
    }

    #[test]
    fn test_mob_and_extruded_tool_meshes_opaque() {
        let mut mobs = MobManager::new();
        mobs.spawn_mob(MobKind::Pig, Vec3::new(2.0, 40.0, 2.0));
        mobs.spawn_mob(MobKind::Zombie, Vec3::new(5.0, 40.0, 2.0));
        mobs.spawn_mob(MobKind::Creeper, Vec3::new(8.0, 40.0, 2.0));

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        mobs.append_mob_and_hand_meshes(
            &mut vertices,
            &mut indices,
            Vec3::new(0.0, 41.6, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            BlockType::StoneAxe,
            0.0,
            0.0,
        );

        assert!(!vertices.is_empty());
        assert_eq!(indices.len() % 3, 0);

        // Every vertex of Pig, Zombie, Creeper, and extruded StoneAxe must sample an opaque texel!
        for v in &vertices {
            let px = ((v.uv[0] * 16.0).floor() as i32).clamp(0, 15) as u32;
            let py = ((v.uv[1] * 16.0).floor() as i32).clamp(0, 15) as u32;
            let rgba = TextureManager::generate_pixel(v.tex_layer, px, py);
            assert_eq!(
                rgba[3], 255,
                "Vertex on layer {} at ({}, {}) had non-opaque alpha {}",
                v.tex_layer, px, py, rgba[3]
            );
        }
    }
}
