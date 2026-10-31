pub struct TextureManager {
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl TextureManager {
    pub const TEX_SIZE: u32 = 16;
    pub const LAYER_COUNT: u32 = 64;

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let size = Self::TEX_SIZE;
        let mut pixels = Vec::with_capacity((size * size * 4 * Self::LAYER_COUNT) as usize);

        for layer in 0..Self::LAYER_COUNT {
            for y in 0..size {
                for x in 0..size {
                    let rgba = Self::generate_pixel(layer, x, y);
                    pixels.extend_from_slice(&rgba);
                }
            }
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Block Texture Array"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: Self::LAYER_COUNT,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size),
                rows_per_image: Some(size),
            },
            wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: Self::LAYER_COUNT,
            },
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture_view,
            sampler,
        }
    }

    fn is_ore_fleck(x: u32, y: u32, seed: u32) -> bool {
        if x < 2 || x > 13 || y < 2 || y > 13 {
            return false;
        }
        let h = (x.wrapping_mul(37 + seed) ^ y.wrapping_mul(91 + seed)).wrapping_add(seed * 13);
        (h % 11 == 0) || (h % 17 == 0 && x % 2 == 1)
    }

    fn tier_color(tier: u32, noise: i32) -> [u8; 4] {
        match tier {
            0 => {
                // Wood (warm oak brown)
                [
                    (168 + noise * 3).clamp(120, 200) as u8,
                    (126 + noise * 2).clamp(85, 155) as u8,
                    (72 + noise * 2).clamp(45, 100) as u8,
                    255,
                ]
            }
            1 => {
                // Stone (cobble grey)
                let v = (145 + noise * 4).clamp(105, 180) as u8;
                [v, v, v, 255]
            }
            2 => {
                // Iron (bright metallic silver)
                let v = (225 + noise * 3).clamp(185, 250) as u8;
                [v, v, (v as i32 + 5).min(255) as u8, 255]
            }
            _ => {
                // Diamond (brilliant cyan/aqua)
                [
                    (70 + noise * 3).clamp(40, 110) as u8,
                    (235 + noise * 2).clamp(195, 255) as u8,
                    (225 + noise * 2).clamp(190, 255) as u8,
                    255,
                ]
            }
        }
    }

    fn generate_pixel(layer: u32, x: u32, y: u32) -> [u8; 4] {
        let noise = ((x.wrapping_mul(31) ^ y.wrapping_mul(97)) % 17) as i32 - 8;
        let noise2 = ((x.wrapping_mul(73) ^ y.wrapping_mul(41)) % 11) as i32 - 5;

        match layer {
            0 => {
                // Grass Top
                let g = (145 + noise * 4).clamp(100, 190) as u8;
                let r = (90 + noise * 3).clamp(60, 120) as u8;
                let b = (45 + noise * 2).clamp(30, 70) as u8;
                [r, g, b, 255]
            }
            1 => {
                // Grass Side
                let overhang = y < 4 || (y == 4 && (x + y) % 2 == 0);
                if overhang {
                    let g = (145 + noise * 4).clamp(100, 190) as u8;
                    let r = (90 + noise * 3).clamp(60, 120) as u8;
                    let b = (45 + noise * 2).clamp(30, 70) as u8;
                    [r, g, b, 255]
                } else {
                    let r = (130 + noise * 4).clamp(90, 160) as u8;
                    let g = (85 + noise * 3).clamp(55, 115) as u8;
                    let b = (50 + noise * 2).clamp(30, 75) as u8;
                    [r, g, b, 255]
                }
            }
            2 => {
                // Dirt
                let r = (134 + noise * 4).clamp(90, 165) as u8;
                let g = (88 + noise * 3).clamp(55, 120) as u8;
                let b = (52 + noise * 2).clamp(30, 80) as u8;
                [r, g, b, 255]
            }
            3 => {
                // Stone
                let v = (128 + noise * 4 + noise2 * 3).clamp(90, 160) as u8;
                [v, v, v, 255]
            }
            4 => {
                // Cobblestone
                let is_border = x % 4 == 0 || y % 4 == 0;
                let base = if is_border { 85 } else { 135 };
                let v = (base + noise * 5).clamp(60, 165) as u8;
                [v, v, v, 255]
            }
            5 => {
                // Oak Wood Log Side
                let bark = if x % 3 == 0 { -15 } else { 10 };
                let r = (110 + bark + noise * 3).clamp(70, 140) as u8;
                let g = (78 + bark + noise * 2).clamp(50, 105) as u8;
                let b = (48 + bark + noise).clamp(30, 70) as u8;
                [r, g, b, 255]
            }
            6 => {
                // Wood Log Top
                let dx = x as i32 - 8;
                let dy = y as i32 - 8;
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                let ring = ((dist * 1.5) as i32 % 2 == 0) as i32 * 15;
                let r = (175 - ring + noise * 3).clamp(120, 200) as u8;
                let g = (140 - ring + noise * 2).clamp(95, 165) as u8;
                let b = (95 - ring + noise).clamp(60, 125) as u8;
                [r, g, b, 255]
            }
            7 => {
                // Leaves (semi-transparent cutouts)
                let cutout = (x.wrapping_mul(17) ^ y.wrapping_mul(31)) % 7 == 0;
                if cutout {
                    [0, 0, 0, 0]
                } else {
                    let g = (140 + noise * 6).clamp(80, 185) as u8;
                    let r = (45 + noise * 2).clamp(25, 75) as u8;
                    let b = (30 + noise * 2).clamp(15, 55) as u8;
                    [r, g, b, 255]
                }
            }
            8 => {
                // Sand
                let r = (218 + noise * 3).clamp(180, 240) as u8;
                let g = (205 + noise * 3).clamp(170, 225) as u8;
                let b = (155 + noise * 2).clamp(125, 180) as u8;
                [r, g, b, 255]
            }
            9 => {
                // Planks
                let is_plank_gap = y % 4 == 0;
                let gap_penalty = if is_plank_gap { -30 } else { 0 };
                let r = (175 + gap_penalty + noise * 3).clamp(110, 205) as u8;
                let g = (135 + gap_penalty + noise * 2).clamp(85, 160) as u8;
                let b = (85 + gap_penalty + noise).clamp(50, 110) as u8;
                [r, g, b, 255]
            }
            10 => {
                // Bedrock
                let v = (50 + noise * 5 + noise2 * 5).clamp(20, 90) as u8;
                [v, v, v, 255]
            }
            11 => {
                // Glass
                let is_frame = x == 0 || x == 15 || y == 0 || y == 15 || (x == 4 && y == 4) || (x == 11 && y == 11);
                if is_frame {
                    [220, 235, 255, 200]
                } else {
                    [255, 255, 255, 20]
                }
            }
            12 => {
                // Water (translucent blue with wave highlights)
                let wave = ((x + y * 2) % 6 == 0) as i32 * 25;
                let r = (35 + wave + noise).clamp(20, 80) as u8;
                let g = (95 + wave + noise * 2).clamp(65, 145) as u8;
                let b = (215 + wave + noise * 2).clamp(170, 250) as u8;
                [r, g, b, 175]
            }
            13 => {
                // Lava (glowing magma orange/yellow/red)
                let swirl = ((x.wrapping_mul(3) + y.wrapping_mul(5)) % 7) as i32;
                let r = (235 + noise * 2).clamp(200, 255) as u8;
                let g = (95 + swirl * 18 + noise * 4).clamp(50, 220) as u8;
                let b = (15 + swirl * 3).clamp(5, 50) as u8;
                [r, g, b, 245]
            }
            14 => {
                // Coal Ore
                if Self::is_ore_fleck(x, y, 1) {
                    let v = (28 + noise * 2).clamp(12, 45) as u8;
                    [v, v, v, 255]
                } else {
                    Self::generate_pixel(3, x, y)
                }
            }
            15 => {
                // Iron Ore
                if Self::is_ore_fleck(x, y, 2) {
                    [
                        (210 + noise * 3).clamp(175, 235) as u8,
                        (170 + noise * 3).clamp(140, 200) as u8,
                        (140 + noise * 2).clamp(115, 165) as u8,
                        255,
                    ]
                } else {
                    Self::generate_pixel(3, x, y)
                }
            }
            16 => {
                // Gold Ore
                if Self::is_ore_fleck(x, y, 3) {
                    [
                        (250 + noise).clamp(220, 255) as u8,
                        (215 + noise * 3).clamp(180, 245) as u8,
                        (45 + noise * 2).clamp(20, 80) as u8,
                        255,
                    ]
                } else {
                    Self::generate_pixel(3, x, y)
                }
            }
            17 => {
                // Diamond Ore
                if Self::is_ore_fleck(x, y, 4) {
                    [
                        (85 + noise * 3).clamp(50, 130) as u8,
                        (240 + noise * 2).clamp(200, 255) as u8,
                        (235 + noise * 2).clamp(195, 255) as u8,
                        255,
                    ]
                } else {
                    Self::generate_pixel(3, x, y)
                }
            }
            18 => {
                // Gravel
                let pebble = ((x * 19 ^ y * 47) % 5 == 0) as i32 * 22;
                let r = (132 - pebble + noise * 5).clamp(80, 165) as u8;
                let g = (126 - pebble + noise * 5).clamp(75, 160) as u8;
                let b = (122 - pebble + noise * 5).clamp(70, 155) as u8;
                [r, g, b, 255]
            }
            19 => {
                // Snow Top
                let v = (242 + noise * 2).clamp(220, 255) as u8;
                [v, v, 255, 255]
            }
            20 => {
                // Snow Side (Dirt with snow overhang)
                if y < 5 || (y == 5 && (x + y) % 2 == 0) {
                    Self::generate_pixel(19, x, y)
                } else {
                    Self::generate_pixel(2, x, y)
                }
            }
            21 => {
                // Ice
                let streak = ((x + y) % 7 == 0) as i32 * 25;
                [
                    (140 + streak + noise * 2).clamp(110, 200) as u8,
                    (190 + streak + noise * 2).clamp(155, 235) as u8,
                    (245 + streak).clamp(210, 255) as u8,
                    195,
                ]
            }
            22 => {
                // Cactus Side
                let rib = if x % 4 == 0 { -22 } else { 8 };
                let spine = (x % 4 == 0 && y % 5 == 2) as i32;
                if spine != 0 {
                    [30, 45, 20, 255]
                } else {
                    [
                        (45 + rib + noise * 2).clamp(20, 85) as u8,
                        (135 + rib + noise * 3).clamp(85, 175) as u8,
                        (40 + rib + noise * 2).clamp(15, 75) as u8,
                        255,
                    ]
                }
            }
            23 => {
                // Cactus Top
                let border = x < 2 || x > 13 || y < 2 || y > 13;
                let g = if border { 115 } else { 150 };
                [50, (g + noise * 3).clamp(90, 180) as u8, 45, 255]
            }
            24 => {
                // Crafting Table Top (3x3 grid on wood)
                let border = x < 2 || x > 13 || y < 2 || y > 13;
                let grid = x == 5 || x == 10 || y == 5 || y == 10;
                if border {
                    [105, 72, 42, 255]
                } else if grid {
                    [75, 48, 28, 255]
                } else {
                    Self::generate_pixel(9, x, y)
                }
            }
            25 => {
                // Crafting Table Side
                if y < 4 {
                    [115, 78, 45, 255]
                } else if (x >= 4 && x <= 7 && y >= 6 && y <= 12) || (x >= 10 && x <= 12 && y >= 5 && y <= 11) {
                    // Tool silhouette on side
                    let v = (90 + noise * 3).clamp(60, 120) as u8;
                    [v, v, (v + 10).min(255), 255]
                } else {
                    Self::generate_pixel(9, x, y)
                }
            }
            26 => {
                // Furnace Front
                let in_hole = x >= 4 && x <= 11 && ((y >= 3 && y <= 6) || (y >= 9 && y <= 13));
                if in_hole {
                    if y >= 11 {
                        // Glowing embers
                        [240, (110 + noise * 8).clamp(60, 180) as u8, 20, 255]
                    } else {
                        [30, 30, 35, 255]
                    }
                } else {
                    Self::generate_pixel(4, x, y)
                }
            }
            27 => {
                // Furnace Side/Top
                let border = x == 0 || x == 15 || y == 0 || y == 15;
                let v = if border { 85 } else { 125 + noise * 4 };
                let c = v.clamp(65, 160) as u8;
                [c, c, c, 255]
            }
            28 => {
                // Torch (wooden stick + bright flame in center, transparent background)
                if x >= 7 && x <= 8 && y >= 3 && y <= 14 {
                    if y <= 5 {
                        // Flame tip
                        let glow = if y == 3 { [255, 250, 150, 255] } else { [255, 175, 40, 255] };
                        glow
                    } else {
                        // Wooden torch stick
                        [150, 110, 60, 255]
                    }
                } else {
                    [0, 0, 0, 0]
                }
            }
            29 => {
                // Brick
                let row = y / 4;
                let mortar_y = y % 4 == 0;
                let mortar_x = (x + (row % 2) * 4) % 8 == 0;
                if mortar_y || mortar_x {
                    let m = (185 + noise * 2).clamp(160, 205) as u8;
                    [m, m, (m - 5), 255]
                } else {
                    [
                        (165 + noise * 4).clamp(130, 195) as u8,
                        (72 + noise * 3).clamp(50, 95) as u8,
                        (58 + noise * 2).clamp(40, 80) as u8,
                        255,
                    ]
                }
            }
            30 => {
                // TNT Side
                if y >= 5 && y <= 10 {
                    // White label band with dark TNT letters
                    let letter = (y >= 6 && y <= 9) && (x == 3 || x == 7 || x == 8 || x == 12);
                    if letter {
                        [35, 35, 35, 255]
                    } else {
                        [235, 235, 235, 255]
                    }
                } else {
                    let stripe = if x % 4 == 0 { -25 } else { 5 };
                    [
                        (205 + stripe + noise * 3).clamp(140, 235) as u8,
                        (42 + noise * 2).clamp(20, 70) as u8,
                        (32 + noise * 2).clamp(15, 60) as u8,
                        255,
                    ]
                }
            }
            31 => {
                // TNT Top
                if x >= 6 && x <= 9 && y >= 6 && y <= 9 {
                    [45, 45, 45, 255] // Fuse
                } else {
                    [195, 45, 35, 255]
                }
            }
            32 => {
                // Obsidian
                let r = (32 + noise * 2).clamp(15, 55) as u8;
                let g = (22 + noise * 2).clamp(10, 40) as u8;
                let b = (52 + noise * 3 + noise2 * 2).clamp(25, 85) as u8;
                [r, g, b, 255]
            }
            33 => {
                // Glowstone
                let crystal = ((x * 23 ^ y * 53) % 4 == 0) as i32 * 35;
                [
                    (225 + crystal + noise * 2).clamp(180, 255) as u8,
                    (185 + crystal + noise * 3).clamp(135, 245) as u8,
                    (85 + crystal).clamp(50, 160) as u8,
                    255,
                ]
            }
            34 => {
                // Bookshelf Side
                if y < 2 || y > 13 || (y >= 7 && y <= 8) {
                    Self::generate_pixel(9, x, y)
                } else {
                    let book_col = (x / 2) % 4;
                    match book_col {
                        0 => [165, 45, 45, 255],
                        1 => [45, 125, 65, 255],
                        2 => [50, 85, 165, 255],
                        _ => [175, 145, 55, 255],
                    }
                }
            }
            35 => {
                // Birch Wood Side
                let notch = (y % 5 == 0 && (x + y) % 3 == 0) || (x == 0 || x == 15);
                if notch {
                    [65, 62, 58, 255]
                } else {
                    let v = (228 + noise * 2).clamp(200, 245) as u8;
                    [v, (v - 4), (v - 10), 255]
                }
            }
            36 => {
                // Spruce Wood Side
                let bark = if x % 3 == 0 { -12 } else { 6 };
                [
                    (68 + bark + noise * 2).clamp(40, 95) as u8,
                    (46 + bark + noise * 2).clamp(25, 70) as u8,
                    (28 + bark + noise).clamp(15, 50) as u8,
                    255,
                ]
            }
            37 => {
                // Stick Icon (diagonal wooden rod)
                let d = (x as i32) - (15 - y as i32);
                if d.abs() <= 1 && x >= 3 && x <= 12 && y >= 3 && y <= 12 {
                    Self::tier_color(0, noise)
                } else {
                    [0, 0, 0, 0]
                }
            }
            38 => {
                // Coal Lump Icon
                let dx = x as i32 - 8;
                let dy = y as i32 - 8;
                if dx * dx + dy * dy <= 24 {
                    let v = (38 + noise * 3).clamp(18, 65) as u8;
                    [v, v, v, 255]
                } else {
                    [0, 0, 0, 0]
                }
            }
            39 => {
                // Iron Ingot Icon
                if x >= 3 && x <= 12 && y >= 5 && y <= 10 {
                    Self::tier_color(2, noise)
                } else {
                    [0, 0, 0, 0]
                }
            }
            40 => {
                // Gold Ingot Icon
                if x >= 3 && x <= 12 && y >= 5 && y <= 10 {
                    [
                        (250 + noise).clamp(215, 255) as u8,
                        (210 + noise * 3).clamp(175, 240) as u8,
                        (45 + noise * 2).clamp(20, 80) as u8,
                        255,
                    ]
                } else {
                    [0, 0, 0, 0]
                }
            }
            41 => {
                // Diamond Gem Icon
                let dx = (x as i32 - 8).abs();
                let dy = (y as i32 - 8).abs();
                if dx + dy <= 5 {
                    Self::tier_color(3, noise)
                } else {
                    [0, 0, 0, 0]
                }
            }
            42..=45 => {
                // Pickaxes (Wood=42, Stone=43, Iron=44, Diamond=45)
                let tier = layer - 42;
                let on_handle = ((x as i32) - (15 - y as i32)).abs() <= 1 && x >= 2 && x <= 11 && y >= 4 && y <= 13;
                let on_head = (y >= 2 && y <= 4 && x >= 7 && x <= 13)
                    || (x >= 11 && x <= 13 && y >= 2 && y <= 8);
                if on_head {
                    Self::tier_color(tier, noise)
                } else if on_handle {
                    Self::tier_color(0, noise)
                } else {
                    [0, 0, 0, 0]
                }
            }
            46..=49 => {
                // Axes (Wood=46, Stone=47, Iron=48, Diamond=49)
                let tier = layer - 46;
                let on_handle = ((x as i32) - (15 - y as i32)).abs() <= 1 && x >= 2 && x <= 11 && y >= 4 && y <= 13;
                let on_blade = x >= 8 && x <= 13 && y >= 2 && y <= 7 && (x + y <= 18);
                if on_blade {
                    Self::tier_color(tier, noise)
                } else if on_handle {
                    Self::tier_color(0, noise)
                } else {
                    [0, 0, 0, 0]
                }
            }
            50..=53 => {
                // Shovels (Wood=50, Stone=51, Iron=52, Diamond=53)
                let tier = layer - 50;
                let on_handle = ((x as i32) - (15 - y as i32)).abs() <= 1 && x >= 2 && x <= 10 && y >= 5 && y <= 13;
                let on_spade = x >= 9 && x <= 13 && y >= 2 && y <= 6;
                if on_spade {
                    Self::tier_color(tier, noise)
                } else if on_handle {
                    Self::tier_color(0, noise)
                } else {
                    [0, 0, 0, 0]
                }
            }
            54..=57 => {
                // Swords (Wood=54, Stone=55, Iron=56, Diamond=57)
                let tier = layer - 54;
                let diag = (x as i32) - (15 - y as i32);
                let on_blade = diag.abs() <= 1 && x >= 6 && x <= 13 && y >= 2 && y <= 9;
                let on_guard = (x + y >= 12 && x + y <= 14) && (x >= 3 && x <= 8 && y >= 7 && y <= 12);
                let on_hilt = diag.abs() <= 1 && x >= 2 && x <= 5 && y >= 10 && y <= 13;
                if on_blade {
                    Self::tier_color(tier, noise)
                } else if on_guard || on_hilt {
                    Self::tier_color(0, noise)
                } else {
                    [0, 0, 0, 0]
                }
            }
            58 => {
                // Apple Icon
                let dx = x as i32 - 8;
                let dy = y as i32 - 9;
                if x == 8 && (y == 3 || y == 4) {
                    [60, 165, 50, 255] // Green leaf stem
                } else if dx * dx + dy * dy <= 20 {
                    [
                        (215 + noise * 3).clamp(170, 245) as u8,
                        (35 + noise * 2).clamp(15, 65) as u8,
                        (35 + noise * 2).clamp(15, 65) as u8,
                        255,
                    ]
                } else {
                    [0, 0, 0, 0]
                }
            }
            59 => {
                // Bread Icon
                if x >= 3 && x <= 12 && y >= 6 && y <= 11 {
                    let crust = y <= 7;
                    let r = if crust { 185 } else { 220 };
                    let g = if crust { 125 } else { 175 };
                    [
                        (r + noise * 2).clamp(140, 240) as u8,
                        (g + noise * 2).clamp(100, 200) as u8,
                        65,
                        255,
                    ]
                } else {
                    [0, 0, 0, 0]
                }
            }
            60 => {
                // Raw Porkchop Icon
                let dx = x as i32 - 8;
                let dy = y as i32 - 8;
                if dx * dx + dy * dy <= 22 {
                    [
                        (230 + noise * 2).clamp(195, 250) as u8,
                        (135 + noise * 3).clamp(105, 165) as u8,
                        (135 + noise * 3).clamp(105, 165) as u8,
                        255,
                    ]
                } else {
                    [0, 0, 0, 0]
                }
            }
            61 => {
                // Cooked Porkchop Icon
                let dx = x as i32 - 8;
                let dy = y as i32 - 8;
                if dx * dx + dy * dy <= 22 {
                    [
                        (195 + noise * 3).clamp(155, 225) as u8,
                        (115 + noise * 3).clamp(85, 145) as u8,
                        (65 + noise * 2).clamp(40, 95) as u8,
                        255,
                    ]
                } else {
                    [0, 0, 0, 0]
                }
            }
            62 => {
                // Rotten Flesh Icon
                let dx = x as i32 - 8;
                let dy = y as i32 - 8;
                if dx * dx + dy * dy <= 20 {
                    [
                        (135 + noise * 3).clamp(95, 165) as u8,
                        (95 + noise * 3).clamp(65, 130) as u8,
                        (55 + noise * 2).clamp(35, 85) as u8,
                        255,
                    ]
                } else {
                    [0, 0, 0, 0]
                }
            }
            _ => [255, 255, 255, 255],
        }
    }
}
