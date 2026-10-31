use super::font::get_char_uv;
use super::vertex::UiVertex;

pub struct UiBatch {
    pub vertices: Vec<UiVertex>,
    pub indices: Vec<u32>,
}

impl UiBatch {
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(2048),
            indices: Vec::with_capacity(3072),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn push_quad(&mut self, quad: [UiVertex; 4]) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&quad);
        self.indices.extend_from_slice(&[
            base,
            base + 1,
            base + 2,
            base,
            base + 2,
            base + 3,
        ]);
    }

    pub fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        let quad = [
            UiVertex { position: [x, y], uv: [0.0, 0.0], color, mode: 0, tex_layer: 0 },
            UiVertex { position: [x + w, y], uv: [1.0, 0.0], color, mode: 0, tex_layer: 0 },
            UiVertex { position: [x + w, y + h], uv: [1.0, 1.0], color, mode: 0, tex_layer: 0 },
            UiVertex { position: [x, y + h], uv: [0.0, 1.0], color, mode: 0, tex_layer: 0 },
        ];
        self.push_quad(quad);
    }

    pub fn draw_rect_outline(&mut self, x: f32, y: f32, w: f32, h: f32, t: f32, color: [f32; 4]) {
        // Top
        self.draw_rect(x, y, w, t, color);
        // Bottom
        self.draw_rect(x, y + h - t, w, t, color);
        // Left
        self.draw_rect(x, y + t, t, h - 2.0 * t, color);
        // Right
        self.draw_rect(x + w - t, y + t, t, h - 2.0 * t, color);
    }

    pub fn draw_button(&mut self, x: f32, y: f32, w: f32, h: f32, text: &str, is_hovered: bool) {
        let bg_color = if is_hovered {
            [0.55, 0.60, 0.75, 0.95] // Light blue-grey highlight
        } else {
            [0.28, 0.28, 0.32, 0.90] // Minecraft stone dark grey
        };
        let border_light = if is_hovered { [0.85, 0.90, 1.0, 1.0] } else { [0.45, 0.45, 0.50, 1.0] };
        let border_dark = if is_hovered { [0.35, 0.40, 0.50, 1.0] } else { [0.15, 0.15, 0.18, 1.0] };

        // Background
        self.draw_rect(x, y, w, h, bg_color);

        // 3D Bevel effect (Minecraft button style)
        self.draw_rect(x, y, w, 3.0, border_light); // Top
        self.draw_rect(x, y, 3.0, h, border_light); // Left
        self.draw_rect(x, y + h - 3.0, w, 3.0, border_dark); // Bottom
        self.draw_rect(x + w - 3.0, y, 3.0, h, border_dark); // Right

        // Center text
        let font_scale = 2.0;
        let text_w = text.len() as f32 * 8.0 * font_scale;
        let text_h = 8.0 * font_scale;
        let text_x = x + (w - text_w) * 0.5;
        let text_y = y + (h - text_h) * 0.5;

        // Text shadow
        let shadow_color = [0.1, 0.1, 0.1, 0.8];
        self.draw_text(text, text_x + 2.0, text_y + 2.0, font_scale, shadow_color);

        // Text main
        let text_color = if is_hovered {
            [1.0, 1.0, 0.6, 1.0] // Yellow on hover
        } else {
            [1.0, 1.0, 1.0, 1.0] // White
        };
        self.draw_text(text, text_x, text_y, font_scale, text_color);
    }

    pub fn draw_text(&mut self, text: &str, start_x: f32, start_y: f32, scale: f32, color: [f32; 4]) {
        let char_w = 8.0 * scale;
        let char_h = 8.0 * scale;
        let mut cur_x = start_x;
        let mut cur_y = start_y;

        for ch in text.chars() {
            if ch == '\n' {
                cur_x = start_x;
                cur_y += char_h + 2.0 * scale;
                continue;
            }

            let [u0, v0, u1, v1] = get_char_uv(ch);
            let quad = [
                UiVertex { position: [cur_x, cur_y], uv: [u0, v0], color, mode: 1, tex_layer: 0 },
                UiVertex { position: [cur_x + char_w, cur_y], uv: [u1, v0], color, mode: 1, tex_layer: 0 },
                UiVertex { position: [cur_x + char_w, cur_y + char_h], uv: [u1, v1], color, mode: 1, tex_layer: 0 },
                UiVertex { position: [cur_x, cur_y + char_h], uv: [u0, v1], color, mode: 1, tex_layer: 0 },
            ];
            self.push_quad(quad);
            cur_x += char_w;
        }
    }

    pub fn draw_block_icon(&mut self, x: f32, y: f32, size: f32, tex_layer: u32) {
        let quad = [
            UiVertex { position: [x, y], uv: [0.0, 0.0], color: [1.0, 1.0, 1.0, 1.0], mode: 2, tex_layer },
            UiVertex { position: [x + size, y], uv: [1.0, 0.0], color: [1.0, 1.0, 1.0, 1.0], mode: 2, tex_layer },
            UiVertex { position: [x + size, y + size], uv: [1.0, 1.0], color: [1.0, 1.0, 1.0, 1.0], mode: 2, tex_layer },
            UiVertex { position: [x, y + size], uv: [0.0, 1.0], color: [1.0, 1.0, 1.0, 1.0], mode: 2, tex_layer },
        ];
        self.push_quad(quad);
    }

    pub fn draw_heart(&mut self, x: f32, y: f32, size: f32, is_half: bool, is_empty: bool) {
        let bg_color = [0.15, 0.15, 0.15, 0.9];
        // Background black container
        self.draw_rect(x, y, size, size, bg_color);

        if !is_empty {
            let heart_color = [0.95, 0.15, 0.15, 1.0]; // Red
            let fill_w = if is_half { size * 0.5 } else { size - 2.0 };
            self.draw_rect(x + 1.0, y + 1.0, fill_w, size - 2.0, heart_color);
        }

        // Heart border
        self.draw_rect_outline(x, y, size, size, 1.0, [0.05, 0.05, 0.05, 1.0]);
    }

    pub fn draw_crosshair(&mut self, screen_w: f32, screen_h: f32) {
        let cx = screen_w * 0.5;
        let cy = screen_h * 0.5;
        let size = 9.0;
        let thick = 2.0;
        let color = [1.0, 1.0, 1.0, 0.85];

        // Horizontal bar
        self.draw_rect(cx - size, cy - thick * 0.5, size * 2.0, thick, color);
        // Vertical bar
        self.draw_rect(cx - thick * 0.5, cy - size, thick, size * 2.0, color);
    }
}
