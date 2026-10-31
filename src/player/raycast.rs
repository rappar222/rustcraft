use crate::voxel::world::World;
use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    pub block_pos: (i32, i32, i32),
    #[allow(dead_code)]
    pub face_normal: (i32, i32, i32),
    pub adjacent_pos: (i32, i32, i32),
}

pub fn raycast_voxels(origin: Vec3, direction: Vec3, max_dist: f32, world: &World) -> Option<RaycastHit> {
    let mut current_x = origin.x.floor() as i32;
    let mut current_y = origin.y.floor() as i32;
    let mut current_z = origin.z.floor() as i32;

    let step_x = if direction.x > 0.0 { 1 } else if direction.x < 0.0 { -1 } else { 0 };
    let step_y = if direction.y > 0.0 { 1 } else if direction.y < 0.0 { -1 } else { 0 };
    let step_z = if direction.z > 0.0 { 1 } else if direction.z < 0.0 { -1 } else { 0 };

    let inv_dx = if direction.x != 0.0 { 1.0 / direction.x.abs() } else { f32::MAX };
    let inv_dy = if direction.y != 0.0 { 1.0 / direction.y.abs() } else { f32::MAX };
    let inv_dz = if direction.z != 0.0 { 1.0 / direction.z.abs() } else { f32::MAX };

    let mut t_max_x = if step_x > 0 {
        ((current_x as f32 + 1.0) - origin.x) * inv_dx
    } else if step_x < 0 {
        (origin.x - current_x as f32) * inv_dx
    } else {
        f32::MAX
    };

    let mut t_max_y = if step_y > 0 {
        ((current_y as f32 + 1.0) - origin.y) * inv_dy
    } else if step_y < 0 {
        (origin.y - current_y as f32) * inv_dy
    } else {
        f32::MAX
    };

    let mut t_max_z = if step_z > 0 {
        ((current_z as f32 + 1.0) - origin.z) * inv_dz
    } else if step_z < 0 {
        (origin.z - current_z as f32) * inv_dz
    } else {
        f32::MAX
    };

    let mut normal = (0, 0, 0);

    while t_max_x.min(t_max_y).min(t_max_z) <= max_dist {
        if world.get_block(current_x, current_y, current_z).is_solid() {
            return Some(RaycastHit {
                block_pos: (current_x, current_y, current_z),
                face_normal: normal,
                adjacent_pos: (
                    current_x + normal.0,
                    current_y + normal.1,
                    current_z + normal.2,
                ),
            });
        }

        if t_max_x < t_max_y {
            if t_max_x < t_max_z {
                current_x += step_x;
                t_max_x += inv_dx;
                normal = (-step_x, 0, 0);
            } else {
                current_z += step_z;
                t_max_z += inv_dz;
                normal = (0, 0, -step_z);
            }
        } else {
            if t_max_y < t_max_z {
                current_y += step_y;
                t_max_y += inv_dy;
                normal = (0, -step_y, 0);
            } else {
                current_z += step_z;
                t_max_z += inv_dz;
                normal = (0, 0, -step_z);
            }
        }
    }

    None
}
