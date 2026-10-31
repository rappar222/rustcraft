use super::draw::UiBatch;
use crate::game::inventory::Inventory;
use crate::game::settings::GraphicsSettings;
use crate::game::state::GameMode;
use crate::game::survival::SurvivalState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryAction {
    SelectPalette(usize),
    ClickCombinedSlot(usize), // 0..9 = hotbar, 9..36 = storage
    CraftRecipe(usize),
}

pub struct MenuRenderer;

impl MenuRenderer {
    pub fn render_main_menu(
        batch: &mut UiBatch,
        screen_w: f32,
        screen_h: f32,
        mouse_pos: (f32, f32),
    ) -> Vec<(String, f32, f32, f32, f32)> {
        // Background
        batch.draw_rect(0.0, 0.0, screen_w, screen_h, [0.12, 0.14, 0.18, 1.0]);

        // Header bar
        batch.draw_rect(0.0, 0.0, screen_w, 140.0, [0.08, 0.09, 0.12, 1.0]);
        batch.draw_rect(0.0, 140.0, screen_w, 4.0, [0.25, 0.35, 0.50, 1.0]);

        // Main Title: RUSTCRAFT
        let title = "RUSTCRAFT";
        let title_scale = 5.0;
        let title_w = title.len() as f32 * 8.0 * title_scale;
        let title_x = (screen_w - title_w) * 0.5;
        let title_y = 45.0;

        // Shadow & Title
        batch.draw_text(title, title_x + 4.0, title_y + 4.0, title_scale, [0.0, 0.0, 0.0, 0.9]);
        batch.draw_text(title, title_x, title_y, title_scale, [0.95, 0.85, 0.20, 1.0]);

        let subtitle = "Vulkan Edition (Maximum Speed)";
        let sub_scale = 2.0;
        let sub_w = subtitle.len() as f32 * 8.0 * sub_scale;
        batch.draw_text(subtitle, (screen_w - sub_w) * 0.5, 105.0, sub_scale, [0.6, 0.8, 1.0, 0.9]);

        // Menu Buttons
        let btn_w = 340.0;
        let btn_h = 48.0;
        let btn_x = (screen_w - btn_w) * 0.5;
        let start_y = 200.0;
        let spacing = 64.0;

        let buttons = [
            ("play_survival", "Play World (Survival)"),
            ("play_creative", "Play World (Creative)"),
            ("settings", "Graphics Settings"),
            ("quit", "Quit Game"),
        ];

        let mut hitboxes = Vec::new();

        for (i, (id, label)) in buttons.iter().enumerate() {
            let y = start_y + (i as f32) * spacing;
            let is_hovered = mouse_pos.0 >= btn_x
                && mouse_pos.0 <= btn_x + btn_w
                && mouse_pos.1 >= y
                && mouse_pos.1 <= y + btn_h;

            batch.draw_button(btn_x, y, btn_w, btn_h, label, is_hovered);
            hitboxes.push((id.to_string(), btn_x, y, btn_w, btn_h));
        }

        // Footer version info
        let footer = "Built with Rust 2024 & Vulkan (wgpu)";
        batch.draw_text(footer, 16.0, screen_h - 28.0, 1.5, [0.5, 0.5, 0.5, 0.8]);

        hitboxes
    }

