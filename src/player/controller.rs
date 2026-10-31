use super::physics::Physics;
use super::raycast::raycast_voxels;
use crate::game::state::GameMode;
use crate::game::survival::SurvivalState;
use crate::render::camera::Camera;
use crate::voxel::block::BlockType;
use crate::voxel::world::World;
use glam::Vec3;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

pub struct PlayerController {
    pub camera: Camera,
    pub velocity: Vec3,
    pub on_ground: bool,
    pub is_flying: bool,

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
        Self {
            camera: Camera::new(
                Vec3::new(spawn_pos.x, spawn_pos.y + Physics::EYE_HEIGHT, spawn_pos.z),
                aspect,
            ),
            velocity: Vec3::ZERO,
            on_ground: false,
            is_flying: initial_mode == GameMode::Creative,

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
        self.camera.position = Vec3::new(spawn_pos.x, spawn_pos.y + Physics::EYE_HEIGHT, spawn_pos.z);
        self.velocity = Vec3::ZERO;
        self.on_ground = true;
        self.is_flying = false;
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

    pub fn handle_mouse_click(&mut self, button: MouseButton, world: &mut World, block_to_place: BlockType) {
        let origin = self.camera.position;
        let direction = self.camera.front();

        if let Some(hit) = raycast_voxels(origin, direction, 7.0, world) {
            match button {
                MouseButton::Left => {
                    let (bx, by, bz) = hit.block_pos;
                    world.set_block(bx, by, bz, BlockType::Air);
                }
                MouseButton::Right => {
                    let (ax, ay, az) = hit.adjacent_pos;
                    let p_pos = Vec3::new(
                        self.camera.position.x,
                        self.camera.position.y - Physics::EYE_HEIGHT,
                        self.camera.position.z,
                    );
                    let p_box = super::physics::Aabb::from_player_pos(
                        p_pos,
                        Physics::PLAYER_WIDTH * 0.5,
                        Physics::PLAYER_HEIGHT,
                    );
                    let b_box = super::physics::Aabb::new(
                        Vec3::new(ax as f32, ay as f32, az as f32),
                        Vec3::new(ax as f32 + 1.0, ay as f32 + 1.0, az as f32 + 1.0),
                    );

                    if !p_box.intersects(&b_box) {
                        world.set_block(ax, ay, az, block_to_place);
                    }
                }
                _ => (),
            }
        }
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
            if self.sprint { 40.0 } else { 18.0 }
        } else {
            if self.sprint { 8.5 } else { 5.0 }
        };

        if move_dir.length_squared() > 0.001 {
            let n = move_dir.normalize();
            self.velocity.x = n.x * speed;
            self.velocity.z = n.z * speed;
        }

        if self.is_flying {
            if self.jump {
                self.velocity.y = speed;
            } else if self.sneak {
                self.velocity.y = -speed;
            } else {
                self.velocity.y = 0.0;
            }
        } else {
            if self.jump && self.on_ground {
                self.velocity.y = 9.0;
                self.on_ground = false;
            }
        }

        let mut feet_pos = Vec3::new(
            self.camera.position.x,
            self.camera.position.y - Physics::EYE_HEIGHT,
            self.camera.position.z,
        );

        self.on_ground = Physics::update_player(
            &mut feet_pos,
            &mut self.velocity,
            world,
            dt,
            self.is_flying,
        );

        self.camera.position = Vec3::new(
            feet_pos.x,
            feet_pos.y + Physics::EYE_HEIGHT,
            feet_pos.z,
        );

        // Survival health & fall damage update
        let mut is_dead = false;
        if game_mode == GameMode::Survival {
            is_dead = survival.update(feet_pos, self.on_ground, dt);
        }

        is_dead
    }
}
