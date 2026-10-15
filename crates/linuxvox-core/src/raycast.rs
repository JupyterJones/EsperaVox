use glam::{IVec3, Vec3};
use crate::grid::VoxelGrid;
use crate::voxel::{VoxelCoord, VoxelId, EMPTY_VOXEL};

/// Result of a successful voxel raycast hit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    /// Coordinate of the voxel that was hit.
    pub voxel_coord: VoxelCoord,
    /// Outward-facing surface normal of the hit voxel face (e.g. `(0, 1, 0)`).
    /// Adding this to `voxel_coord` gives the placement coordinate for a new voxel.
    pub face_normal: IVec3,
    /// Distance from ray origin to the hit point.
    pub distance: f32,
    /// Palette ID of the hit voxel.
    pub voxel_id: VoxelId,
}

/// Casts a ray through the `VoxelGrid` using the 3D DDA (Digital Differential Analyzer) algorithm.
pub fn raycast_grid(
    origin: Vec3,
    dir: Vec3,
    max_distance: f32,
    grid: &VoxelGrid,
) -> Option<RayHit> {
    let dir = dir.normalize();

    // Bounds of the grid volume
    let box_min = Vec3::ZERO;
    let box_max = Vec3::new(
        grid.size_x as f32,
        grid.size_y as f32,
        grid.size_z as f32,
    );

    // Find intersection with the grid's AABB
    let (t_min, t_max) = intersect_aabb(origin, dir, box_min, box_max)?;

    if t_max < 0.0 || t_min > max_distance {
        return None;
    }

    // Start point inside or on the boundary of the box
    let t_start = t_min.max(0.0);
    let start_point = origin + dir * (t_start + 1e-4);

    let mut current_voxel = IVec3::new(
        start_point.x.floor() as i32,
        start_point.y.floor() as i32,
        start_point.z.floor() as i32,
    );

    // If starting inside a solid voxel directly
    if grid.is_in_bounds(current_voxel.x, current_voxel.y, current_voxel.z) {
        let id = grid.get_coord(current_voxel);
        if id != EMPTY_VOXEL {
            return Some(RayHit {
                voxel_coord: current_voxel,
                face_normal: -face_normal_from_dir(dir),
                distance: t_start,
                voxel_id: id,
            });
        }
    }

    let step_x = if dir.x > 0.0 { 1 } else { -1 };
    let step_y = if dir.y > 0.0 { 1 } else { -1 };
    let step_z = if dir.z > 0.0 { 1 } else { -1 };

    let inv_dir_x = if dir.x != 0.0 { 1.0 / dir.x.abs() } else { f32::MAX };
    let inv_dir_y = if dir.y != 0.0 { 1.0 / dir.y.abs() } else { f32::MAX };
    let inv_dir_z = if dir.z != 0.0 { 1.0 / dir.z.abs() } else { f32::MAX };

    let next_voxel_boundary_x = if dir.x > 0.0 {
        (current_voxel.x + 1) as f32
    } else {
        current_voxel.x as f32
    };
    let next_voxel_boundary_y = if dir.y > 0.0 {
        (current_voxel.y + 1) as f32
    } else {
        current_voxel.y as f32
    };
    let next_voxel_boundary_z = if dir.z > 0.0 {
        (current_voxel.z + 1) as f32
    } else {
        current_voxel.z as f32
    };

    let mut t_max_x = if dir.x != 0.0 {
        (next_voxel_boundary_x - origin.x) / dir.x
    } else {
        f32::MAX
    };
    let mut t_max_y = if dir.y != 0.0 {
        (next_voxel_boundary_y - origin.y) / dir.y
    } else {
        f32::MAX
    };
    let mut t_max_z = if dir.z != 0.0 {
        (next_voxel_boundary_z - origin.z) / dir.z
    } else {
        f32::MAX
    };

    #[allow(unused_assignments)]
    let mut last_normal = IVec3::ZERO;
    let mut total_dist = t_start;

    while total_dist <= max_distance {
        if t_max_x < t_max_y {
            if t_max_x < t_max_z {
                total_dist = t_max_x;
                current_voxel.x += step_x;
                t_max_x += inv_dir_x;
                last_normal = IVec3::new(-step_x, 0, 0);
            } else {
                total_dist = t_max_z;
                current_voxel.z += step_z;
                t_max_z += inv_dir_z;
                last_normal = IVec3::new(0, 0, -step_z);
            }
        } else {
            if t_max_y < t_max_z {
                total_dist = t_max_y;
                current_voxel.y += step_y;
                t_max_y += inv_dir_y;
                last_normal = IVec3::new(0, -step_y, 0);
            } else {
                total_dist = t_max_z;
                current_voxel.z += step_z;
                t_max_z += inv_dir_z;
                last_normal = IVec3::new(0, 0, -step_z);
            }
        }

        if !grid.is_in_bounds(current_voxel.x, current_voxel.y, current_voxel.z) {
            break;
        }

        let id = grid.get_coord(current_voxel);
        if id != EMPTY_VOXEL {
            return Some(RayHit {
                voxel_coord: current_voxel,
                face_normal: last_normal,
                distance: total_dist,
                voxel_id: id,
            });
        }
    }

    None
}

fn intersect_aabb(origin: Vec3, dir: Vec3, box_min: Vec3, box_max: Vec3) -> Option<(f32, f32)> {
    let mut t_min = f32::MIN;
    let mut t_max = f32::MAX;

    for i in 0..3 {
        let o = origin[i];
        let d = dir[i];
        let min_val = box_min[i];
        let max_val = box_max[i];

        if d.abs() < 1e-6 {
            if o < min_val || o > max_val {
                return None;
            }
        } else {
            let inv_d = 1.0 / d;
            let mut t1 = (min_val - o) * inv_d;
            let mut t2 = (max_val - o) * inv_d;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            t_min = t_min.max(t1);
            t_max = t_max.min(t2);
            if t_min > t_max {
                return None;
            }
        }
    }

    Some((t_min, t_max))
}

fn face_normal_from_dir(dir: Vec3) -> IVec3 {
    let abs_d = dir.abs();
    if abs_d.x > abs_d.y && abs_d.x > abs_d.z {
        IVec3::new(dir.x.signum() as i32, 0, 0)
    } else if abs_d.y > abs_d.z {
        IVec3::new(0, dir.y.signum() as i32, 0)
    } else {
        IVec3::new(0, 0, dir.z.signum() as i32)
    }
}