    pub fn render_hud(
        batch: &mut UiBatch,
        screen_w: f32,
        screen_h: f32,
        inventory: &Inventory,
        game_mode: GameMode,
        survival: &SurvivalState,
        fps: f32,
        pos: glam::Vec3,
        break_progress: f32,
        is_crouching: bool,
    ) {
        // Crosshair at screen center
        batch.draw_crosshair(screen_w, screen_h);

        // Block breaking progress bar below crosshair
        if break_progress > 0.0 {
            let bar_w = 56.0;
            let bar_h = 8.0;
            let bar_x = (screen_w - bar_w) * 0.5;
            let bar_y = screen_h * 0.5 + 16.0;

            batch.draw_rect(bar_x - 2.0, bar_y - 2.0, bar_w + 4.0, bar_h + 4.0, [0.0, 0.0, 0.0, 0.85]);
            let fill_color = [1.0 - break_progress * 0.5, 0.25 + break_progress * 0.75, 0.15, 0.98];
            batch.draw_rect(bar_x, bar_y, bar_w * break_progress.clamp(0.0, 1.0), bar_h, fill_color);
        }

        // Top-left HUD info
        let mode_str = match game_mode {
            GameMode::Survival => {
                if is_crouching {
                    "Mode: SURVIVAL [SNEAKING]"
                } else {
                    "Mode: SURVIVAL"
                }
            }
            GameMode::Creative => "Mode: CREATIVE (Fly: [F])",
        };
        batch.draw_text(mode_str, 16.0, 16.0, 2.0, [1.0, 1.0, 1.0, 0.9]);

        let pos_str = format!("XYZ: {:.1} / {:.1} / {:.1}", pos.x, pos.y, pos.z);
        batch.draw_text(&pos_str, 16.0, 38.0, 1.5, [0.8, 0.8, 0.8, 0.8]);

        let fps_str = format!("FPS: {:.0}", fps);
        batch.draw_text(&fps_str, 16.0, 56.0, 1.5, [0.5, 1.0, 0.5, 0.8]);

        let controls_str = "[E] Inventory / Craft   [Shift] Crouch   [Scroll/1-9] Slot";
        batch.draw_text(controls_str, 16.0, 74.0, 1.2, [0.7, 0.75, 0.85, 0.75]);

        // Survival Hearts HUD (if in Survival mode)
        if game_mode == GameMode::Survival {
            let heart_size = 14.0;
            let heart_spacing = 16.0;
            let start_x = screen_w * 0.5 - 196.0;
            let start_y = screen_h - 75.0;

            let total_hearts = 10;
            for i in 0..total_hearts {
                let x = start_x + (i as f32) * heart_spacing;
                let heart_value = (i + 1) * 2;
                let is_empty = survival.health < (heart_value - 1) as f32;
                let is_half = survival.health >= (heart_value - 1) as f32 && survival.health < heart_value as f32;
                batch.draw_heart(x, start_y, heart_size, is_half, is_empty);
            }
        }

        // 9-slot Hotbar at bottom center
        let slot_size = 40.0;
        let slot_spacing = 44.0;
        let total_w = 9.0 * slot_spacing;
        let hotbar_x = (screen_w - total_w) * 0.5;
        let hotbar_y = screen_h - 52.0;

        for i in 0..9 {
            let sx = hotbar_x + (i as f32) * slot_spacing;
            let is_selected = i == inventory.selected_slot;

            let bg_color = if is_selected {
                [0.45, 0.50, 0.65, 0.9]
            } else {
                [0.15, 0.15, 0.18, 0.8]
            };
            batch.draw_rect(sx, hotbar_y, slot_size, slot_size, bg_color);

            let border_color = if is_selected {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.35, 0.35, 0.40, 0.8]
            };
            let thick = if is_selected { 3.0 } else { 1.5 };
            batch.draw_rect_outline(sx, hotbar_y, slot_size, slot_size, thick, border_color);

            let stack = inventory.hotbar[i];
            if !stack.is_empty() {
                let layer = stack.block.texture_layer(2);
                batch.draw_block_icon(sx + 4.0, hotbar_y + 4.0, slot_size - 8.0, layer);

                if game_mode == GameMode::Survival {
                    let count_str = stack.count.to_string();
                    let cnt_w = count_str.len() as f32 * 8.0;
                    let cx = sx + slot_size - cnt_w - 2.0;
                    let cy = hotbar_y + slot_size - 10.0;
                    batch.draw_text(&count_str, cx + 1.0, cy + 1.0, 1.0, [0.0, 0.0, 0.0, 0.9]);
                    batch.draw_text(&count_str, cx, cy, 1.0, [1.0, 1.0, 1.0, 1.0]);
                }
            }

            let num_str = (i + 1).to_string();
            batch.draw_text(&num_str, sx + 3.0, hotbar_y + 2.0, 1.0, [1.0, 1.0, 1.0, 0.6]);
        }

        // Active block name above hotbar
        let sel_stack = inventory.selected_stack();
        let sel_name = if sel_stack.is_empty() {
            "Empty Hand".to_string()
        } else if game_mode == GameMode::Survival {
            format!("{:?} ({})", sel_stack.block, sel_stack.count)
        } else {
            format!("{:?}", sel_stack.block)
        };

