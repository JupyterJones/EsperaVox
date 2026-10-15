use glam::IVec3;
use crate::voxel::{VoxelCoord, VoxelId, EMPTY_VOXEL};

/// A dense 3D voxel grid volume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoxelGrid {
    pub size_x: u32,
    pub size_y: u32,
    pub size_z: u32,
    data: Vec<VoxelId>,
}

impl VoxelGrid {
    /// Creates a new empty voxel grid with specified dimensions.
    pub fn new(size_x: u32, size_y: u32, size_z: u32) -> Self {
        let total_voxels = (size_x as usize) * (size_y as usize) * (size_z as usize);
        Self {
            size_x,
            size_y,
            size_z,
            data: vec![EMPTY_VOXEL; total_voxels],
        }
    }

    /// Default cubic grid of 32x32x32.
    pub fn default_cube() -> Self {
        Self::new(32, 32, 32)
    }

    #[inline]
    pub fn is_in_bounds(&self, x: i32, y: i32, z: i32) -> bool {
        x >= 0 && x < self.size_x as i32 &&
        y >= 0 && y < self.size_y as i32 &&
        z >= 0 && z < self.size_z as i32
    }

    #[inline]
    fn to_index(&self, x: u32, y: u32, z: u32) -> usize {
        (x + y * self.size_x + z * self.size_x * self.size_y) as usize
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32, z: i32) -> VoxelId {
        if self.is_in_bounds(x, y, z) {
            self.data[self.to_index(x as u32, y as u32, z as u32)]
        } else {
            EMPTY_VOXEL
        }
    }

    #[inline]
    pub fn get_coord(&self, coord: VoxelCoord) -> VoxelId {
        self.get(coord.x, coord.y, coord.z)
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, z: i32, id: VoxelId) -> bool {
        if self.is_in_bounds(x, y, z) {
            let idx = self.to_index(x as u32, y as u32, z as u32);
            self.data[idx] = id;
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn set_coord(&mut self, coord: VoxelCoord, id: VoxelId) -> bool {
        self.set(coord.x, coord.y, coord.z, id)
    }

    /// Clears all voxels to empty.
    pub fn clear(&mut self) {
        self.data.fill(EMPTY_VOXEL);
    }

    /// Returns the number of non-empty voxels.
    pub fn count_active(&self) -> usize {
        self.data.iter().filter(|&&v| v != EMPTY_VOXEL).count()
    }

    /// Fills an axis-aligned box with the given voxel id.
    pub fn fill_box(&mut self, min: IVec3, max: IVec3, id: VoxelId) {
        let x_start = min.x.min(max.x);
        let x_end = min.x.max(max.x);
        let y_start = min.y.min(max.y);
        let y_end = min.y.max(max.y);
        let z_start = min.z.min(max.z);
        let z_end = min.z.max(max.z);

        for z in z_start..=z_end {
            for y in y_start..=y_end {
                for x in x_start..=x_end {
                    self.set(x, y, z, id);
                }
            }
        }
    }

    /// Computes the minimal bounding box containing all active voxels.
    pub fn bounding_box(&self) -> Option<(IVec3, IVec3)> {
        let mut min = IVec3::new(i32::MAX, i32::MAX, i32::MAX);
        let mut max = IVec3::new(i32::MIN, i32::MIN, i32::MIN);
        let mut has_voxels = false;

        for z in 0..self.size_z as i32 {
            for y in 0..self.size_y as i32 {
                for x in 0..self.size_x as i32 {
                    if self.get(x, y, z) != EMPTY_VOXEL {
                        has_voxels = true;
                        min = min.min(IVec3::new(x, y, z));
                        max = max.max(IVec3::new(x, y, z));
                    }
                }
            }
        }

        if has_voxels {
            Some((min, max))
        } else {
            None
        }
    }

    /// Returns the dimensions as an `IVec3`.
    #[inline]
    pub fn dimensions(&self) -> IVec3 {
        IVec3::new(self.size_x as i32, self.size_y as i32, self.size_z as i32)
    }
}
