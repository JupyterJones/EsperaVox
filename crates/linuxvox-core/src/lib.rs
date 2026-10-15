pub mod voxel;
pub mod grid;
pub mod palette;
pub mod raycast;
pub mod mesher;
pub mod vox_io;
pub mod history;
pub mod renderer;
pub mod cinematic;

pub use voxel::{Direction, VoxelCoord, VoxelId, EMPTY_VOXEL};
pub use grid::VoxelGrid;
pub use palette::{Color, Palette};
pub use raycast::{raycast_grid, RayHit};
pub use mesher::{generate_culled_mesh, Mesh, Vertex};
pub use vox_io::{read_vox, write_vox, VoxModel};
pub use history::{EditCommand, HistoryManager, VoxelDiff};
pub use renderer::{render_mesh_to_image, BackgroundMode, RenderOptions};
pub use cinematic::{CameraKeyframe, CameraTrack};

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{IVec3, Vec3};
    use std::io::Cursor;

    #[test]
    fn test_grid_operations() {
        let mut grid = VoxelGrid::new(16, 16, 16);
        assert_eq!(grid.count_active(), 0);

        grid.set(4, 5, 6, 42);
        assert_eq!(grid.get(4, 5, 6), 42);
        assert_eq!(grid.count_active(), 1);

        grid.clear();
        assert_eq!(grid.count_active(), 0);
    }

    #[test]
    fn test_raycast_3d_dda() {
        let mut grid = VoxelGrid::new(16, 16, 16);
        // Place a voxel at (8, 8, 8)
        grid.set(8, 8, 8, 10);

        // Cast a ray from outside looking straight towards (8, 8, 8) along +X
        let origin = Vec3::new(0.5, 8.5, 8.5);
        let dir = Vec3::new(1.0, 0.0, 0.0);

        let hit = raycast_grid(origin, dir, 20.0, &grid);
        assert!(hit.is_some());
        let hit = hit.unwrap();

        assert_eq!(hit.voxel_coord, IVec3::new(8, 8, 8));
        // Hit came from -X direction, so the face normal should point towards -X (-1, 0, 0)
        assert_eq!(hit.face_normal, IVec3::new(-1, 0, 0));
        assert_eq!(hit.voxel_id, 10);
    }

    #[test]
    fn test_mesher_single_voxel() {
        let mut grid = VoxelGrid::new(8, 8, 8);
        grid.set(2, 2, 2, 5);
        let palette = Palette::magica_default();

        let mesh = generate_culled_mesh(&grid, &palette);
        // A single isolated voxel has 6 visible faces = 6 * 2 = 12 triangles = 24 vertices
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.triangle_count(), 12);
    }

    #[test]
    fn test_vox_roundtrip() {
        let mut grid = VoxelGrid::new(10, 10, 10);
        grid.set(1, 2, 3, 55);
        grid.set(7, 8, 9, 88);
        let palette = Palette::magica_default();

        let mut buffer = Vec::new();
        write_vox(&mut buffer, &grid, &palette).expect("Failed to write .vox");

        let loaded = read_vox(Cursor::new(buffer)).expect("Failed to read .vox");
        assert_eq!(loaded.grid.size_x, 10);
        assert_eq!(loaded.grid.size_y, 10);
        assert_eq!(loaded.grid.size_z, 10);
        assert_eq!(loaded.grid.get(1, 2, 3), 55);
        assert_eq!(loaded.grid.get(7, 8, 9), 88);
        assert_eq!(loaded.grid.count_active(), 2);
    }

    #[test]
    fn test_undo_redo() {
        let mut grid = VoxelGrid::new(8, 8, 8);
        let mut history = HistoryManager::new(50);

        let coord = IVec3::new(2, 3, 4);
        grid.set_coord(coord, 15);
        history.record(EditCommand::new(
            "Place Voxel",
            vec![VoxelDiff {
                coord,
                old_id: EMPTY_VOXEL,
                new_id: 15,
            }],
        ));

        assert_eq!(grid.get_coord(coord), 15);
        assert!(history.can_undo());

        // Undo
        assert!(history.undo(&mut grid));
        assert_eq!(grid.get_coord(coord), EMPTY_VOXEL);
        assert!(history.can_redo());

        // Redo
        assert!(history.redo(&mut grid));
        assert_eq!(grid.get_coord(coord), 15);
    }
}
