use crate::game::dropped_item::DroppedItemManager;
use crate::game::inventory::Inventory;
use crate::game::mob::MobManager;
use crate::game::save::WorldSaveData;
use crate::game::settings::GraphicsSettings;
use crate::game::state::{GameMode, GameState};
use crate::game::survival::SurvivalState;
use crate::player::controller::PlayerController;
use crate::render::renderer::{RenderError, Renderer};
use crate::ui::draw::UiBatch;
use crate::ui::menus::{InventoryAction, MenuRenderer};
use crate::voxel::block::BlockType;
use crate::voxel::chunk::ChunkPos;
use crate::voxel::world::World;
use glam::Vec3;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    world: World,
    player: Option<PlayerController>,
    inventory: Inventory,
    survival_inventory_backup: Option<Inventory>,
    survival: SurvivalState,
    dropped_items: DroppedItemManager,
    mob_manager: MobManager,
    targeted_block: Option<(i32, i32, i32)>,
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
    time_of_day: f32,
    status_message: Option<(String, f32)>,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            world: World::new(42),
            player: None,
            inventory: Inventory::new_survival(),
            survival_inventory_backup: None,
            survival: SurvivalState::new(),
            dropped_items: DroppedItemManager::new(),
            mob_manager: MobManager::new(),
            targeted_block: None,
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
            time_of_day: 0.36, // Bright morning
            status_message: None,
        }
    }

    fn show_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), 2.8));
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

    fn clear_active_gameplay_inputs(&mut self) {
        self.is_lmb_down = false;
        self.breaking_block = None;
        self.break_progress = 0.0;
        self.inventory.held_slot = None;
        if let Some(player) = &mut self.player {
            player.reset_inputs();
        }
    }

    fn get_safe_spawn_pos(&self) -> Vec3 {
        self.world.find_safe_spawn()
    }

    fn is_station_nearby(&self, block: BlockType) -> bool {
        if let Some(player) = &self.player {
            self.world.is_block_nearby(player.position, 5, block)
        } else {
            false
        }
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
        self.world = World::new(42);
        self.world_generated = false;
        self.game_mode = mode;
        self.time_of_day = 0.36;
        self.ensure_world_generated();

        self.inventory = match mode {
            GameMode::Survival => Inventory::new_survival(),
            GameMode::Creative => Inventory::new_creative(),
        };
        self.survival_inventory_backup = None;

        self.dropped_items.clear();
        self.mob_manager.clear();
        self.targeted_block = None;
        self.clear_active_gameplay_inputs();

        let spawn_pos = self.get_safe_spawn_pos();

        if let Some(player) = &mut self.player {
            player.respawn(spawn_pos);
            player.is_flying = mode == GameMode::Creative;
            player.on_ground = true;
        }

        self.survival.reset(spawn_pos.y);
        self.game_state = GameState::Playing;
        self.lock_cursor(true);
        self.show_status("Welcome to Rustcraft! Press [E] to Craft, [T] for Time, [F5] to Save");
    }

    fn save_current_world(&mut self) {
        if let Some(player) = &self.player {
            let ok = WorldSaveData::save(
                &self.world,
                self.game_mode,
                self.time_of_day,
                player.position,
                player.camera.yaw,
                player.camera.pitch,
                &self.survival,
                &self.inventory,
            )
            .is_ok();
            if ok {
                self.show_status("World Saved to rustcraft_save.dat!");
            } else {
                self.show_status("Failed to save world");
            }
        }
    }

    fn load_saved_world(&mut self) {
        if let Some(save) = WorldSaveData::load() {
            self.world = World::new(save.seed);
            self.game_mode = save.game_mode;
            self.time_of_day = save.time_of_day;

            let center_chunk = ChunkPos::from_block_coords(
                save.player_pos.x.floor() as i32,
                save.player_pos.z.floor() as i32,
            );
            self.world.generate_chunks_around(center_chunk, self.settings.render_distance);
            for ((x, y, z), block) in save.modified_blocks {
                self.world.set_block(x, y, z, block);
            }
            self.last_player_chunk = Some(center_chunk);
            self.world_generated = true;

            if let Some(renderer) = &mut self.renderer {
                renderer.update_dirty_chunks(&mut self.world);
            }

            self.inventory = save.inventory;
            self.survival_inventory_backup = None;

            self.survival.reset(save.player_pos.y);
            self.survival.health = save.health.clamp(1.0, self.survival.max_health);
            self.survival.hunger = save.hunger.clamp(0.0, self.survival.max_hunger);

            if let Some(player) = &mut self.player {
                player.respawn(save.player_pos);
                player.camera.yaw = save.yaw;
                player.camera.pitch = save.pitch;
                player.is_flying = self.game_mode == GameMode::Creative;
            }

            self.dropped_items.clear();
            self.mob_manager.clear();
            self.clear_active_gameplay_inputs();
            self.game_state = GameState::Playing;
            self.lock_cursor(true);
            self.show_status("Loaded Saved World!");
        } else {
            self.start_game(GameMode::Survival);
        }
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

        self.lock_cursor(false);
        self.last_frame_time = Instant::now();
        self.last_fps_update = Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                if self.world_generated && self.game_state != GameState::MainMenu {
                    self.save_current_world();
                }
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
            WindowEvent::MouseWheel { delta, .. } => {
                if self.game_state == GameState::Playing && self.cursor_locked {
                    let scroll_y = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y,
                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };
                    if scroll_y > 0.1 {
                        self.inventory.cycle_slot(-1);
                    } else if scroll_y < -0.1 {
                        self.inventory.cycle_slot(1);
                    }
                }
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
                                        self.clear_active_gameplay_inputs();
                                        self.game_state = GameState::PauseMenu;
                                        self.lock_cursor(false);
                                        return;
                                    }
                                    KeyCode::KeyE => {
                                        self.clear_active_gameplay_inputs();
                                        self.game_state = GameState::Inventory;
                                        self.lock_cursor(false);
                                        return;
                                    }
                                    KeyCode::KeyQ => {
                                        // Drop 1 item from selected slot
                                        let is_creative = self.game_mode == GameMode::Creative;
                                        let dropped = if is_creative {
                                            Some(self.inventory.selected_block())
                                                .filter(|&b| b != BlockType::Air)
                                        } else {
                                            self.inventory.drop_selected_one()
                                        };
                                        if let Some(dropped_block) = dropped {
                                            if let Some(player) = &mut self.player {
                                                player.trigger_swing();
                                                let eye = player.camera.position;
                                                let dir = player.camera.front();
                                                self.dropped_items.spawn_thrown(
                                                    dropped_block,
                                                    eye + dir * 0.65 - Vec3::new(0.0, 0.2, 0.0),
                                                    dir,
                                                );
                                            }
                                        }
                                    }
                                    KeyCode::KeyT => {
                                        self.time_of_day = (self.time_of_day + 0.22).rem_euclid(1.0);
                                        self.show_status("Time of Day Advanced ([T])");
                                    }
                                    KeyCode::F5 => {
                                        self.save_current_world();
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
                                self.inventory.held_slot = None;
                                self.game_state = GameState::Playing;
                                self.lock_cursor(true);
                            }
                        }
                        GameState::PauseMenu => {
                            if pressed && key_code == KeyCode::Escape {
                                self.game_state = GameState::Playing;
                                self.lock_cursor(true);
                            } else if pressed && key_code == KeyCode::F5 {
                                self.save_current_world();
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
                                        "load_saved" => self.load_saved_world(),
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
                                        if let Some(player) = &mut self.player {
                                            player.trigger_swing();
                                        }

                                        // Check melee attack on nearby Mobs first!
                                        let mut hit_mob = false;
                                        if let Some(player) = &self.player {
                                            let origin = player.camera.position;
                                            let dir = player.camera.front();
                                            let dmg = self.inventory.selected_block().attack_damage();
                                            hit_mob = self.mob_manager.try_attack_mob(
                                                origin,
                                                dir,
                                                4.2,
                                                dmg,
                                                &mut self.dropped_items,
                                            );
                                        }

                                        if !hit_mob {
                                            self.is_lmb_down = true;
                                            if self.game_mode == GameMode::Creative {
                                                if let Some((bx, by, bz)) = self.targeted_block {
                                                    if self.world.get_block(bx, by, bz) == BlockType::TNT {
                                                        let drops = self.world.explode(bx, by, bz, 3.0);
                                                        for (b, p) in drops {
                                                            self.dropped_items.spawn(b, p);
                                                        }
                                                        self.show_status("BOOM! TNT Detonated!");
                                                    } else if let Some(player) = &mut self.player {
                                                        player.handle_instant_break(&mut self.world);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    MouseButton::Right => {
                                        let is_creative = self.game_mode == GameMode::Creative;
                                        let selected_block = self.inventory.selected_block();
                                        let is_crouching = self.player.as_ref().map(|p| p.is_crouching).unwrap_or(false);

                                        // 1. Check if right-clicking an interactive block (CraftingTable, Furnace, or TNT)
                                        if !is_crouching {
                                            if let Some((bx, by, bz)) = self.targeted_block {
                                                let target = self.world.get_block(bx, by, bz);
                                                if target == BlockType::CraftingTable || target == BlockType::Furnace {
                                                    self.clear_active_gameplay_inputs();
                                                    self.game_state = GameState::Inventory;
                                                    self.lock_cursor(false);
                                                    return;
                                                } else if target == BlockType::TNT {
                                                    let drops = self.world.explode(bx, by, bz, 3.0);
                                                    for (b, p) in drops {
                                                        self.dropped_items.spawn(b, p);
                                                    }
                                                    if let Some(player) = &mut self.player {
                                                        player.trigger_swing();
                                                    }
                                                    self.show_status("BOOM! TNT Detonated!");
                                                    return;
                                                }
                                            }
                                        }

                                        // 2. Check if holding edible food
                                        if let Some(food_pts) = selected_block.food_restore() {
                                            if self.game_mode == GameMode::Survival {
                                                if self.survival.eat_food(food_pts) {
                                                    self.inventory.consume_selected(false);
                                                    if let Some(player) = &mut self.player {
                                                        player.trigger_swing();
                                                    }
                                                    self.show_status(format!(
                                                        "Ate {} (+{:.0} Hunger)",
                                                        selected_block.name(),
                                                        food_pts
                                                    ));
                                                } else {
                                                    self.show_status("Hunger & Health are already full!");
                                                }
                                            }
                                            return;
                                        }

                                        // 3. Otherwise place block if placeable
                                        if self.inventory.can_place_selected(is_creative) {
                                            if let Some(player) = &mut self.player {
                                                if player.try_place_block(&mut self.world, selected_block) {
                                                    self.inventory.consume_selected(is_creative);
                                                    player.trigger_swing();
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
                        if is_pressed && button == MouseButton::Left {
                            let size = self.window.as_ref().map(|w| w.inner_size()).unwrap_or(PhysicalSize::new(1280, 720));
                            let near_table = self.is_station_nearby(BlockType::CraftingTable);
                            let near_furnace = self.is_station_nearby(BlockType::Furnace);
                            let mut dummy = UiBatch::new();
                            let action = MenuRenderer::render_inventory(
                                &mut dummy,
                                size.width as f32,
                                size.height as f32,
                                &self.inventory,
                                self.game_mode,
                                near_table,
                                near_furnace,
                                (mx, my),
                            );

                            match action {
                                Some(InventoryAction::SelectPalette(pal_idx)) => {
                                    let block = Inventory::PALETTE[pal_idx];
                                    if self.game_mode == GameMode::Creative {
                                        self.inventory.set_slot(
                                            self.inventory.selected_slot,
                                            block,
                                            block.max_stack_size(),
                                        );
                                    }
                                }
                                Some(InventoryAction::CraftRecipe(recipe_idx)) => {
                                    if self.game_mode == GameMode::Survival {
                                        if self.inventory.craft(recipe_idx, near_table, near_furnace) {
                                            let out_name = Inventory::RECIPES[recipe_idx].output.name();
                                            self.show_status(format!("Crafted {}!", out_name));
                                        }
                                    }
                                }
                                Some(InventoryAction::ClickCombinedSlot(slot_idx)) => {
                                    if self.game_mode == GameMode::Creative {
                                        if slot_idx < 9 {
                                            self.inventory.select_slot(slot_idx);
                                        }
                                    } else if let Some(prev_slot) = self.inventory.held_slot {
                                        if prev_slot != slot_idx {
                                            self.inventory.interact_combined_slots(prev_slot, slot_idx);
                                        }
                                        self.inventory.held_slot = None;
                                        if slot_idx < 9 {
                                            self.inventory.select_slot(slot_idx);
                                        }
                                    } else {
                                        if !self.inventory.get_combined_slot(slot_idx).is_empty() {
                                            self.inventory.held_slot = Some(slot_idx);
                                        }
                                        if slot_idx < 9 {
                                            self.inventory.select_slot(slot_idx);
                                        }
                                    }
                                }
                                None => {
                                    self.inventory.held_slot = None;
                                }
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
                                        "save" => {
                                            self.save_current_world();
                                            self.game_state = GameState::Playing;
                                            self.lock_cursor(true);
                                        }
                                        "toggle_mode" => {
                                            match self.game_mode {
                                                GameMode::Survival => {
                                                    self.survival_inventory_backup = Some(self.inventory.clone());
                                                    self.game_mode = GameMode::Creative;
                                                    self.inventory = Inventory::new_creative();
                                                }
                                                GameMode::Creative => {
                                                    self.game_mode = GameMode::Survival;
                                                    self.inventory = self
                                                        .survival_inventory_backup
                                                        .take()
                                                        .unwrap_or_else(Inventory::new_survival);
                                                    if let Some(player) = &self.player {
                                                        self.survival.highest_fall_y = player.feet_pos().y;
                                                    }
                                                }
                                            }
                                            if let Some(player) = &mut self.player {
                                                player.is_flying = self.game_mode == GameMode::Creative;
                                            }
                                        }
                                        "settings" => self.game_state = GameState::Settings { from_main_menu: false },
                                        "title" => {
                                            self.save_current_world();
                                            self.clear_active_gameplay_inputs();
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
                                                        player.position.x as i32,
                                                        player.position.z as i32,
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
                                            self.clear_active_gameplay_inputs();
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

                // Update status banner timer
                if let Some((_, timer)) = &mut self.status_message {
                    *timer -= dt;
                    if *timer <= 0.0 {
                        self.status_message = None;
                    }
                }

                // Update game simulation if playing
                if self.game_state == GameState::Playing {
                    // Advance Day/Night cycle (1 full day = 480 seconds = 8 minutes)
                    self.time_of_day = (self.time_of_day + dt / 480.0).rem_euclid(1.0);

                    let mut should_game_over = false;
                    let mut current_chunk = None;

                    if let Some(player) = &mut self.player {
                        let is_dead = player.update(&self.world, dt, self.game_mode, &mut self.survival);
                        if is_dead {
                            should_game_over = true;
                        } else {
                            current_chunk = Some(ChunkPos::from_block_coords(
                                player.position.x as i32,
                                player.position.z as i32,
                            ));
                        }
                    }

                    // Update Mobs (spawning, AI movement, Creeper explosions, Zombie combat)
                    let p_pos = self.player.as_ref().map(|p| p.feet_pos()).unwrap_or(Vec3::ZERO);
                    let (_, daylight) =
                        crate::render::camera::Camera::sky_atmosphere(self.time_of_day, false);
                    if let Some(player) = &mut self.player {
                        self.mob_manager.update(
                            &mut self.world,
                            p_pos,
                            &mut player.velocity,
                            self.game_mode,
                            &mut self.survival,
                            &mut self.dropped_items,
                            daylight,
                            dt,
                        );
                    }
                    if self.survival.is_dead() && self.game_mode == GameMode::Survival {
                        should_game_over = true;
                    }

                    if should_game_over {
                        self.clear_active_gameplay_inputs();
                        self.game_state = GameState::GameOver;
                        self.lock_cursor(false);
                    } else if let Some(chunk) = current_chunk {
                        if self.last_player_chunk != Some(chunk) {
                            self.world.generate_chunks_around(chunk, self.settings.render_distance);
                            self.last_player_chunk = Some(chunk);
                        }
                    }

                    // Update dropped items
                    self.dropped_items.update(&self.world, p_pos, &mut self.inventory, dt);

                    // Raycast targeted block & survival block breaking loop
                    self.targeted_block = None;
                    if let Some(player) = &self.player {
                        let origin = player.camera.position;
                        let dir = player.camera.front();
                        if let Some(hit) = crate::player::raycast::raycast_voxels(origin, dir, 6.0, &self.world) {
                            self.targeted_block = Some(hit.block_pos);
                        }
                    }

                    if self.is_lmb_down && self.game_mode == GameMode::Survival && self.cursor_locked {
                        if let Some(player) = &mut self.player {
                            if player.swing_timer <= 0.0 {
                                player.trigger_swing();
                            }
                        }

                        if let Some((bx, by, bz)) = self.targeted_block {
                            let target_block = self.world.get_block(bx, by, bz);

                            if target_block.is_targetable() {
                                if self.breaking_block == Some((bx, by, bz)) {
                                    let base_time = target_block.break_time();
                                    let tool_mult = self
                                        .inventory
                                        .selected_block()
                                        .tool_speed_multiplier(target_block);
                                    let effective_time = base_time / tool_mult;

                                    if effective_time.is_finite() && effective_time > 0.0 {
                                        self.break_progress += dt / effective_time;
                                        if self.break_progress >= 1.0 {
                                            if target_block == BlockType::TNT {
                                                let drops = self.world.explode(bx, by, bz, 3.0);
                                                for (b, p) in drops {
                                                    self.dropped_items.spawn(b, p);
                                                }
                                                self.show_status("BOOM! TNT Detonated!");
                                            } else {
                                                self.world.set_block(bx, by, bz, BlockType::Air);
                                                let drop_pos = Vec3::new(
                                                    bx as f32 + 0.5,
                                                    by as f32 + 0.5,
                                                    bz as f32 + 0.5,
                                                );
                                                if let Some(drop) = target_block.drop_item() {
                                                    self.dropped_items.spawn(drop, drop_pos);
                                                }
                                                // Breaking Oak Leaves has a 25% chance to also drop a fresh Apple!
                                                if target_block == BlockType::Leaves
                                                    && ((bx.wrapping_mul(31) ^ by ^ bz.wrapping_mul(17)).unsigned_abs() % 4 == 0)
                                                {
                                                    self.dropped_items.spawn(BlockType::Apple, drop_pos);
                                                }
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
                            let biome_name = self
                                .world
                                .biome_at(player.position.x.floor() as i32, player.position.z.floor() as i32)
                                .name();
                            let status_str = self.status_message.as_ref().map(|(s, _)| s.as_str());
                            MenuRenderer::render_hud(
                                &mut self.ui_batch,
                                win_w,
                                win_h,
                                &self.inventory,
                                self.game_mode,
                                &self.survival,
                                self.fps,
                                player.position,
                                self.break_progress,
                                player.is_crouching,
                                player.head_underwater,
                                biome_name,
                                self.time_of_day,
                                self.mob_manager.mobs.len(),
                                status_str,
                            );
                        }
                    }
                    GameState::Inventory => {
                        let near_table = self.is_station_nearby(BlockType::CraftingTable);
                        let near_furnace = self.is_station_nearby(BlockType::Furnace);
                        MenuRenderer::render_inventory(
                            &mut self.ui_batch,
                            win_w,
                            win_h,
                            &self.inventory,
                            self.game_mode,
                            near_table,
                            near_furnace,
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
                        let highlight_target = if self.game_state == GameState::Playing {
                            self.targeted_block
                        } else {
                            None
                        };
                        let mut item_mesh = self
                            .dropped_items
                            .generate_mesh_with_highlight(highlight_target, self.break_progress);

                        if render_3d {
                            self.mob_manager.append_mob_and_hand_meshes(
                                &mut item_mesh.0,
                                &mut item_mesh.1,
                                player.camera.position,
                                player.camera.front(),
                                player.camera.right(),
                                player.camera.up(),
                                self.inventory.selected_block(),
                                player.swing_timer,
                                player.walk_bob,
                            );
                        }

                        match renderer.render(
                            &player.camera,
                            view_dist_blocks,
                            self.settings.fog_enabled,
                            self.time_of_day,
                            player.head_underwater,
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
