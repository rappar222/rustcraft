use crate::game::dropped_item::DroppedItemManager;
use crate::game::inventory::Inventory;
use crate::game::settings::GraphicsSettings;
use crate::game::state::{GameMode, GameState};
use crate::game::survival::SurvivalState;
use crate::player::controller::PlayerController;
use crate::render::renderer::{RenderError, Renderer};
use crate::ui::draw::UiBatch;
use crate::ui::menus::MenuRenderer;
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
    inventory: Inventory,
    survival: SurvivalState,
    dropped_items: DroppedItemManager,
    breaking_block: Option<(i32, i32, i32)>,
    break_progress: f32,
    is_lmb_down: bool,
    game_mode: GameMode,
    game_state: GameState,
    settings: GraphicsSettings,
    ui_batch: UiBatch,
    mouse_pos: (f32, f32),
    cursor_locked: bool,
    last_frame_time: Instant,
    last_fps_update: Instant,
    frame_count: u32,
    fps: f32,
    last_player_chunk: Option<ChunkPos>,
    world_generated: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            world: World::new(42),
            player: None,
            inventory: Inventory::new_survival(),
            survival: SurvivalState::new(),
            dropped_items: DroppedItemManager::new(),
            breaking_block: None,
            break_progress: 0.0,
            is_lmb_down: false,
            game_mode: GameMode::Survival,
            game_state: GameState::MainMenu,
            settings: GraphicsSettings::default(),
            ui_batch: UiBatch::new(),
            mouse_pos: (0.0, 0.0),
            cursor_locked: false,
            last_frame_time: Instant::now(),
            last_fps_update: Instant::now(),
            frame_count: 0,
            fps: 0.0,
            last_player_chunk: None,
            world_generated: false,
        }
    }

    fn lock_cursor(&mut self, lock: bool) {
        if let Some(window) = &self.window {
            self.cursor_locked = lock;
            window.set_cursor_visible(!lock);

            if lock {
                let grab_result = window.set_cursor_grab(CursorGrabMode::Locked);
                if grab_result.is_err() {
                    let _ = window.set_cursor_grab(CursorGrabMode::Confined);
                }
            } else {
                let _ = window.set_cursor_grab(CursorGrabMode::None);
            }
        }
    }

    fn get_safe_spawn_pos(&self) -> Vec3 {
        let spawn_x = 0;
        let spawn_z = 0;
        let ground_y = self.world.get_surface_height(spawn_x, spawn_z);
        Vec3::new(spawn_x as f32 + 0.5, ground_y as f32 + 1.001, spawn_z as f32 + 0.5)
    }

    fn ensure_world_generated(&mut self) {
        if !self.world_generated {
            println!("Generating initial world chunks for radius {}...", self.settings.render_distance);
            let spawn_chunk = ChunkPos::from_block_coords(0, 0);
            self.world.generate_chunks_around(spawn_chunk, self.settings.render_distance);
            self.last_player_chunk = Some(spawn_chunk);

            if let Some(renderer) = &mut self.renderer {
                renderer.update_dirty_chunks(&mut self.world);
            }
            self.world_generated = true;
        }
    }

    fn start_game(&mut self, mode: GameMode) {
        self.game_mode = mode;
        self.ensure_world_generated();

        // In Survival, player starts with an empty inventory; in Creative, with palette!
        self.inventory = match mode {
            GameMode::Survival => Inventory::new_survival(),
            GameMode::Creative => Inventory::new_creative(),
        };

        self.dropped_items.clear();
        self.breaking_block = None;
        self.break_progress = 0.0;
        self.is_lmb_down = false;

        let spawn_pos = self.get_safe_spawn_pos();

        if let Some(player) = &mut self.player {
            player.respawn(spawn_pos);
            player.is_flying = mode == GameMode::Creative;
            player.on_ground = true;
        }

        self.survival.reset(spawn_pos.y);
        self.game_state = GameState::Playing;
        self.lock_cursor(true);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("Rustcraft [Vulkan]")
            .with_inner_size(PhysicalSize::new(1280, 720));

        let window = Arc::new(event_loop.create_window(window_attributes).expect("Failed to create window"));
        self.window = Some(window.clone());

        println!("Initializing Vulkan renderer...");
        let renderer = pollster::block_on(Renderer::new(window.clone()));
        self.renderer = Some(renderer);

        let aspect = 1280.0 / 720.0;
        let spawn_pos = Vec3::new(0.0, 52.0, 0.0);
        let mut player = PlayerController::new(spawn_pos, aspect, self.game_mode);
        player.camera.set_fov_degrees(self.settings.fov_degrees);
        self.player = Some(player);

        // Start with cursor unlocked for Main Menu
        self.lock_cursor(false);
        self.last_frame_time = Instant::now();
        self.last_fps_update = Instant::now();
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
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = (position.x as f32, position.y as f32);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    let pressed = event.state == ElementState::Pressed;

                    match self.game_state {
                        GameState::MainMenu => {
                            if key_code == KeyCode::Escape && pressed {
                                event_loop.exit();
                            }
                        }
                        GameState::Playing => {
                            if pressed {
                                match key_code {
                                    KeyCode::Escape => {
                                        self.is_lmb_down = false;
                                        self.breaking_block = None;
                                        self.break_progress = 0.0;
                                        self.game_state = GameState::PauseMenu;
                                        self.lock_cursor(false);
                                        return;
                                    }
                                    KeyCode::KeyE => {
                                        self.is_lmb_down = false;
                                        self.breaking_block = None;
                                        self.break_progress = 0.0;
                                        self.game_state = GameState::Inventory;
                                        self.lock_cursor(false);
                                        return;
                                    }
                                    KeyCode::Digit1 => self.inventory.select_slot(0),
                                    KeyCode::Digit2 => self.inventory.select_slot(1),
                                    KeyCode::Digit3 => self.inventory.select_slot(2),
                                    KeyCode::Digit4 => self.inventory.select_slot(3),
                                    KeyCode::Digit5 => self.inventory.select_slot(4),
                                    KeyCode::Digit6 => self.inventory.select_slot(5),
                                    KeyCode::Digit7 => self.inventory.select_slot(6),
                                    KeyCode::Digit8 => self.inventory.select_slot(7),
                                    KeyCode::Digit9 => self.inventory.select_slot(8),
                                    _ => (),
                                }
                            }

                            if let Some(player) = &mut self.player {
                                player.handle_key(key_code, pressed, self.game_mode);
                            }
                        }
                        GameState::Inventory => {
                            if pressed && (key_code == KeyCode::KeyE || key_code == KeyCode::Escape) {
                                self.game_state = GameState::Playing;
                                self.lock_cursor(true);
                            }
                        }
                        GameState::PauseMenu => {
                            if pressed && key_code == KeyCode::Escape {
                                self.game_state = GameState::Playing;
                                self.lock_cursor(true);
                            }
                        }
                        GameState::Settings { from_main_menu } => {
                            if pressed && key_code == KeyCode::Escape {
                                self.game_state = if from_main_menu {
                                    GameState::MainMenu
                                } else {
                                    GameState::PauseMenu
                                };
                            }
                        }
                        GameState::GameOver => {
                            if pressed && key_code == KeyCode::Space {
                                let spawn_pos = self.get_safe_spawn_pos();
                                if let Some(player) = &mut self.player {
                                    player.respawn(spawn_pos);
                                }
                                self.survival.reset(spawn_pos.y);
                                self.game_state = GameState::Playing;
                                self.lock_cursor(true);
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let is_pressed = state == ElementState::Pressed;
                let (mx, my) = self.mouse_pos;

                match self.game_state {
                    GameState::MainMenu => {
                        if is_pressed {
                            let size = self.window.as_ref().map(|w| w.inner_size()).unwrap_or(PhysicalSize::new(1280, 720));
                            let mut dummy = UiBatch::new();
                            let buttons = MenuRenderer::render_main_menu(&mut dummy, size.width as f32, size.height as f32, (mx, my));

                            for (id, bx, by, bw, bh) in buttons {
                                if mx >= bx && mx <= bx + bw && my >= by && my <= by + bh {
                                    match id.as_str() {
                                        "play_survival" => self.start_game(GameMode::Survival),
                                        "play_creative" => self.start_game(GameMode::Creative),
                                        "settings" => self.game_state = GameState::Settings { from_main_menu: true },
                                        "quit" => event_loop.exit(),
                                        _ => (),
                                    }
                                    break;
                                }
                            }
                        }
                    }
                    GameState::Playing => {
                        if is_pressed {
                            if self.cursor_locked {
                                match button {
                                    MouseButton::Left => {
                                        self.is_lmb_down = true;
                                        if self.game_mode == GameMode::Creative {
                                            // Instant break in creative
                                            if let Some(player) = &mut self.player {
                                                player.handle_instant_break(&mut self.world);
                                            }
                                        }
                                    }
                                    MouseButton::Right => {
                                        // Place block
                                        let is_creative = self.game_mode == GameMode::Creative;
                                        if self.inventory.can_place_selected(is_creative) {
                                            let block = self.inventory.selected_block();
                                            if let Some(player) = &mut self.player {
                                                if player.try_place_block(&mut self.world, block) {
                                                    self.inventory.consume_selected(is_creative);
                                                }
                                            }
                                        }
                                    }
                                    _ => (),
                                }
                            } else if button == MouseButton::Left {
                                self.lock_cursor(true);
                            }
                        } else if button == MouseButton::Left {
                            self.is_lmb_down = false;
                            self.breaking_block = None;
                            self.break_progress = 0.0;
                        }
                    }
                    GameState::Inventory => {
                        if is_pressed {
                            let size = self.window.as_ref().map(|w| w.inner_size()).unwrap_or(PhysicalSize::new(1280, 720));
                            let mut dummy = UiBatch::new();
                            let (palette_hit, hotbar_hit) = MenuRenderer::render_inventory(
                                &mut dummy,
                                size.width as f32,
                                size.height as f32,
                                &self.inventory,
                                self.game_mode,
                                (mx, my),
                            );

                            if let Some(pal_idx) = palette_hit {
                                let block = Inventory::PALETTE[pal_idx];
                                if self.game_mode == GameMode::Creative {
                                    self.inventory.set_slot(self.inventory.selected_slot, block, 64);
                                }
                            } else if let Some(hb_idx) = hotbar_hit {
                                self.inventory.select_slot(hb_idx);
                            }
                        }
                    }
                    GameState::PauseMenu => {
                        if is_pressed {
                            let size = self.window.as_ref().map(|w| w.inner_size()).unwrap_or(PhysicalSize::new(1280, 720));
                            let mut dummy = UiBatch::new();
                            let buttons = MenuRenderer::render_pause_menu(
                                &mut dummy,
                                size.width as f32,
                                size.height as f32,
                                self.game_mode,
                                (mx, my),
                            );

                            for (id, bx, by, bw, bh) in buttons {
                                if mx >= bx && mx <= bx + bw && my >= by && my <= by + bh {
                                    match id.as_str() {
                                        "resume" => {
                                            self.game_state = GameState::Playing;
                                            self.lock_cursor(true);
                                        }
                                        "toggle_mode" => {
                                            self.game_mode = match self.game_mode {
                                                GameMode::Survival => GameMode::Creative,
                                                GameMode::Creative => GameMode::Survival,
                                            };
                                            if self.game_mode == GameMode::Creative {
                                                self.inventory = Inventory::new_creative();
                                            }
                                            if let Some(player) = &mut self.player {
                                                player.is_flying = self.game_mode == GameMode::Creative;
                                            }
                                        }
                                        "settings" => self.game_state = GameState::Settings { from_main_menu: false },
                                        "title" => {
                                            self.game_state = GameState::MainMenu;
                                            self.lock_cursor(false);
                                        }
                                        _ => (),
                                    }
                                    break;
                                }
                            }
                        }
                    }
                    GameState::Settings { from_main_menu } => {
                        if is_pressed {
                            let size = self.window.as_ref().map(|w| w.inner_size()).unwrap_or(PhysicalSize::new(1280, 720));
                            let mut dummy = UiBatch::new();
                            let buttons = MenuRenderer::render_settings_menu(
                                &mut dummy,
                                size.width as f32,
                                size.height as f32,
                                &self.settings,
                                (mx, my),
                            );

                            for (id, bx, by, bw, bh) in buttons {
                                if mx >= bx && mx <= bx + bw && my >= by && my <= by + bh {
                                    match id.as_str() {
                                        "cycle_rd" => {
                                            self.settings.render_distance = match self.settings.render_distance {
                                                4 => 8,
                                                8 => 12,
                                                12 => 16,
                                                _ => 4,
                                            };
                                            if self.world_generated {
                                                if let Some(player) = &self.player {
                                                    let cp = ChunkPos::from_block_coords(
                                                        player.camera.position.x as i32,
                                                        player.camera.position.z as i32,
                                                    );
                                                    self.world.generate_chunks_around(cp, self.settings.render_distance);
                                                }
                                            }
                                        }
                                        "cycle_fov" => {
                                            self.settings.fov_degrees = match self.settings.fov_degrees as i32 {
                                                60 => 75.0,
                                                75 => 90.0,
                                                90 => 105.0,
                                                _ => 60.0,
                                            };
                                            if let Some(player) = &mut self.player {
                                                player.camera.set_fov_degrees(self.settings.fov_degrees);
                                            }
                                        }
                                        "toggle_fog" => {
                                            self.settings.fog_enabled = !self.settings.fog_enabled;
                                        }
                                        "toggle_vsync" => {
                                            self.settings.vsync_enabled = !self.settings.vsync_enabled;
                                            if let Some(renderer) = &mut self.renderer {
                                                renderer.set_vsync(self.settings.vsync_enabled);
                                            }
                                        }
                                        "back" => {
                                            self.game_state = if from_main_menu {
                                                GameState::MainMenu
                                            } else {
                                                GameState::PauseMenu
                                            };
                                        }
                                        _ => (),
                                    }
                                    break;
                                }
                            }
                        }
                    }
                    GameState::GameOver => {
                        if is_pressed {
                            let size = self.window.as_ref().map(|w| w.inner_size()).unwrap_or(PhysicalSize::new(1280, 720));
                            let mut dummy = UiBatch::new();
                            let buttons = MenuRenderer::render_game_over(
                                &mut dummy,
                                size.width as f32,
                                size.height as f32,
                                (mx, my),
                            );

                            for (id, bx, by, bw, bh) in buttons {
                                if mx >= bx && mx <= bx + bw && my >= by && my <= by + bh {
                                    match id.as_str() {
                                        "respawn" => {
                                            let spawn_pos = self.get_safe_spawn_pos();
                                            if let Some(player) = &mut self.player {
                                                player.respawn(spawn_pos);
                                            }
                                            self.survival.reset(spawn_pos.y);
                                            self.game_state = GameState::Playing;
                                            self.lock_cursor(true);
                                        }
                                        "title" => {
                                            self.game_state = GameState::MainMenu;
                                            self.lock_cursor(false);
                                        }
                                        _ => (),
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last_frame_time).as_secs_f32().min(0.1);
                self.last_frame_time = now;

                // FPS Counter
                self.frame_count += 1;
                if now.duration_since(self.last_fps_update).as_secs_f32() >= 0.5 {
                    let elapsed = now.duration_since(self.last_fps_update).as_secs_f32();
                    self.fps = self.frame_count as f32 / elapsed;
                    self.frame_count = 0;
                    self.last_fps_update = now;
                }

                // Update game simulation if playing
                if self.game_state == GameState::Playing {
                    let mut should_game_over = false;
                    let mut current_chunk = None;

                    if let Some(player) = &mut self.player {
                        let is_dead = player.update(&self.world, dt, self.game_mode, &mut self.survival);
                        if is_dead {
                            should_game_over = true;
                        } else {
                            current_chunk = Some(ChunkPos::from_block_coords(
                                player.camera.position.x as i32,
                                player.camera.position.z as i32,
                            ));
                        }
                    }

                    if should_game_over {
                        self.game_state = GameState::GameOver;
                        self.is_lmb_down = false;
                        self.breaking_block = None;
                        self.break_progress = 0.0;
                        self.lock_cursor(false);
                    } else if let Some(chunk) = current_chunk {
                        if self.last_player_chunk != Some(chunk) {
                            self.world.generate_chunks_around(chunk, self.settings.render_distance);
                            self.last_player_chunk = Some(chunk);
                        }
                    }

                    // Update dropped items
                    let p_pos = self.player.as_ref().map(|p| p.camera.position).unwrap_or(Vec3::ZERO);
                    self.dropped_items.update(&self.world, p_pos, &mut self.inventory, dt);

                    // Survival block breaking loop
                    if self.is_lmb_down && self.game_mode == GameMode::Survival && self.cursor_locked {
                        if let Some(player) = &mut self.player {
                            let origin = player.camera.position;
                            let dir = player.camera.front();
                            if let Some(hit) = crate::player::raycast::raycast_voxels(origin, dir, 6.0, &self.world) {
                                let (bx, by, bz) = hit.block_pos;
                                let target_block = self.world.get_block(bx, by, bz);

                                if target_block != crate::voxel::block::BlockType::Air {
                                    if self.breaking_block == Some((bx, by, bz)) {
                                        let break_time = target_block.break_time();
                                        if break_time.is_finite() {
                                            self.break_progress += dt / break_time;
                                            if self.break_progress >= 1.0 {
                                                // Destroy block!
                                                self.world.set_block(bx, by, bz, crate::voxel::block::BlockType::Air);
                                                if let Some(drop) = target_block.drop_item() {
                                                    let drop_pos = Vec3::new(bx as f32 + 0.5, by as f32 + 0.5, bz as f32 + 0.5);
                                                    self.dropped_items.spawn(drop, drop_pos);
                                                }
                                                self.breaking_block = None;
                                                self.break_progress = 0.0;
                                            }
                                        }
                                    } else {
                                        self.breaking_block = Some((bx, by, bz));
                                        self.break_progress = 0.0;
                                    }
                                } else {
                                    self.breaking_block = None;
                                    self.break_progress = 0.0;
                                }
                            } else {
                                self.breaking_block = None;
                                self.break_progress = 0.0;
                            }
                        }
                    } else if self.game_mode == GameMode::Survival {
                        self.breaking_block = None;
                        self.break_progress = 0.0;
                    }
                }

                // Render Pass Preparation
                let (win_w, win_h) = if let Some(w) = &self.window {
                    let sz = w.inner_size();
                    (sz.width as f32, sz.height as f32)
                } else {
                    (1280.0, 720.0)
                };

                self.ui_batch.clear();

                // Build 2D UI elements according to current state
                match self.game_state {
                    GameState::MainMenu => {
                        MenuRenderer::render_main_menu(&mut self.ui_batch, win_w, win_h, self.mouse_pos);
                    }
                    GameState::Playing => {
                        if let Some(player) = &self.player {
                            MenuRenderer::render_hud(
                                &mut self.ui_batch,
                                win_w,
                                win_h,
                                &self.inventory,
                                self.game_mode,
                                &self.survival,
                                self.fps,
                                player.camera.position,
                                self.break_progress,
                                player.is_crouching,
                            );
                        }
                    }
                    GameState::Inventory => {
                        MenuRenderer::render_inventory(
                            &mut self.ui_batch,
                            win_w,
                            win_h,
                            &self.inventory,
                            self.game_mode,
                            self.mouse_pos,
                        );
                    }
                    GameState::PauseMenu => {
                        MenuRenderer::render_pause_menu(
                            &mut self.ui_batch,
                            win_w,
                            win_h,
                            self.game_mode,
                            self.mouse_pos,
                        );
                    }
                    GameState::Settings { .. } => {
                        MenuRenderer::render_settings_menu(
                            &mut self.ui_batch,
                            win_w,
                            win_h,
                            &self.settings,
                            self.mouse_pos,
                        );
                    }
                    GameState::GameOver => {
                        MenuRenderer::render_game_over(
                            &mut self.ui_batch,
                            win_w,
                            win_h,
                            self.mouse_pos,
                        );
                    }
                }

                if let Some(renderer) = &mut self.renderer {
                    if self.world_generated {
                        renderer.update_dirty_chunks(&mut self.world);
                    }

                    if let Some(player) = &self.player {
                        let render_3d = self.world_generated && self.game_state != GameState::MainMenu;
                        let view_dist_blocks = (self.settings.render_distance * 16) as f32;
                        let item_mesh = self.dropped_items.generate_mesh();

                        match renderer.render(
                            &player.camera,
                            view_dist_blocks,
                            self.settings.fog_enabled,
                            render_3d,
                            &item_mesh,
                            &self.ui_batch,
                        ) {
                            Ok(_) => {}
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
            if self.cursor_locked && self.game_state == GameState::Playing {
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
