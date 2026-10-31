use super::physics::Physics;
use super::raycast::raycast_voxels;
use crate::game::state::GameMode;
use crate::game::survival::SurvivalState;
use crate::render::camera::Camera;
use crate::voxel::block::BlockType;
use crate::voxel::world::World;
use glam::Vec3;
use winit::keyboard::KeyCode;

pub struct PlayerController {
    pub position: Vec3, // Exact feet position
    pub camera: Camera,
    pub velocity: Vec3,
    pub on_ground: bool,
    pub is_flying: bool,
    pub is_crouching: bool,
    pub eye_height: f32,

    // Input states
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sneak: bool,
    pub sprint: bool,
}

impl PlayerController {
    pub fn new(spawn_pos: Vec3, aspect: f32, initial_mode: GameMode) -> Self {
        let initial_eye = Physics::EYE_HEIGHT_STAND;
        Self {
            position: spawn_pos,
            camera: Camera::new(
                Vec3::new(spawn_pos.x, spawn_pos.y + initial_eye, spawn_pos.z),
                aspect,
            ),
            velocity: Vec3::ZERO,
            on_ground: true,
            is_flying: initial_mode == GameMode::Creative,
            is_crouching: false,
            eye_height: initial_eye,

            forward: false,
            backward: false,
            left: false,
            right: false,
            jump: false,
            sneak: false,
            sprint: false,
        }
    }

    pub fn respawn(&mut self, spawn_pos: Vec3) {
        self.position = spawn_pos;
        self.eye_height = Physics::EYE_HEIGHT_STAND;
        self.camera.position = Vec3::new(spawn_pos.x, spawn_pos.y + self.eye_height, spawn_pos.z);
        self.velocity = Vec3::ZERO;
        self.on_ground = true;
        self.is_flying = false;
        self.is_crouching = false;
        self.reset_inputs();
    }

    pub fn reset_inputs(&mut self) {
        self.forward = false;
        self.backward = false;
        self.left = false;
        self.right = false;
        self.jump = false;
        self.sneak = false;
        self.sprint = false;
    }

    pub fn feet_pos(&self) -> Vec3 {
        self.position
    }

    #[allow(dead_code)]
    pub fn center_pos(&self) -> Vec3 {
        Vec3::new(
            self.position.x,
            self.position.y + (self.eye_height * 0.5),
            self.position.z,
        )
    }

    pub fn handle_key(&mut self, key: KeyCode, pressed: bool, game_mode: GameMode) {
        match key {
            KeyCode::KeyW => self.forward = pressed,
            KeyCode::KeyS => self.backward = pressed,
            KeyCode::KeyA => self.left = pressed,
            KeyCode::KeyD => self.right = pressed,
            KeyCode::Space => self.jump = pressed,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => self.sneak = pressed,
            KeyCode::ControlLeft => self.sprint = pressed,
            KeyCode::KeyF if pressed && game_mode == GameMode::Creative => {
                self.is_flying = !self.is_flying;
            }
            _ => (),
        }
    }

    pub fn handle_mouse_move(&mut self, dx: f64, dy: f64) {
        let sensitivity = 0.0025;
        self.camera.yaw += (dx as f32) * sensitivity;
        self.camera.pitch -= (dy as f32) * sensitivity;

        let limit = 89.0_f32.to_radians();
        self.camera.pitch = self.camera.pitch.clamp(-limit, limit);
    }

    pub fn try_place_block(&mut self, world: &mut World, block_to_place: BlockType) -> bool {
        if block_to_place == BlockType::Air {
            return false;
        }

        let origin = self.camera.position;
        let direction = self.camera.front();

        if let Some(hit) = raycast_voxels(origin, direction, 6.0, world) {
            if hit.face_normal == (0, 0, 0) {
                return false;
            }
            let (ax, ay, az) = hit.adjacent_pos;
            let half_w = Physics::PLAYER_WIDTH * 0.5;
            let player_h = if self.is_crouching {
                Physics::PLAYER_HEIGHT_CROUCH
            } else {
                Physics::PLAYER_HEIGHT_STAND
            };

            let p_box = super::physics::Aabb::from_player_pos(self.position, half_w, player_h);
            let b_box = super::physics::Aabb::new(
                Vec3::new(ax as f32, ay as f32, az as f32),
                Vec3::new(ax as f32 + 1.0, ay as f32 + 1.0, az as f32 + 1.0),
            );

            if !p_box.intersects(&b_box) {
                return world.set_block(ax, ay, az, block_to_place);
            }
        }
        false
    }

    pub fn handle_instant_break(&mut self, world: &mut World) -> Option<(BlockType, Vec3)> {
        let origin = self.camera.position;
        let direction = self.camera.front();

        if let Some(hit) = raycast_voxels(origin, direction, 6.0, world) {
            let (bx, by, bz) = hit.block_pos;
            let old_block = world.get_block(bx, by, bz);
            if old_block != BlockType::Air {
                world.set_block(bx, by, bz, BlockType::Air);
                let drop_pos = Vec3::new(bx as f32 + 0.5, by as f32 + 0.5, bz as f32 + 0.5);
                return Some((old_block, drop_pos));
            }
        }
        None
    }

