use super::draw::UiBatch;
use crate::game::inventory::Inventory;
use crate::game::save::WorldSaveData;
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
        batch.draw_rect(0.0, 0.0, screen_w, screen_h, [0.10, 0.13, 0.18, 1.0]);

        // Header bar
        batch.draw_rect(0.0, 0.0, screen_w, 136.0, [0.07, 0.09, 0.12, 1.0]);
        batch.draw_rect(0.0, 136.0, screen_w, 4.0, [0.25, 0.45, 0.65, 1.0]);

        // Main Title: RUSTCRAFT
        let title = "RUSTCRAFT";
        let title_scale = 5.0;
        let title_w = title.len() as f32 * 8.0 * title_scale;
        let title_x = (screen_w - title_w) * 0.5;
        let title_y = 40.0;

        batch.draw_text(title, title_x + 4.0, title_y + 4.0, title_scale, [0.0, 0.0, 0.0, 0.9]);
        batch.draw_text(title, title_x, title_y, title_scale, [0.95, 0.85, 0.20, 1.0]);

        let subtitle = "Complete Base Edition - Biomes, Mobs, Tools, Smelting & Day/Night";
        let sub_scale = 1.6;
        let sub_w = subtitle.len() as f32 * 8.0 * sub_scale;
        batch.draw_text(subtitle, (screen_w - sub_w) * 0.5, 102.0, sub_scale, [0.65, 0.85, 1.0, 0.95]);

        let btn_w = 360.0;
        let btn_h = 46.0;
        let btn_x = (screen_w - btn_w) * 0.5;
        let start_y = 176.0;
        let spacing = 58.0;

        let mut buttons: Vec<(&str, &str)> = Vec::new();
        if WorldSaveData::exists() {
            buttons.push(("load_saved", "Continue Saved World"));
        }
        buttons.push(("play_survival", "New World (Survival Mode)"));
        buttons.push(("play_creative", "New World (Creative Mode)"));
        buttons.push(("settings", "Graphics Settings"));
        buttons.push(("quit", "Quit Game"));

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

        let footer = "Rust 2024 + Vulkan (wgpu) | Full Survival, Mobs, Biomes, Caves, Ores, TNT & Persistence";
        batch.draw_text(footer, 16.0, screen_h - 28.0, 1.3, [0.55, 0.60, 0.68, 0.85]);

        hitboxes
    }

    #[allow(clippy::too_many_arguments)]
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
        head_underwater: bool,
        biome_name: &str,
        time_of_day: f32,
        mob_count: usize,
        status_msg: Option<&str>,
    ) {
        // Underwater blue screen tint
        if head_underwater {
            batch.draw_rect(0.0, 0.0, screen_w, screen_h, [0.05, 0.25, 0.55, 0.35]);
        }

        // Hurt flash red vignette when damaged
        if survival.hurt_timer > 0.0 {
            let alpha = (survival.hurt_timer / 0.45).clamp(0.0, 1.0) * 0.38;
            batch.draw_rect(0.0, 0.0, screen_w, screen_h, [0.9, 0.05, 0.05, alpha]);
        }

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
        batch.draw_text(mode_str, 16.0, 14.0, 1.8, [1.0, 1.0, 1.0, 0.92]);

        let phase = if (0.20..0.30).contains(&time_of_day) {
            "Sunrise"
        } else if (0.30..0.70).contains(&time_of_day) {
            "Day"
        } else if (0.70..0.80).contains(&time_of_day) {
            "Sunset"
        } else {
            "Night"
        };

        let pos_str = format!(
            "XYZ: {:.1} / {:.1} / {:.1}  |  Biome: {}  |  Time: {} ({})",
            pos.x, pos.y, pos.z, biome_name, phase, mob_count
        );
        batch.draw_text(&pos_str, 16.0, 34.0, 1.35, [0.85, 0.90, 0.95, 0.88]);

        let fps_str = format!("FPS: {:.0}", fps);
        batch.draw_text(&fps_str, 16.0, 52.0, 1.35, [0.5, 1.0, 0.5, 0.85]);

        let controls_str =
            "[E] Inv/Craft  [Q] Drop  [RMB] Place/Use/Eat  [T] Time  [F5] Save  [Shift] Crouch";
        batch.draw_text(controls_str, 16.0, 70.0, 1.15, [0.72, 0.78, 0.88, 0.78]);

        // Status message banner at top center
        if let Some(msg) = status_msg {
            let scale = 1.6;
            let msg_w = msg.len() as f32 * 8.0 * scale;
            let mx = (screen_w - msg_w) * 0.5;
            let my = 24.0;
            batch.draw_rect(mx - 10.0, my - 6.0, msg_w + 20.0, 26.0, [0.08, 0.10, 0.14, 0.88]);
            batch.draw_rect_outline(mx - 10.0, my - 6.0, msg_w + 20.0, 26.0, 1.5, [0.4, 0.8, 0.45, 0.95]);
            batch.draw_text(msg, mx, my, scale, [0.6, 1.0, 0.65, 1.0]);
        }

        // 9-slot Hotbar dimensions
        let slot_size = 40.0;
        let slot_spacing = 44.0;
        let total_w = 9.0 * slot_spacing;
        let hotbar_x = (screen_w - total_w) * 0.5;
        let hotbar_y = screen_h - 52.0;

        // Survival Health, Hunger & Oxygen HUD above the hotbar
        if game_mode == GameMode::Survival {
            let icon_size = 13.0;
            let icon_spacing = 15.0;
            let bar_y = hotbar_y - 38.0;

            // Left: 10 Hearts
            for i in 0..10 {
                let x = hotbar_x + (i as f32) * icon_spacing;
                let heart_val = (i + 1) * 2;
                let is_empty = survival.health < (heart_val - 1) as f32;
                let is_half = survival.health >= (heart_val - 1) as f32 && survival.health < heart_val as f32;
                batch.draw_heart(x, bar_y, icon_size, is_half, is_empty);
            }

            // Right: 10 Hunger drumsticks
            let hunger_start_x = hotbar_x + total_w - 4.0 - 10.0 * icon_spacing;
            for i in 0..10 {
                let x = hunger_start_x + (i as f32) * icon_spacing;
                let food_val = (i + 1) * 2;
                let is_empty = survival.hunger < (food_val - 1) as f32;
                let is_half = survival.hunger >= (food_val - 1) as f32 && survival.hunger < food_val as f32;
                batch.draw_hunger_icon(x, bar_y, icon_size, is_half, is_empty);
            }

            // Above Hearts: 10 Oxygen Bubbles when underwater or recovering breath
            if head_underwater || survival.oxygen < survival.max_oxygen - 0.1 {
                let oxy_y = bar_y - 17.0;
                for i in 0..10 {
                    let x = hotbar_x + (i as f32) * icon_spacing;
                    let bubble_val = (i + 1) * 2;
                    let is_empty = survival.oxygen < (bubble_val - 1) as f32;
                    batch.draw_bubble_icon(x, oxy_y, icon_size, is_empty);
                }
            }
        }

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

                if game_mode == GameMode::Survival && stack.count > 1 {
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

        // Active item/block name above hotbar
        let sel_stack = inventory.selected_stack();
        let sel_name = if sel_stack.is_empty() {
            "Empty Hand".to_string()
        } else {
            let base = sel_stack.block.name();
            if let Some(food_pts) = sel_stack.block.food_restore() {
                format!("{} (+{:.0} Food) [RMB to Eat]", base, food_pts)
            } else if sel_stack.block.is_tool_or_weapon() {
                format!("{} (Atk: {:.0})", base, sel_stack.block.attack_damage())
            } else if game_mode == GameMode::Survival {
                format!("{} ({})", base, sel_stack.count)
            } else {
                base.to_string()
            }
        };

        let sel_scale = 1.35;
        let name_w = sel_name.len() as f32 * 8.0 * sel_scale;
        let name_x = (screen_w - name_w) * 0.5;
        let name_y = hotbar_y - 19.0;
        batch.draw_text(&sel_name, name_x, name_y, sel_scale, [1.0, 1.0, 0.6, 0.95]);
    }

    pub fn render_inventory(
        batch: &mut UiBatch,
        screen_w: f32,
        screen_h: f32,
        inventory: &Inventory,
        game_mode: GameMode,
        near_table: bool,
        near_furnace: bool,
        mouse_pos: (f32, f32),
    ) -> Option<InventoryAction> {
        // Dark translucent overlay
        batch.draw_rect(0.0, 0.0, screen_w, screen_h, [0.0, 0.0, 0.0, 0.78]);

        let win_w = 640.0;
        let win_h = 548.0;
        let win_x = (screen_w - win_w) * 0.5;
        let win_y = (screen_h - win_h) * 0.5;

        // Window background with Minecraft border
        batch.draw_rect(win_x, win_y, win_w, win_h, [0.18, 0.19, 0.22, 0.98]);
        batch.draw_rect_outline(win_x, win_y, win_w, win_h, 3.0, [0.52, 0.55, 0.62, 1.0]);

        let mut action = None;
        let mut hovered_tooltip: Option<String> = None;

        if game_mode == GameMode::Creative {
            let title = "CREATIVE BLOCK & ITEM PALETTE";
            let title_w = title.len() as f32 * 8.0 * 2.0;
            batch.draw_text(title, win_x + (win_w - title_w) * 0.5, win_y + 16.0, 2.0, [1.0, 1.0, 0.6, 1.0]);

            let help = "Click any block/tool/item to assign to the selected hotbar slot";
            let help_w = help.len() as f32 * 8.0 * 1.2;
            batch.draw_text(help, win_x + (win_w - help_w) * 0.5, win_y + 42.0, 1.2, [0.78, 0.80, 0.85, 0.9]);

            // Palette grid (4 rows of 7 items = 28 items)
            let cols = 7;
            let slot_size = 54.0;
            let spacing = 66.0;
            let start_x = win_x + (win_w - (cols as f32) * spacing) * 0.5 + 6.0;
            let start_y = win_y + 72.0;

            for (idx, &block) in Inventory::PALETTE.iter().enumerate() {
                let row = idx / cols;
                let col = idx % cols;
                let sx = start_x + (col as f32) * spacing;
                let sy = start_y + (row as f32) * spacing;

                let is_hover = mouse_pos.0 >= sx
                    && mouse_pos.0 <= sx + slot_size
                    && mouse_pos.1 >= sy
                    && mouse_pos.1 <= sy + slot_size;

                if is_hover {
                    action = Some(InventoryAction::SelectPalette(idx));
                    hovered_tooltip = Some(block.name().to_string());
                }

                let bg = if is_hover {
                    [0.45, 0.55, 0.75, 1.0]
                } else {
                    [0.14, 0.14, 0.17, 1.0]
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
            // SURVIVAL INVENTORY: Crafting + Smelting + 27-Slot Backpack + 9-Slot Hotbar
            let title = "SURVIVAL INVENTORY, CRAFTING & SMELTING";
            let title_w = title.len() as f32 * 8.0 * 1.75;
            batch.draw_text(title, win_x + (win_w - title_w) * 0.5, win_y + 12.0, 1.75, [1.0, 1.0, 0.6, 1.0]);

            // Station availability badges
            let table_status = if near_table {
                ("Table: ACTIVE", [0.35, 0.95, 0.45, 1.0])
            } else {
                ("Table: Place nearby for [T] recipes", [0.75, 0.65, 0.35, 0.9])
            };
            let furnace_status = if near_furnace {
                ("Furnace: ACTIVE", [1.0, 0.65, 0.25, 1.0])
            } else {
                ("Furnace: Place nearby for [F] smelting", [0.75, 0.55, 0.45, 0.9])
            };
            batch.draw_text(table_status.0, win_x + 22.0, win_y + 34.0, 1.1, table_status.1);
            batch.draw_text(furnace_status.0, win_x + 335.0, win_y + 34.0, 1.1, furnace_status.1);

            // 18 Recipes in 3 columns x 6 rows
            let btn_w = 192.0;
            let btn_h = 26.0;
            for (r_idx, recipe) in Inventory::RECIPES.iter().enumerate() {
                let row = r_idx / 3;
                let col = r_idx % 3;
                let rx = win_x + 20.0 + (col as f32) * (btn_w + 12.0);
                let ry = win_y + 52.0 + (row as f32) * (btn_h + 5.0);

                let station_ok = (!recipe.requires_table || near_table)
                    && (!recipe.requires_furnace || near_furnace);
                let can_craft = inventory.can_craft(r_idx, near_table, near_furnace);
                let is_hover = mouse_pos.0 >= rx
                    && mouse_pos.0 <= rx + btn_w
                    && mouse_pos.1 >= ry
                    && mouse_pos.1 <= ry + btn_h;

                if is_hover {
                    if can_craft {
                        action = Some(InventoryAction::CraftRecipe(r_idx));
                    }
                    hovered_tooltip = Some(format!(
                        "{} -> {}x {}",
                        recipe.label,
                        recipe.output_count,
                        recipe.output.name()
                    ));
                }

                let bg = if can_craft {
                    if is_hover {
                        [0.25, 0.58, 0.32, 1.0]
                    } else {
                        [0.18, 0.40, 0.23, 0.95]
                    }
                } else if !station_ok {
                    [0.18, 0.14, 0.14, 0.85]
                } else {
                    [0.13, 0.14, 0.16, 0.85]
                };
                batch.draw_rect(rx, ry, btn_w, btn_h, bg);
                let border = if can_craft && is_hover {
                    [0.5, 1.0, 0.5, 1.0]
                } else if can_craft {
                    [0.3, 0.72, 0.36, 1.0]
                } else {
                    [0.26, 0.27, 0.30, 0.8]
                };
                batch.draw_rect_outline(rx, ry, btn_w, btn_h, 1.5, border);

                let out_layer = recipe.output.texture_layer(2);
                batch.draw_block_icon(rx + 4.0, ry + 3.0, 20.0, out_layer);

                let txt_color = if can_craft {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    [0.52, 0.52, 0.55, 0.85]
                };
                // Truncate label slightly so it fits cleanly inside 192px button
                let short_label: String = recipe.label.chars().take(19).collect();
                batch.draw_text(&short_label, rx + 28.0, ry + 9.0, 1.0, txt_color);
            }

            // 27-Slot Storage Grid (3 rows of 9 slots)
            let storage_label = "BACKPACK STORAGE (Click two slots to swap/merge):";
            batch.draw_text(storage_label, win_x + 24.0, win_y + 244.0, 1.2, [0.85, 0.85, 0.85, 0.9]);

            let st_size = 44.0;
            let st_spacing = 50.0;
            let st_start_x = win_x + (win_w - 9.0 * st_spacing) * 0.5 + 3.0;
            let st_start_y = win_y + 262.0;

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

                let stack = inventory.storage[s_idx];
                if is_hover {
                    action = Some(InventoryAction::ClickCombinedSlot(combined_idx));
                    if !stack.is_empty() {
                        hovered_tooltip = Some(format!("{} ({})", stack.block.name(), stack.count));
                    }
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

                if !stack.is_empty() {
                    let layer = stack.block.texture_layer(2);
                    batch.draw_block_icon(sx + 5.0, sy + 5.0, st_size - 10.0, layer);

                    if stack.count > 1 {
                        let count_str = stack.count.to_string();
                        let cx = sx + st_size - count_str.len() as f32 * 8.0 - 3.0;
                        let cy = sy + st_size - 11.0;
                        batch.draw_text(&count_str, cx + 1.0, cy + 1.0, 1.0, [0.0, 0.0, 0.0, 0.9]);
                        batch.draw_text(&count_str, cx, cy, 1.0, [1.0, 1.0, 1.0, 1.0]);
                    }
                }
            }
        }

        // Hotbar Section Divider
        let div_y = win_y + 420.0;
        batch.draw_rect(win_x + 24.0, div_y, win_w - 48.0, 2.0, [0.35, 0.35, 0.40, 1.0]);

        let hotbar_title = "HOTBAR (SLOTS 1 - 9):";
        batch.draw_text(hotbar_title, win_x + 24.0, div_y + 8.0, 1.3, [0.9, 0.9, 0.9, 0.9]);

        let hb_size = 44.0;
        let hb_spacing = 50.0;
        let hb_start_x = win_x + (win_w - 9.0 * hb_spacing) * 0.5 + 3.0;
        let hb_start_y = div_y + 28.0;

        for i in 0..9 {
            let sx = hb_start_x + (i as f32) * hb_spacing;
            let is_selected = i == inventory.selected_slot;
            let is_held = inventory.held_slot == Some(i);

            let is_hover = mouse_pos.0 >= sx
                && mouse_pos.0 <= sx + hb_size
                && mouse_pos.1 >= hb_start_y
                && mouse_pos.1 <= hb_start_y + hb_size;

            let stack = inventory.hotbar[i];
            if is_hover {
                action = Some(InventoryAction::ClickCombinedSlot(i));
                if !stack.is_empty() {
                    hovered_tooltip = Some(format!("{} ({})", stack.block.name(), stack.count));
                }
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

            if !stack.is_empty() {
                let layer = stack.block.texture_layer(2);
                batch.draw_block_icon(sx + 5.0, hb_start_y + 5.0, hb_size - 10.0, layer);

                if game_mode == GameMode::Survival && stack.count > 1 {
                    let count_str = stack.count.to_string();
                    let cx = sx + hb_size - count_str.len() as f32 * 8.0 - 3.0;
                    let cy = hb_start_y + hb_size - 11.0;
                    batch.draw_text(&count_str, cx + 1.0, cy + 1.0, 1.0, [0.0, 0.0, 0.0, 0.9]);
                    batch.draw_text(&count_str, cx, cy, 1.0, [1.0, 1.0, 1.0, 1.0]);
                }
            }

            batch.draw_text(&(i + 1).to_string(), sx + 3.0, hb_start_y + 2.0, 1.0, [1.0, 1.0, 1.0, 0.8]);
        }

        // Hovered item/recipe tooltip at bottom of window
        if let Some(tip) = hovered_tooltip {
            let tip_w = tip.len() as f32 * 8.0 * 1.4;
            batch.draw_text(
                &tip,
                win_x + (win_w - tip_w) * 0.5,
                win_y + win_h - 42.0,
                1.4,
                [1.0, 0.95, 0.55, 1.0],
            );
        }

        let close_txt = "[E] or [ESC] to return to game";
        let c_w = close_txt.len() as f32 * 8.0 * 1.25;
        batch.draw_text(close_txt, win_x + (win_w - c_w) * 0.5, win_y + win_h - 20.0, 1.25, [0.6, 0.6, 0.6, 0.9]);

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
        batch.draw_text(title, title_x, 86.0, 3.5, [1.0, 1.0, 1.0, 1.0]);

        let btn_w = 340.0;
        let btn_h = 46.0;
        let btn_x = (screen_w - btn_w) * 0.5;
        let start_y = 166.0;
        let spacing = 58.0;

        let mode_label = match game_mode {
            GameMode::Survival => "Game Mode: SURVIVAL",
            GameMode::Creative => "Game Mode: CREATIVE",
        };

        let buttons = [
            ("resume", "Back to Game"),
            ("save", "Save World Now ([F5])"),
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
