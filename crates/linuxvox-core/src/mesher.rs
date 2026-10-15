use glam::{IVec3, Vec3};
use crate::grid::VoxelGrid;
use crate::palette::Palette;
use crate::voxel::{Direction, EMPTY_VOXEL};

/// A single 3D vertex in the generated mesh.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
    pub palette_id: u8,
}

/// A standard indexed triangle mesh.
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() || self.indices.is_empty()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Exports this mesh to standard Wavefront OBJ format.
    pub fn to_obj(&self) -> String {
        let mut obj = String::new();
        obj.push_str("# EsperaVox OBJ Export\n");

        for v in &self.vertices {
            obj.push_str(&format!(
                "v {} {} {} {} {} {}\n",
                v.position[0], v.position[1], v.position[2],
                v.color[0], v.color[1], v.color[2]
            ));
        }

        for v in &self.vertices {
            obj.push_str(&format!(
                "vn {} {} {}\n",
                v.normal[0], v.normal[1], v.normal[2]
            ));
        }

        for chunk in self.indices.chunks_exact(3) {
            // OBJ indices are 1-based
            let i0 = chunk[0] + 1;
            let i1 = chunk[1] + 1;
            let i2 = chunk[2] + 1;
            obj.push_str(&format!("f {}//{} {}//{} {}//{}\n", i0, i0, i1, i1, i2, i2));
        }

        obj
    }
}

/// Generates a face-culled mesh from a `VoxelGrid`.
/// Only outward-facing surfaces adjacent to empty air are emitted.
pub fn generate_culled_mesh(grid: &VoxelGrid, palette: &Palette) -> Mesh {
    let mut mesh = Mesh::default();

    for z in 0..grid.size_z as i32 {
        for y in 0..grid.size_y as i32 {
            for x in 0..grid.size_x as i32 {
                let id = grid.get(x, y, z);
                if id == EMPTY_VOXEL {
                    continue;
                }

                let color = palette.get(id).to_normalized();
                let voxel_pos = IVec3::new(x, y, z);

                // Check 6 directions
                for dir in Direction::ALL {
                    let neighbor_pos = voxel_pos + dir.normal();
                    let neighbor_id = grid.get(neighbor_pos.x, neighbor_pos.y, neighbor_pos.z);

                    // If neighbor is air, this face is visible!
                    if neighbor_id == EMPTY_VOXEL {
                        add_face(&mut mesh, voxel_pos, dir, color, id);
                    }
                }
            }
        }
    }

    mesh
}

fn add_face(mesh: &mut Mesh, pos: IVec3, dir: Direction, color: [f32; 4], palette_id: u8) {
    let base_idx = mesh.vertices.len() as u32;
    let normal = dir.normal().as_vec3();
    let norm_arr = [normal.x, normal.y, normal.z];

    let p = pos.as_vec3();

    // 4 corners of the unit cube [0, 1]^3 for each direction
    let corners: [Vec3; 4] = match dir {
        Direction::PositiveX => [
            p + Vec3::new(1.0, 0.0, 0.0),
            p + Vec3::new(1.0, 1.0, 0.0),
            p + Vec3::new(1.0, 1.0, 1.0),
            p + Vec3::new(1.0, 0.0, 1.0),
        ],
        Direction::NegativeX => [
            p + Vec3::new(0.0, 0.0, 1.0),
            p + Vec3::new(0.0, 1.0, 1.0),
            p + Vec3::new(0.0, 1.0, 0.0),
            p + Vec3::new(0.0, 0.0, 0.0),
        ],
        Direction::PositiveY => [
            p + Vec3::new(0.0, 1.0, 0.0),
            p + Vec3::new(0.0, 1.0, 1.0),
            p + Vec3::new(1.0, 1.0, 1.0),
            p + Vec3::new(1.0, 1.0, 0.0),
        ],
        Direction::NegativeY => [
            p + Vec3::new(0.0, 0.0, 1.0),
            p + Vec3::new(0.0, 0.0, 0.0),
            p + Vec3::new(1.0, 0.0, 0.0),
            p + Vec3::new(1.0, 0.0, 1.0),
        ],
        Direction::PositiveZ => [
            p + Vec3::new(0.0, 0.0, 1.0),
            p + Vec3::new(1.0, 0.0, 1.0),
            p + Vec3::new(1.0, 1.0, 1.0),
            p + Vec3::new(0.0, 1.0, 1.0),
        ],
        Direction::NegativeZ => [
            p + Vec3::new(1.0, 0.0, 0.0),
            p + Vec3::new(0.0, 0.0, 0.0),
            p + Vec3::new(0.0, 1.0, 0.0),
            p + Vec3::new(1.0, 1.0, 0.0),
        ],
    };

    for corner in &corners {
        mesh.vertices.push(Vertex {
            position: [corner.x, corner.y, corner.z],
            normal: norm_arr,
            color,
            palette_id,
        });
    }

    // Two triangles forming the quad: (0, 1, 2) and (0, 2, 3)
    mesh.indices.extend_from_slice(&[
        base_idx,
        base_idx + 1,
        base_idx + 2,
        base_idx,
        base_idx + 2,
        base_idx + 3,
    ]);
}