        let sel_scale = 1.5;
        let name_w = sel_name.len() as f32 * 8.0 * sel_scale;
        let name_x = (screen_w - name_w) * 0.5;
        let name_y = hotbar_y - 22.0;
        batch.draw_text(&sel_name, name_x, name_y, sel_scale, [1.0, 1.0, 0.6, 0.9]);
    }

    pub fn render_inventory(
        batch: &mut UiBatch,
        screen_w: f32,
        screen_h: f32,
        inventory: &Inventory,
        game_mode: GameMode,
        mouse_pos: (f32, f32),
    ) -> Option<InventoryAction> {
        // Dark translucent overlay
        batch.draw_rect(0.0, 0.0, screen_w, screen_h, [0.0, 0.0, 0.0, 0.75]);

        let win_w = 560.0;
        let win_h = 460.0;
        let win_x = (screen_w - win_w) * 0.5;
        let win_y = (screen_h - win_h) * 0.5;

        // Window background with Minecraft border
        batch.draw_rect(win_x, win_y, win_w, win_h, [0.20, 0.21, 0.24, 0.98]);
        batch.draw_rect_outline(win_x, win_y, win_w, win_h, 3.0, [0.50, 0.52, 0.58, 1.0]);

        let mut action = None;

        if game_mode == GameMode::Creative {
            let title = "CREATIVE INVENTORY & PALETTE";
            let title_w = title.len() as f32 * 8.0 * 2.0;
            batch.draw_text(title, win_x + (win_w - title_w) * 0.5, win_y + 18.0, 2.0, [1.0, 1.0, 0.6, 1.0]);

            let help = "Click any block in palette to assign to selected hotbar slot";
            let help_w = help.len() as f32 * 8.0 * 1.2;
            batch.draw_text(help, win_x + (win_w - help_w) * 0.5, win_y + 44.0, 1.2, [0.75, 0.75, 0.75, 0.9]);

            // Palette grid (2 rows of 5 blocks)
            let slot_size = 56.0;
            let spacing = 68.0;
            let start_x = win_x + (win_w - 5.0 * spacing) * 0.5 + 6.0;
            let start_y = win_y + 84.0;

            for (idx, &block) in Inventory::PALETTE.iter().enumerate() {
                let row = idx / 5;
                let col = idx % 5;
                let sx = start_x + (col as f32) * spacing;
                let sy = start_y + (row as f32) * spacing;

                let is_hover = mouse_pos.0 >= sx
                    && mouse_pos.0 <= sx + slot_size
                    && mouse_pos.1 >= sy
                    && mouse_pos.1 <= sy + slot_size;

                if is_hover {
                    action = Some(InventoryAction::SelectPalette(idx));
                }

                let bg = if is_hover {
                    [0.45, 0.55, 0.75, 1.0]
                } else {
                    [0.15, 0.15, 0.18, 1.0]
                };
                batch.draw_rect(sx, sy, slot_size, slot_size, bg);
                let border = if is_hover {
                    [1.0, 1.0, 0.5, 1.0]
                } else {
                    [0.35, 0.35, 0.40, 1.0]
                };
                batch.draw_rect_outline(sx, sy, slot_size, slot_size, 2.0, border);

                let layer = block.texture_layer(2);
                batch.draw_block_icon(sx + 6.0, sy + 6.0, slot_size - 12.0, layer);
            }
        } else {
            // SURVIVAL INVENTORY: Quick Crafting + 27-Slot Backpack + 9-Slot Hotbar
            let title = "SURVIVAL INVENTORY & CRAFTING";
            let title_w = title.len() as f32 * 8.0 * 2.0;
            batch.draw_text(title, win_x + (win_w - title_w) * 0.5, win_y + 14.0, 2.0, [1.0, 1.0, 0.6, 1.0]);

            // Crafting Section
            let craft_title = "CRAFTING (Click recipe to craft):";
            batch.draw_text(craft_title, win_x + 36.0, win_y + 42.0, 1.3, [0.85, 0.9, 1.0, 0.95]);

            let btn_w = 236.0;
            let btn_h = 30.0;
            for (r_idx, recipe) in Inventory::RECIPES.iter().enumerate() {
                let row = r_idx / 2;
                let col = r_idx % 2;
                let rx = win_x + 36.0 + (col as f32) * (btn_w + 16.0);
                let ry = win_y + 62.0 + (row as f32) * (btn_h + 8.0);

                let can_craft = inventory.can_craft(r_idx);
                let is_hover = mouse_pos.0 >= rx
                    && mouse_pos.0 <= rx + btn_w
                    && mouse_pos.1 >= ry
                    && mouse_pos.1 <= ry + btn_h;

                if is_hover && can_craft {
                    action = Some(InventoryAction::CraftRecipe(r_idx));
                }

                let bg = if can_craft {
                    if is_hover {
                        [0.25, 0.55, 0.30, 1.0]
                    } else {
                        [0.18, 0.38, 0.22, 0.95]
                    }
                } else {
                    [0.14, 0.14, 0.16, 0.85]
                };
                batch.draw_rect(rx, ry, btn_w, btn_h, bg);
                let border = if can_craft && is_hover {
                    [0.5, 1.0, 0.5, 1.0]
                } else if can_craft {
                    [0.3, 0.7, 0.35, 1.0]
                } else {
                    [0.28, 0.28, 0.32, 0.8]
                };
                batch.draw_rect_outline(rx, ry, btn_w, btn_h, 1.5, border);

                // Output icon preview
                let out_layer = recipe.output.texture_layer(2);
                batch.draw_block_icon(rx + 5.0, ry + 4.0, 22.0, out_layer);

                let txt_color = if can_craft {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    [0.5, 0.5, 0.5, 0.8]
                };
                batch.draw_text(recipe.label, rx + 34.0, ry + 10.0, 1.2, txt_color);
            }

            // 27-Slot Storage Grid (3 rows of 9 slots)
            let storage_label = "BACKPACK STORAGE (Click two slots to swap/merge):";
            batch.draw_text(storage_label, win_x + 36.0, win_y + 144.0, 1.25, [0.85, 0.85, 0.85, 0.9]);

            let st_size = 46.0;
            let st_spacing = 52.0;
            let st_start_x = win_x + (win_w - 9.0 * st_spacing) * 0.5 + 3.0;
            let st_start_y = win_y + 164.0;

            for s_idx in 0..27 {
                let combined_idx = 9 + s_idx;
                let row = s_idx / 9;
                let col = s_idx % 9;
                let sx = st_start_x + (col as f32) * st_spacing;
                let sy = st_start_y + (row as f32) * st_spacing;

                let is_hover = mouse_pos.0 >= sx
                    && mouse_pos.0 <= sx + st_size
                    && mouse_pos.1 >= sy
                    && mouse_pos.1 <= sy + st_size;
                let is_held = inventory.held_slot == Some(combined_idx);

                if is_hover {
                    action = Some(InventoryAction::ClickCombinedSlot(combined_idx));
                }

                let bg = if is_held {
                    [0.55, 0.50, 0.25, 1.0]
                } else if is_hover {
                    [0.32, 0.36, 0.46, 1.0]
                } else {
                    [0.14, 0.14, 0.17, 1.0]
                };
                batch.draw_rect(sx, sy, st_size, st_size, bg);

                let border = if is_held {
                    [1.0, 0.95, 0.3, 1.0]
                } else if is_hover {
                    [0.75, 0.8, 0.95, 1.0]
                } else {
                    [0.32, 0.32, 0.38, 1.0]
                };
                batch.draw_rect_outline(sx, sy, st_size, st_size, if is_held { 2.5 } else { 1.5 }, border);

                let stack = inventory.storage[s_idx];
                if !stack.is_empty() {
                    let layer = stack.block.texture_layer(2);
                    batch.draw_block_icon(sx + 5.0, sy + 5.0, st_size - 10.0, layer);

                    let count_str = stack.count.to_string();
                    let cx = sx + st_size - count_str.len() as f32 * 8.0 - 3.0;
                    let cy = sy + st_size - 11.0;
                    batch.draw_text(&count_str, cx + 1.0, cy + 1.0, 1.0, [0.0, 0.0, 0.0, 0.9]);
                    batch.draw_text(&count_str, cx, cy, 1.0, [1.0, 1.0, 1.0, 1.0]);
                }
            }
        }

        // Hotbar Section Divider
        let div_y = win_y + 330.0;
        batch.draw_rect(win_x + 24.0, div_y, win_w - 48.0, 2.0, [0.35, 0.35, 0.40, 1.0]);

        let hotbar_title = "HOTBAR (SLOTS 1 - 9):";
        batch.draw_text(hotbar_title, win_x + 36.0, div_y + 10.0, 1.4, [0.9, 0.9, 0.9, 0.9]);

        let hb_size = 46.0;
        let hb_spacing = 52.0;
        let hb_start_x = win_x + (win_w - 9.0 * hb_spacing) * 0.5 + 3.0;
        let hb_start_y = div_y + 32.0;

        for i in 0..9 {
            let sx = hb_start_x + (i as f32) * hb_spacing;
            let is_selected = i == inventory.selected_slot;
            let is_held = inventory.held_slot == Some(i);

            let is_hover = mouse_pos.0 >= sx
                && mouse_pos.0 <= sx + hb_size
                && mouse_pos.1 >= hb_start_y
                && mouse_pos.1 <= hb_start_y + hb_size;

            if is_hover {
                action = Some(InventoryAction::ClickCombinedSlot(i));
            }

            let bg = if is_held {
                [0.55, 0.50, 0.25, 1.0]
            } else if is_selected {
                [0.4, 0.5, 0.7, 1.0]
            } else if is_hover {
                [0.3, 0.35, 0.45, 1.0]
            } else {
                [0.15, 0.15, 0.18, 1.0]
            };
            batch.draw_rect(sx, hb_start_y, hb_size, hb_size, bg);

            let border = if is_held {
                [1.0, 0.95, 0.3, 1.0]
            } else if is_selected {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.35, 0.35, 0.40, 1.0]
            };
            batch.draw_rect_outline(
                sx,
                hb_start_y,
                hb_size,
                hb_size,
                if is_selected || is_held { 3.0 } else { 1.5 },
                border,
            );

            let stack = inventory.hotbar[i];
            if !stack.is_empty() {
                let layer = stack.block.texture_layer(2);
                batch.draw_block_icon(sx + 5.0, hb_start_y + 5.0, hb_size - 10.0, layer);

                if game_mode == GameMode::Survival {
                    let count_str = stack.count.to_string();
                    let cx = sx + hb_size - count_str.len() as f32 * 8.0 - 3.0;
                    let cy = hb_start_y + hb_size - 11.0;
                    batch.draw_text(&count_str, cx + 1.0, cy + 1.0, 1.0, [0.0, 0.0, 0.0, 0.9]);
                    batch.draw_text(&count_str, cx, cy, 1.0, [1.0, 1.0, 1.0, 1.0]);
                }
            }

            batch.draw_text(&(i + 1).to_string(), sx + 3.0, hb_start_y + 2.0, 1.0, [1.0, 1.0, 1.0, 0.8]);
        }

        let close_txt = "[E] or [ESC] to return to game";
        let c_w = close_txt.len() as f32 * 8.0 * 1.3;
        batch.draw_text(close_txt, win_x + (win_w - c_w) * 0.5, win_y + win_h - 24.0, 1.3, [0.6, 0.6, 0.6, 0.9]);

        action
    }

    pub fn render_pause_menu(
        batch: &mut UiBatch,
        screen_w: f32,
        screen_h: f32,
        game_mode: GameMode,
        mouse_pos: (f32, f32),
    ) -> Vec<(String, f32, f32, f32, f32)> {
        batch.draw_rect(0.0, 0.0, screen_w, screen_h, [0.0, 0.0, 0.0, 0.70]);

        let title = "GAME PAUSED";
        let title_w = title.len() as f32 * 8.0 * 3.5;
        let title_x = (screen_w - title_w) * 0.5;
        batch.draw_text(title, title_x, 100.0, 3.5, [1.0, 1.0, 1.0, 1.0]);

        let btn_w = 320.0;
        let btn_h = 46.0;
        let btn_x = (screen_w - btn_w) * 0.5;
        let start_y = 190.0;
        let spacing = 60.0;

        let mode_label = match game_mode {
            GameMode::Survival => "Game Mode: SURVIVAL",
            GameMode::Creative => "Game Mode: CREATIVE",
        };

        let buttons = [
            ("resume", "Back to Game"),
            ("toggle_mode", mode_label),
            ("settings", "Graphics Settings"),
            ("title", "Save & Quit to Title"),
        ];

        let mut hitboxes = Vec::new();

        for (i, (id, label)) in buttons.iter().enumerate() {
            let y = start_y + (i as f32) * spacing;
            let is_hovered = mouse_pos.0 >= btn_x
                && mouse_pos.0 <= btn_x + btn_w
                && mouse_pos.1 >= y
                && mouse_pos.1 <= y + btn_h;

            batch.draw_button(btn_x, y, btn_w, btn_h, label, is_hovered);
            hitboxes.push((id.to_string(), btn_x, y, btn_w, btn_h));
        }

        hitboxes
    }

    pub fn render_settings_menu(
        batch: &mut UiBatch,
        screen_w: f32,
        screen_h: f32,
        settings: &GraphicsSettings,
        mouse_pos: (f32, f32),
    ) -> Vec<(String, f32, f32, f32, f32)> {
        batch.draw_rect(0.0, 0.0, screen_w, screen_h, [0.12, 0.14, 0.18, 0.95]);

        let title = "GRAPHICS SETTINGS";
        let title_w = title.len() as f32 * 8.0 * 3.0;
        batch.draw_text(title, (screen_w - title_w) * 0.5, 60.0, 3.0, [1.0, 1.0, 0.6, 1.0]);

        let btn_w = 360.0;
        let btn_h = 44.0;
        let btn_x = (screen_w - btn_w) * 0.5;
        let start_y = 150.0;
        let spacing = 58.0;

        let rd_label = format!("Render Distance: {} Chunks", settings.render_distance);
        let fov_label = format!("FOV: {:.0} Degrees", settings.fov_degrees);
        let fog_label = format!("Atmospheric Fog: {}", if settings.fog_enabled { "ON" } else { "OFF" });
        let vsync_label = format!("Vertical Sync: {}", if settings.vsync_enabled { "ON" } else { "OFF" });

        let buttons = [
            ("cycle_rd", rd_label),
            ("cycle_fov", fov_label),
            ("toggle_fog", fog_label),
            ("toggle_vsync", vsync_label),
            ("back", "Done & Back".to_string()),
        ];

        let mut hitboxes = Vec::new();

        for (i, (id, label)) in buttons.iter().enumerate() {
            let y = start_y + (i as f32) * spacing;
            let is_hovered = mouse_pos.0 >= btn_x
                && mouse_pos.0 <= btn_x + btn_w
                && mouse_pos.1 >= y
                && mouse_pos.1 <= y + btn_h;

            batch.draw_button(btn_x, y, btn_w, btn_h, &label, is_hovered);
            hitboxes.push((id.to_string(), btn_x, y, btn_w, btn_h));
        }

        hitboxes
    }

    pub fn render_game_over(
        batch: &mut UiBatch,
        screen_w: f32,
        screen_h: f32,
        mouse_pos: (f32, f32),
    ) -> Vec<(String, f32, f32, f32, f32)> {
        batch.draw_rect(0.0, 0.0, screen_w, screen_h, [0.35, 0.05, 0.05, 0.85]);

        let title = "YOU DIED!";
        let title_w = title.len() as f32 * 8.0 * 5.0;
        batch.draw_text(title, (screen_w - title_w) * 0.5, 140.0, 5.0, [1.0, 0.2, 0.2, 1.0]);

        let btn_w = 300.0;
        let btn_h = 48.0;
        let btn_x = (screen_w - btn_w) * 0.5;

        let buttons = [
            ("respawn", "Respawn", 260.0),
            ("title", "Title Screen", 328.0),
        ];

        let mut hitboxes = Vec::new();

        for (id, label, y) in buttons {
            let is_hovered = mouse_pos.0 >= btn_x
                && mouse_pos.0 <= btn_x + btn_w
                && mouse_pos.1 >= y
                && mouse_pos.1 <= y + btn_h;

            batch.draw_button(btn_x, y, btn_w, btn_h, label, is_hovered);
            hitboxes.push((id.to_string(), btn_x, y, btn_w, btn_h));
        }

        hitboxes
    }
}
