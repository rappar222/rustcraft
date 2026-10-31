use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [f32; 16],
    pub cam_pos: [f32; 4],
    pub fog_color: [f32; 4],
    pub fog_info: [f32; 4], // [min_dist, max_dist, 0.0, 0.0]
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

    pub fn to_uniform(&self, view_distance_blocks: f32, fog_enabled: bool) -> CameraUniform {
        let view_proj = self.build_view_projection_matrix();
        let (fog_min, fog_max) = if fog_enabled {
            (view_distance_blocks * 0.60, view_distance_blocks * 0.98)
        } else {
            (99999.0, 100000.0)
        };

        CameraUniform {
            view_proj: view_proj.to_cols_array(),
            cam_pos: [self.position.x, self.position.y, self.position.z, 1.0],
            fog_color: [0.65, 0.82, 1.0, 1.0], // Sky blue
            fog_info: [fog_min, fog_max, 0.0, 0.0],
        }
    }
}
