use super::draw::UiBatch;
use crate::game::inventory::Inventory;
use crate::game::settings::GraphicsSettings;
use crate::game::state::GameMode;
use crate::game::survival::SurvivalState;

pub struct MenuRenderer;

impl MenuRenderer {
    pub fn render_main_menu(
        batch: &mut UiBatch,
        screen_w: f32,
        screen_h: f32,
        mouse_pos: (f32, f32),
    ) -> Vec<(String, f32, f32, f32, f32)> {
        // Tiled or gradient background
        batch.draw_rect(0.0, 0.0, screen_w, screen_h, [0.12, 0.14, 0.18, 1.0]);

        // Decorative dirt pattern or gradient header
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
    ) {
        // Crosshair at screen center
        batch.draw_crosshair(screen_w, screen_h);

        // Top-left HUD info
        let mode_str = match game_mode {
            GameMode::Survival => "Mode: SURVIVAL",
            GameMode::Creative => "Mode: CREATIVE (Fly: [F])",
        };
        batch.draw_text(mode_str, 16.0, 16.0, 2.0, [1.0, 1.0, 1.0, 0.9]);

        let pos_str = format!("XYZ: {:.1} / {:.1} / {:.1}", pos.x, pos.y, pos.z);
        batch.draw_text(&pos_str, 16.0, 38.0, 1.5, [0.8, 0.8, 0.8, 0.8]);

        let fps_str = format!("FPS: {:.0}", fps);
        batch.draw_text(&fps_str, 16.0, 56.0, 1.5, [0.5, 1.0, 0.5, 0.8]);

        // Survival Hearts HUD (if in Survival mode)
        if game_mode == GameMode::Survival {
            let heart_size = 14.0;
            let heart_spacing = 16.0;
            let start_x = screen_w * 0.5 - 180.0;
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

            // Block icon inside slot
            let block = inventory.hotbar[i];
            let layer = block.texture_layer(2); // Top face
            batch.draw_block_icon(sx + 4.0, hotbar_y + 4.0, slot_size - 8.0, layer);

            // Slot number 1..9
            let num_str = (i + 1).to_string();
            batch.draw_text(&num_str, sx + 3.0, hotbar_y + 2.0, 1.0, [1.0, 1.0, 1.0, 0.7]);
        }

        // Active block name above hotbar
        let sel_name = format!("{:?}", inventory.selected_block());
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
        mouse_pos: (f32, f32),
    ) -> (Option<usize>, Option<usize>) {
        // Dark translucent overlay
        batch.draw_rect(0.0, 0.0, screen_w, screen_h, [0.0, 0.0, 0.0, 0.75]);

        // Main Dialog Window
        let win_w = 540.0;
        let win_h = 420.0;
        let win_x = (screen_w - win_w) * 0.5;
        let win_y = (screen_h - win_h) * 0.5;

        // Window background with Minecraft border
        batch.draw_rect(win_x, win_y, win_w, win_h, [0.22, 0.22, 0.25, 0.98]);
        batch.draw_rect_outline(win_x, win_y, win_w, win_h, 3.0, [0.50, 0.50, 0.55, 1.0]);

        // Title
        let title = "CREATIVE INVENTORY & PALETTE";
        let title_w = title.len() as f32 * 8.0 * 2.0;
        batch.draw_text(title, win_x + (win_w - title_w) * 0.5, win_y + 20.0, 2.0, [1.0, 1.0, 0.6, 1.0]);

        let help = "Click a block in the palette to assign to active hotbar slot";
        let help_w = help.len() as f32 * 8.0 * 1.2;
        batch.draw_text(help, win_x + (win_w - help_w) * 0.5, win_y + 48.0, 1.2, [0.75, 0.75, 0.75, 0.9]);

        // Palette grid (2 rows of 5 blocks)
        let slot_size = 56.0;
        let spacing = 68.0;
        let start_x = win_x + (win_w - 5.0 * spacing) * 0.5 + 6.0;
        let start_y = win_y + 90.0;

        let mut clicked_palette = None;

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
                clicked_palette = Some(idx);
            }

            let bg = if is_hover { [0.45, 0.55, 0.75, 1.0] } else { [0.15, 0.15, 0.18, 1.0] };
            batch.draw_rect(sx, sy, slot_size, slot_size, bg);
            let border = if is_hover { [1.0, 1.0, 0.5, 1.0] } else { [0.35, 0.35, 0.40, 1.0] };
            batch.draw_rect_outline(sx, sy, slot_size, slot_size, 2.0, border);

            // Block preview
            let layer = block.texture_layer(2);
            batch.draw_block_icon(sx + 6.0, sy + 6.0, slot_size - 12.0, layer);
        }

        // Hotbar Section Divider
        let div_y = win_y + 260.0;
        batch.draw_rect(win_x + 20.0, div_y, win_w - 40.0, 2.0, [0.35, 0.35, 0.40, 1.0]);

        let hotbar_title = "HOTBAR (SLOTS 1 - 9):";
        batch.draw_text(hotbar_title, win_x + 30.0, div_y + 12.0, 1.5, [0.9, 0.9, 0.9, 0.9]);

        // 9 Hotbar slots
        let hb_size = 46.0;
        let hb_spacing = 52.0;
        let hb_start_x = win_x + (win_w - 9.0 * hb_spacing) * 0.5 + 3.0;
        let hb_start_y = div_y + 40.0;

        let mut clicked_hotbar = None;

        for i in 0..9 {
            let sx = hb_start_x + (i as f32) * hb_spacing;
            let is_selected = i == inventory.selected_slot;

            let is_hover = mouse_pos.0 >= sx
                && mouse_pos.0 <= sx + hb_size
                && mouse_pos.1 >= hb_start_y
                && mouse_pos.1 <= hb_start_y + hb_size;

            if is_hover {
                clicked_hotbar = Some(i);
            }

            let bg = if is_selected {
                [0.4, 0.5, 0.7, 1.0]
            } else if is_hover {
                [0.3, 0.35, 0.45, 1.0]
            } else {
                [0.15, 0.15, 0.18, 1.0]
            };
            batch.draw_rect(sx, hb_start_y, hb_size, hb_size, bg);

            let border = if is_selected {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.35, 0.35, 0.40, 1.0]
            };
            batch.draw_rect_outline(sx, hb_start_y, hb_size, hb_size, if is_selected { 3.0 } else { 1.5 }, border);

            // Block preview
            let block = inventory.hotbar[i];
            let layer = block.texture_layer(2);
            batch.draw_block_icon(sx + 5.0, hb_start_y + 5.0, hb_size - 10.0, layer);

            // Number
            batch.draw_text(&(i + 1).to_string(), sx + 3.0, hb_start_y + 2.0, 1.0, [1.0, 1.0, 1.0, 0.8]);
        }

        // Close instruction
        let close_txt = "[E] or [ESC] to return to game";
        let c_w = close_txt.len() as f32 * 8.0 * 1.3;
        batch.draw_text(close_txt, win_x + (win_w - c_w) * 0.5, win_y + win_h - 28.0, 1.3, [0.6, 0.6, 0.6, 0.9]);

        (clicked_palette, clicked_hotbar)
    }

    pub fn render_pause_menu(
        batch: &mut UiBatch,
        screen_w: f32,
        screen_h: f32,
        game_mode: GameMode,
        mouse_pos: (f32, f32),
    ) -> Vec<(String, f32, f32, f32, f32)> {
        // Translucent overlay
        batch.draw_rect(0.0, 0.0, screen_w, screen_h, [0.0, 0.0, 0.0, 0.70]);

        // Title
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
        // Background
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
        // Red tint overlay
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
