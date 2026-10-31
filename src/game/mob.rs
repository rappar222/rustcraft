use crate::game::dropped_item::DroppedItemManager;
use crate::game::state::GameMode;
use crate::game::survival::SurvivalState;
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
                    let tl_pink = BlockType::RawPorkchop.texture_layer(0);
                    // Body
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.0, 0.45, 0.0], [0.42, 0.25, 0.28], tl_pink, ao);
                    // Head
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.45, 0.62, 0.0], [0.22, 0.22, 0.22], tl_pink, ao);
                    // 4 Animated Legs
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.25 + leg_swing, 0.15, 0.16], [0.10, 0.15, 0.10], tl_pink, ao * 0.85);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.25 - leg_swing, 0.15, -0.16], [0.10, 0.15, 0.10], tl_pink, ao * 0.85);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [-0.25 - leg_swing, 0.15, 0.16], [0.10, 0.15, 0.10], tl_pink, ao * 0.85);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [-0.25 + leg_swing, 0.15, -0.16], [0.10, 0.15, 0.10], tl_pink, ao * 0.85);
                }
                MobKind::Zombie => {
                    let tl_skin = BlockType::Cactus.texture_layer(2);
                    let tl_shirt = BlockType::Water.texture_layer(0);
                    let tl_pants = BlockType::Obsidian.texture_layer(0);
                    // Legs
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [leg_swing, 0.35, 0.12], [0.11, 0.35, 0.11], tl_pants, ao * 0.85);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [-leg_swing, 0.35, -0.12], [0.11, 0.35, 0.11], tl_pants, ao * 0.85);
                    // Torso
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.0, 1.02, 0.0], [0.14, 0.34, 0.24], tl_shirt, ao);
                    // Outstretched Zombie Arms
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.28, 1.18, 0.32], [0.28, 0.10, 0.10], tl_skin, ao);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.28, 1.18, -0.32], [0.28, 0.10, 0.10], tl_skin, ao);
                    // Head
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.0, 1.56, 0.0], [0.22, 0.22, 0.22], tl_skin, ao);
                }
                MobKind::Creeper => {
                    let tl_green = BlockType::Grass.texture_layer(2);
                    // 4 Short Creeper Feet
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.18 + leg_swing, 0.18, 0.14], [0.12, 0.18, 0.12], tl_green, ao * 0.85);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.18 - leg_swing, 0.18, -0.14], [0.12, 0.18, 0.12], tl_green, ao * 0.85);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [-0.18 - leg_swing, 0.18, 0.14], [0.12, 0.18, 0.12], tl_green, ao * 0.85);
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [-0.18 + leg_swing, 0.18, -0.14], [0.12, 0.18, 0.12], tl_green, ao * 0.85);
                    // Tall Torso
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.0, 0.82, 0.0], [0.16, 0.45, 0.20], tl_green, ao);
                    // Head
                    Self::append_oriented_part(vertices, indices, mob.position, mob.yaw, [0.0, 1.46, 0.0], [0.22, 0.22, 0.22], tl_green, ao);
                }
            }
        }

        // 2. Render 3D First-Person Right Hand / Held Block / Held Tool in bottom-right of view!
        let bob_x = (walk_bob * 0.5).cos() * 0.018;
        let bob_y = (walk_bob).sin().abs() * 0.022;
        let swing_angle = (swing_timer * std::f32::consts::PI).sin();
        let hand_pos = cam_pos
            + cam_front * (0.48 - swing_angle * 0.10)
            + cam_right * (0.32 + bob_x - swing_angle * 0.08)
            - cam_up * (0.26 - bob_y + swing_angle * 0.12);

        let yaw = cam_front.z.atan2(cam_front.x) + 0.4 + swing_angle * 0.6;
        if held_item == BlockType::Air {
            // Empty skin hand
            let tl_hand = BlockType::Sand.texture_layer(0);
            Self::append_oriented_part(vertices, indices, hand_pos, yaw, [0.0, 0.0, 0.0], [0.14, 0.07, 0.07], tl_hand, 1.0);
        } else if held_item.is_placeable() && held_item != BlockType::Torch {
            // Held 3D mini-block
            let s = 0.11;
            for face in 0..6 {
                let tl = held_item.texture_layer(face);
                Self::append_oriented_face(vertices, indices, hand_pos, yaw, s, face, tl, 1.05);
            }
        } else {
            // Held Tool / Sword / Torch / Food rod
            let tl = held_item.texture_layer(0);
            Self::append_oriented_part(vertices, indices, hand_pos, yaw, [0.0, 0.04, 0.0], [0.16, 0.16, 0.03], tl, 1.15);
        }
    }

    fn append_oriented_part(
        vertices: &mut Vec<BlockVertex>,
        indices: &mut Vec<u32>,
        origin: Vec3,
        yaw: f32,
        local_offset: [f32; 3],
        half_extents: [f32; 3],
        tl: u32,
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

        let faces: [([f32; 3], [f32; 3], [f32; 3], [f32; 3], u32); 6] = [
            (xform(hx, -hy, hz), xform(hx, -hy, -hz), xform(hx, hy, -hz), xform(hx, hy, hz), 0),
            (xform(-hx, -hy, -hz), xform(-hx, -hy, hz), xform(-hx, hy, hz), xform(-hx, hy, -hz), 1),
            (xform(-hx, hy, hz), xform(hx, hy, hz), xform(hx, hy, -hz), xform(-hx, hy, -hz), 2),
            (xform(-hx, -hy, -hz), xform(hx, -hy, -hz), xform(hx, -hy, hz), xform(-hx, -hy, hz), 3),
            (xform(-hx, -hy, hz), xform(hx, -hy, hz), xform(hx, hy, hz), xform(-hx, hy, hz), 4),
            (xform(hx, -hy, -hz), xform(-hx, -hy, -hz), xform(-hx, hy, -hz), xform(hx, hy, -hz), 5),
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

    fn append_oriented_face(
        vertices: &mut Vec<BlockVertex>,
        indices: &mut Vec<u32>,
        origin: Vec3,
        yaw: f32,
        s: f32,
        face: usize,
        tl: u32,
        ao: f32,
    ) {
        let cos_y = yaw.cos();
        let sin_y = yaw.sin();
        let xform = |lx: f32, ly: f32, lz: f32| -> [f32; 3] {
            [
                origin.x + lx * cos_y - lz * sin_y,
                origin.y + ly,
                origin.z + lx * sin_y + lz * cos_y,
            ]
        };

        let (p0, p1, p2, p3, n_idx) = match face {
            0 => (xform(s, -s, s), xform(s, -s, -s), xform(s, s, -s), xform(s, s, s), 0),
            1 => (xform(-s, -s, -s), xform(-s, -s, s), xform(-s, s, s), xform(-s, s, -s), 1),
            2 => (xform(-s, s, s), xform(s, s, s), xform(s, s, -s), xform(-s, s, -s), 2),
            3 => (xform(-s, -s, -s), xform(s, -s, -s), xform(s, -s, s), xform(-s, -s, s), 3),
            4 => (xform(-s, -s, s), xform(s, -s, s), xform(s, s, s), xform(-s, s, s), 4),
            _ => (xform(s, -s, -s), xform(-s, -s, -s), xform(-s, s, -s), xform(s, s, -s), 5),
        };

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
}
