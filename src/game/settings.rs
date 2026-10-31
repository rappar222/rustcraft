#[derive(Debug, Clone)]
pub struct GraphicsSettings {
    pub render_distance: i32, // 4, 8, 12, 16
    pub fov_degrees: f32,     // 60, 75, 90, 105
    pub fog_enabled: bool,
    pub vsync_enabled: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            render_distance: 8,
            fov_degrees: 75.0,
            fog_enabled: true,
            vsync_enabled: false,
        }
    }
}
