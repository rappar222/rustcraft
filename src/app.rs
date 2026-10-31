use crate::player::controller::PlayerController;
use crate::render::renderer::{RenderError, Renderer};
use crate::voxel::chunk::ChunkPos;
use crate::voxel::world::World;
use glam::Vec3;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    world: World,
    player: Option<PlayerController>,
    cursor_locked: bool,
    last_frame_time: Instant,
    last_fps_update: Instant,
    frame_count: u32,
    last_player_chunk: Option<ChunkPos>,
    render_distance: i32,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            world: World::new(42),
            player: None,
            cursor_locked: false,
            last_frame_time: Instant::now(),
            last_fps_update: Instant::now(),
            frame_count: 0,
            last_player_chunk: None,
            render_distance: 8, // 8 chunks radius = 17x17 grid (289 chunks)
        }
    }

    fn toggle_cursor(&mut self) {
        if let Some(window) = &self.window {
            self.cursor_locked = !self.cursor_locked;
            window.set_cursor_visible(!self.cursor_locked);

            let grab_result = window.set_cursor_grab(CursorGrabMode::Locked);
            if grab_result.is_err() {
                let _ = window.set_cursor_grab(CursorGrabMode::Confined);
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Minecraft in Rust (Vulkan)")
            .with_inner_size(PhysicalSize::new(1280, 720));

        let window = Arc::new(event_loop.create_window(window_attributes).expect("Failed to create window"));
        self.window = Some(window.clone());

        println!("Initializing Vulkan renderer...");
        let renderer = pollster::block_on(Renderer::new(window.clone()));
        self.renderer = Some(renderer);

        let aspect = 1280.0 / 720.0;
        let spawn_pos = Vec3::new(0.0, 52.0, 0.0);
        self.player = Some(PlayerController::new(spawn_pos, aspect));

        // Initial chunk generation around spawn
        println!("Generating initial terrain chunks with Rayon thread pool...");
        let spawn_chunk = ChunkPos::from_block_coords(spawn_pos.x as i32, spawn_pos.z as i32);
        self.world.generate_chunks_around(spawn_chunk, self.render_distance);
        self.last_player_chunk = Some(spawn_chunk);

        if let Some(renderer) = &mut self.renderer {
            renderer.update_dirty_chunks(&mut self.world);
        }

        // Lock cursor
        self.toggle_cursor();
        self.last_frame_time = Instant::now();
        self.last_fps_update = Instant::now();
        println!("Engine started!");
        println!("Controls:");
        println!("  WASD: Move horizontal");
        println!("  Space: Jump (or fly up)");
        println!("  Left Shift: Sneak (or fly down)");
        println!("  Left Ctrl: Sprint");
        println!("  F: Toggle flying mode");
        println!("  1-9: Select block (Grass, Dirt, Stone, Cobble, Wood, Leaves, Sand, Planks, Glass)");
        println!("  Left Mouse Button: Break block");
        println!("  Right Mouse Button: Place block");
        println!("  Escape: Toggle cursor grab");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size);
                }
                if let Some(player) = &mut self.player {
                    if new_size.height > 0 {
                        player.camera.aspect = new_size.width as f32 / new_size.height as f32;
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    let pressed = event.state == ElementState::Pressed;
                    if key_code == KeyCode::Escape && pressed {
                        self.toggle_cursor();
                        return;
                    }

                    if let Some(player) = &mut self.player {
                        player.handle_key(key_code, pressed);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if state == ElementState::Pressed && self.cursor_locked {
                    if let Some(player) = &mut self.player {
                        player.handle_mouse_click(button, &mut self.world);
                    }
                } else if state == ElementState::Pressed && !self.cursor_locked && button == MouseButton::Left {
                    self.toggle_cursor();
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last_frame_time).as_secs_f32().min(0.1);
                self.last_frame_time = now;

                if let Some(player) = &mut self.player {
                    player.update(&self.world, dt);

                    // Check if player entered a new chunk
                    let current_chunk = ChunkPos::from_block_coords(
                        player.camera.position.x as i32,
                        player.camera.position.z as i32,
                    );

                    if self.last_player_chunk != Some(current_chunk) {
                        self.world.generate_chunks_around(current_chunk, self.render_distance);
                        self.last_player_chunk = Some(current_chunk);
                    }
                }

                if let Some(renderer) = &mut self.renderer {
                    // Update any dirty chunk meshes
                    renderer.update_dirty_chunks(&mut self.world);

                    // Render scene
                    if let Some(player) = &self.player {
                        match renderer.render(&player.camera) {
                            Ok(rendered_count) => {
                                self.frame_count += 1;
                                if now.duration_since(self.last_fps_update).as_secs_f32() >= 0.5 {
                                    let elapsed = now.duration_since(self.last_fps_update).as_secs_f32();
                                    let fps = self.frame_count as f32 / elapsed;
                                    let frame_time_ms = (elapsed / self.frame_count as f32) * 1000.0;
                                    self.frame_count = 0;
                                    self.last_fps_update = now;

                                    if let Some(window) = &self.window {
                                        let title = format!(
                                            "Minecraft Rust [Vulkan] | FPS: {:.0} ({:.2} ms) | Chunks: {}/{} | Pos: ({:.1}, {:.1}, {:.1}) | Block: {:?} | Fly: {} | [ESC: cursor]",
                                            fps,
                                            frame_time_ms,
                                            rendered_count,
                                            renderer.chunk_meshes.len(),
                                            player.camera.position.x,
                                            player.camera.position.y,
                                            player.camera.position.z,
                                            player.selected_block,
                                            if player.is_flying { "ON" } else { "OFF" }
                                        );
                                        window.set_title(&title);
                                    }
                                }
                            }
                            Err(RenderError::Lost | RenderError::Outdated) => {
                                renderer.resize(renderer.window.inner_size());
                            }
                            Err(RenderError::OutOfMemory) => {
                                event_loop.exit();
                            }
                            Err(_) => {}
                        }
                    }
                }
            }
            _ => (),
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.cursor_locked {
                if let Some(player) = &mut self.player {
                    player.handle_mouse_move(delta.0, delta.1);
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
