use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [f32; 16],
    pub cam_pos: [f32; 4],
    pub fog_color: [f32; 4],
    pub fog_info: [f32; 4], // [min_dist, max_dist, daylight_factor, is_underwater]
}

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,   // in radians
    pub pitch: f32, // in radians
    pub aspect: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera {
    pub fn new(position: Vec3, aspect: f32) -> Self {
        Self {
            position,
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
            aspect,
            fov_y: 75.0_f32.to_radians(),
            z_near: 0.1,
            z_far: 1000.0,
        }
    }

    pub fn set_fov_degrees(&mut self, fov_deg: f32) {
        self.fov_y = fov_deg.to_radians();
    }

    pub fn front(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    pub fn right(&self) -> Vec3 {
        self.front().cross(Vec3::Y).normalize()
    }

    pub fn up(&self) -> Vec3 {
        self.right().cross(self.front()).normalize()
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let front = self.front();
        let right = self.right();
        let up = self.up();
        let eye = self.position;

        // View matrix
        let view = Mat4::from_cols(
            glam::Vec4::new(right.x, up.x, -front.x, 0.0),
            glam::Vec4::new(right.y, up.y, -front.y, 0.0),
            glam::Vec4::new(right.z, up.z, -front.z, 0.0),
            glam::Vec4::new(-right.dot(eye), -up.dot(eye), front.dot(eye), 1.0),
        );

        // Vulkan zero-to-one right-handed perspective projection
        let f = 1.0 / (self.fov_y * 0.5).tan();
        let range_inv = 1.0 / (self.z_near - self.z_far);
        let proj = Mat4::from_cols(
            glam::Vec4::new(f / self.aspect, 0.0, 0.0, 0.0),
            glam::Vec4::new(0.0, f, 0.0, 0.0),
            glam::Vec4::new(0.0, 0.0, self.z_far * range_inv, -1.0),
            glam::Vec4::new(0.0, 0.0, self.z_near * self.z_far * range_inv, 0.0),
        );

        proj * view
    }

    /// Compute sky color and daylight factor (0.20..1.0) from `time_of_day` in 0.0..1.0
    /// (0.0 = Dawn, 0.25 = Noon, 0.5 = Dusk, 0.75 = Midnight)
    pub fn sky_atmosphere(time_of_day: f32, is_underwater: bool) -> ([f32; 4], f32) {
        let angle = time_of_day * std::f32::consts::TAU;
        let sun_height = angle.sin(); // +1 at noon (0.25), -1 at midnight (0.75)
        let daylight = ((sun_height + 0.25) / 1.25).clamp(0.20, 1.0);

        if is_underwater {
            let uw_b = 0.45 * daylight + 0.12;
            return ([0.04, 0.18 * daylight + 0.04, uw_b, 1.0], daylight * 0.85);
        }

        let sky_rgb = if sun_height > 0.15 {
            // Day
            let t = ((sun_height - 0.15) / 0.85).clamp(0.0, 1.0);
            [
                0.75 * (1.0 - t) + 0.56 * t,
                0.62 * (1.0 - t) + 0.78 * t,
                0.55 * (1.0 - t) + 0.98 * t,
            ]
        } else if sun_height > -0.18 {
            // Sunrise / Sunset golden-orange glow
            let t = ((sun_height + 0.18) / 0.33).clamp(0.0, 1.0);
            [
                0.10 * (1.0 - t) + 0.78 * t,
                0.10 * (1.0 - t) + 0.52 * t,
                0.22 * (1.0 - t) + 0.48 * t,
            ]
        } else {
            // Night sky
            [0.04, 0.06, 0.14]
        };

        ([sky_rgb[0], sky_rgb[1], sky_rgb[2], 1.0], daylight)
    }

    pub fn to_uniform(
        &self,
        view_distance_blocks: f32,
        fog_enabled: bool,
        time_of_day: f32,
        is_underwater: bool,
    ) -> CameraUniform {
        let view_proj = self.build_view_projection_matrix();
        let (fog_color, daylight) = Self::sky_atmosphere(time_of_day, is_underwater);

        let (fog_min, fog_max) = if is_underwater {
            (3.0, 26.0)
        } else if fog_enabled {
            (view_distance_blocks * 0.58, view_distance_blocks * 0.98)
        } else {
            (99999.0, 100000.0)
        };

        CameraUniform {
            view_proj: view_proj.to_cols_array(),
            cam_pos: [self.position.x, self.position.y, self.position.z, 1.0],
            fog_color,
            fog_info: [
                fog_min,
                fog_max,
                daylight,
                if is_underwater { 1.0 } else { 0.0 },
            ],
        }
    }
}
