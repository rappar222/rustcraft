use crate::voxel::world::World;
use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RaycastHit {
    pub block_pos: (i32, i32, i32),
    pub face_normal: (i32, i32, i32),
    pub adjacent_pos: (i32, i32, i32),
}

pub fn raycast_voxels(
    origin: Vec3,
    direction: Vec3,
    max_dist: f32,
    world: &World,
) -> Option<RaycastHit> {
    let dir_len = direction.length();
    if dir_len < 1e-6 {
        return None;
    }
    let dir = direction / dir_len;

    let mut current_x = origin.x.floor() as i32;
    let mut current_y = origin.y.floor() as i32;
    let mut current_z = origin.z.floor() as i32;

    let step_x = if dir.x > 0.0 { 1 } else if dir.x < 0.0 { -1 } else { 0 };
    let step_y = if dir.y > 0.0 { 1 } else if dir.y < 0.0 { -1 } else { 0 };
    let step_z = if dir.z > 0.0 { 1 } else if dir.z < 0.0 { -1 } else { 0 };

    let inv_dx = if dir.x.abs() > 1e-6 { 1.0 / dir.x.abs() } else { f32::MAX };
    let inv_dy = if dir.y.abs() > 1e-6 { 1.0 / dir.y.abs() } else { f32::MAX };
    let inv_dz = if dir.z.abs() > 1e-6 { 1.0 / dir.z.abs() } else { f32::MAX };

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
    let mut t_curr = 0.0;

    while t_curr <= max_dist {
        if world.get_block(current_x, current_y, current_z).is_targetable() {
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
                t_curr = t_max_x;
                current_x += step_x;
                t_max_x += inv_dx;
                normal = (-step_x, 0, 0);
            } else {
                t_curr = t_max_z;
                current_z += step_z;
                t_max_z += inv_dz;
                normal = (0, 0, -step_z);
            }
        } else if t_max_y < t_max_z {
            t_curr = t_max_y;
            current_y += step_y;
            t_max_y += inv_dy;
            normal = (0, -step_y, 0);
        } else {
            t_curr = t_max_z;
            current_z += step_z;
            t_max_z += inv_dz;
            normal = (0, 0, -step_z);
        }
    }

    None
}
