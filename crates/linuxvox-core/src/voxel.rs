use glam::IVec3;

/// A 3D coordinate for a voxel in grid space.
pub type VoxelCoord = IVec3;

/// 8-bit palette index for a voxel.
/// `0` denotes an empty voxel (air), while `1..=255` map to palette colors.
pub type VoxelId = u8;

pub const EMPTY_VOXEL: VoxelId = 0;

/// The six cardinal 3D face directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl Direction {
    pub const ALL: [Direction; 6] = [
        Direction::PositiveX,
        Direction::NegativeX,
        Direction::PositiveY,
        Direction::NegativeY,
        Direction::PositiveZ,
        Direction::NegativeZ,
    ];

    #[inline]
    pub fn normal(&self) -> IVec3 {
        match self {
            Direction::PositiveX => IVec3::new(1, 0, 0),
            Direction::NegativeX => IVec3::new(-1, 0, 0),
            Direction::PositiveY => IVec3::new(0, 1, 0),
            Direction::NegativeY => IVec3::new(0, -1, 0),
            Direction::PositiveZ => IVec3::new(0, 0, 1),
            Direction::NegativeZ => IVec3::new(0, 0, -1),
        }
    }

    #[inline]
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::PositiveX => Direction::NegativeX,
            Direction::NegativeX => Direction::PositiveX,
            Direction::PositiveY => Direction::NegativeY,
            Direction::NegativeY => Direction::PositiveY,
            Direction::PositiveZ => Direction::NegativeZ,
            Direction::NegativeZ => Direction::PositiveZ,
        }
    }
}