    pub fn update(
        &mut self,
        world: &World,
        dt: f32,
        game_mode: GameMode,
        survival: &mut SurvivalState,
    ) -> bool {
        if game_mode == GameMode::Survival {
            self.is_flying = false;
        }

        // Crouch when holding Shift (and not flying), OR keep crouched if under a low ceiling!
        if self.is_flying {
            self.is_crouching = false;
        } else if self.sneak {
            self.is_crouching = true;
        } else if self.is_crouching && Physics::can_stand_up(self.position, world) {
            self.is_crouching = false;
        }

        // Smooth camera eye height dip on crouch without disturbing feet position!
        let target_eye = if self.is_crouching {
            Physics::EYE_HEIGHT_CROUCH
        } else {
            Physics::EYE_HEIGHT_STAND
        };
        self.eye_height += (target_eye - self.eye_height) * (18.0 * dt).min(1.0);

        let mut move_dir = Vec3::ZERO;
        let front = Vec3::new(self.camera.yaw.cos(), 0.0, self.camera.yaw.sin()).normalize();
        let right = Vec3::new(-self.camera.yaw.sin(), 0.0, self.camera.yaw.cos()).normalize();

        if self.forward {
            move_dir += front;
        }
        if self.backward {
            move_dir -= front;
        }
        if self.right {
            move_dir += right;
        }
        if self.left {
            move_dir -= right;
        }

        let speed = if self.is_flying {
            if self.sprint { 36.0 } else { 16.0 }
        } else if self.is_crouching {
            1.95 // Authentic sneak speed
        } else if self.sprint {
            7.8
        } else {
            4.8
        };

        if move_dir.length_squared() > 0.001 {
            let n = move_dir.normalize();
            self.velocity.x = n.x * speed;
            self.velocity.z = n.z * speed;
        }

        if self.is_flying {
            if self.jump {
                self.velocity.y = speed * 0.75;
            } else if self.sneak {
                self.velocity.y = -speed * 0.75;
            } else {
                self.velocity.y = 0.0;
            }
        } else if self.jump && self.on_ground {
            self.velocity.y = 8.6;
            self.on_ground = false;
        }

        let was_on_ground = self.on_ground;
        self.on_ground = Physics::update_player(
            &mut self.position,
            &mut self.velocity,
            world,
            dt,
            self.is_flying,
            self.is_crouching,
            was_on_ground,
        );

        // Keep creative player from falling into extreme void
        if self.is_flying && self.position.y < -30.0 {
            self.position.y = -30.0;
            if self.velocity.y < 0.0 {
                self.velocity.y = 0.0;
            }
        }

        self.camera.position = Vec3::new(
            self.position.x,
            self.position.y + self.eye_height,
            self.position.z,
        );

        // Survival health & fall damage update
        let mut is_dead = false;
        if game_mode == GameMode::Survival {
            is_dead = survival.update(self.position, self.on_ground, dt);
        }

        is_dead
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel::chunk::ChunkPos;

    #[test]
    fn test_crouch_does_not_shift_feet_or_eject_player() {
        let mut world = World::new(42);
        world.generate_chunks_around(ChunkPos::new(0, 0), 1);
        let gy = world.get_surface_height(0, 0);
        let spawn = Vec3::new(0.5, gy as f32 + 1.0001, 0.5);

        let mut player = PlayerController::new(spawn, 1.77, GameMode::Survival);
        let mut survival = SurvivalState::new();
        survival.reset(spawn.y);

        // Toggle crouch on and off across 20 frames while standing still
        player.handle_key(KeyCode::ShiftLeft, true, GameMode::Survival);
        for _ in 0..10 {
            player.update(&world, 0.016, GameMode::Survival, &mut survival);
            assert!(player.on_ground, "Player should remain grounded while crouching");
            assert!((player.position.x - 0.5).abs() < 0.001);
            assert!((player.position.z - 0.5).abs() < 0.001);
        }
        assert!(player.eye_height < Physics::EYE_HEIGHT_STAND - 0.15);

        // Release crouch and ensure uncrouching does not push player into floor
        player.handle_key(KeyCode::ShiftLeft, false, GameMode::Survival);
        for _ in 0..10 {
            player.update(&world, 0.016, GameMode::Survival, &mut survival);
            assert!(player.on_ground, "Player should remain grounded while uncrouching");
            assert!((player.position.x - 0.5).abs() < 0.001);
            assert!((player.position.z - 0.5).abs() < 0.001);
        }
    }

    #[test]
    fn test_crouch_prevents_falling_off_single_block_pillar() {
        let mut world = World::new(42);
        world.generate_chunks_around(ChunkPos::new(0, 0), 1);

        // Place a single isolated block high in the air at (5, 100, 5)
        world.set_block(5, 100, 5, BlockType::Stone);
        let spawn = Vec3::new(5.5, 101.0001, 5.5);

        let mut player = PlayerController::new(spawn, 1.77, GameMode::Survival);
        let mut survival = SurvivalState::new();
        survival.reset(spawn.y);

        // Hold crouch and walk forward + right diagonally for 120 frames
        player.handle_key(KeyCode::ShiftLeft, true, GameMode::Survival);
        player.handle_key(KeyCode::KeyW, true, GameMode::Survival);
        player.handle_key(KeyCode::KeyD, true, GameMode::Survival);

        for _ in 0..120 {
            player.update(&world, 0.016, GameMode::Survival, &mut survival);
            assert!(
                player.position.y >= 101.0,
                "Player fell off the pillar while crouching! y = {}",
                player.position.y
            );
        }
    }
}
