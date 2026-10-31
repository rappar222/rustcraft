pub struct TextureManager {
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl TextureManager {
    pub const TEX_SIZE: u32 = 16;
    pub const LAYER_COUNT: u32 = 12;

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
                let overhang = if y < 4 || (y == 4 && (x + y) % 2 == 0) { true } else { false };
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
                // Wood Log Side
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
                // Glass (clear pane with frosted border)
                let is_frame = x == 0 || x == 15 || y == 0 || y == 15 || (x == 4 && y == 4) || (x == 11 && y == 11);
                if is_frame {
                    [220, 235, 255, 200]
                } else {
                    [255, 255, 255, 20]
                }
            }
            _ => [255, 255, 255, 255],
        }
    }
}
