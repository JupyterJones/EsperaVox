use std::io::{self, Cursor, Read, Write};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crate::grid::VoxelGrid;
use crate::palette::{Color, Palette};
use crate::voxel::EMPTY_VOXEL;

const VOX_MAGIC: &[u8; 4] = b"VOX ";
const VOX_VERSION: u32 = 150;

/// Parsed `.vox` model data.
#[derive(Debug, Clone)]
pub struct VoxModel {
    pub grid: VoxelGrid,
    pub palette: Palette,
}

/// Reads a `.vox` file from any reader.
pub fn read_vox<R: Read>(mut reader: R) -> io::Result<VoxModel> {
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if &magic != VOX_MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid VOX magic header",
        ));
    }

    let _version = reader.read_u32::<LittleEndian>()?;

    let mut main_id = [0u8; 4];
    reader.read_exact(&mut main_id)?;
    if &main_id != b"MAIN" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Expected MAIN chunk",
        ));
    }

    let _main_content_len = reader.read_u32::<LittleEndian>()?;
    let main_children_len = reader.read_u32::<LittleEndian>()? as usize;

    let mut main_children_buf = vec![0u8; main_children_len];
    reader.read_exact(&mut main_children_buf)?;

    let mut cursor = Cursor::new(main_children_buf);
    let mut size_x = 32u32;
    let mut size_y = 32u32;
    let mut size_z = 32u32;
    let mut raw_voxels = Vec::new();
    let mut palette = Palette::magica_default();

    while (cursor.position() as usize) < cursor.get_ref().len() {
        let mut chunk_id = [0u8; 4];
        if cursor.read_exact(&mut chunk_id).is_err() {
            break;
        }

        let content_len = cursor.read_u32::<LittleEndian>()? as usize;
        let children_len = cursor.read_u32::<LittleEndian>()? as usize;

        let mut content = vec![0u8; content_len];
        cursor.read_exact(&mut content)?;

        match &chunk_id {
            b"SIZE" => {
                let mut c = Cursor::new(content);
                size_x = c.read_u32::<LittleEndian>()?;
                size_y = c.read_u32::<LittleEndian>()?;
                size_z = c.read_u32::<LittleEndian>()?;
            }
            b"XYZI" => {
                let mut c = Cursor::new(content);
                let num_voxels = c.read_u32::<LittleEndian>()?;
                for _ in 0..num_voxels {
                    let x = c.read_u8()?;
                    let y = c.read_u8()?;
                    let z = c.read_u8()?;
                    let i = c.read_u8()?;
                    raw_voxels.push((x, y, z, i));
                }
            }
            b"RGBA" => {
                let mut c = Cursor::new(content);
                // MagicaVoxel stores 256 colors; color index i maps to colors[i-1] in file format
                for i in 1..=255 {
                    let r = c.read_u8()?;
                    let g = c.read_u8()?;
                    let b = c.read_u8()?;
                    let a = c.read_u8()?;
                    palette.set(i as u8, Color::new(r, g, b, a));
                }
            }
            _ => {
                // Skip unhandled chunks (e.g. nTRN, nGRP, MATL)
            }
        }

        // Skip children if any
        if children_len > 0 {
            let mut skip = vec![0u8; children_len];
            cursor.read_exact(&mut skip)?;
        }
    }

    let mut grid = VoxelGrid::new(size_x, size_y, size_z);
    for (x, y, z, id) in raw_voxels {
        grid.set(x as i32, y as i32, z as i32, id);
    }

    Ok(VoxModel { grid, palette })
}

/// Serializes a `VoxelGrid` and `Palette` into valid MagicaVoxel `.vox` binary data.
pub fn write_vox<W: Write>(mut writer: W, grid: &VoxelGrid, palette: &Palette) -> io::Result<()> {
    writer.write_all(VOX_MAGIC)?;
    writer.write_u32::<LittleEndian>(VOX_VERSION)?;

    // 1. Build SIZE chunk content
    let mut size_content = Vec::new();
    size_content.write_u32::<LittleEndian>(grid.size_x)?;
    size_content.write_u32::<LittleEndian>(grid.size_y)?;
    size_content.write_u32::<LittleEndian>(grid.size_z)?;

    // 2. Build XYZI chunk content
    let mut xyzi_content = Vec::new();
    let mut active_voxels = Vec::new();
    for z in 0..grid.size_z {
        for y in 0..grid.size_y {
            for x in 0..grid.size_x {
                let id = grid.get(x as i32, y as i32, z as i32);
                if id != EMPTY_VOXEL {
                    active_voxels.push((x as u8, y as u8, z as u8, id));
                }
            }
        }
    }

    xyzi_content.write_u32::<LittleEndian>(active_voxels.len() as u32)?;
    for (x, y, z, id) in active_voxels {
        xyzi_content.write_u8(x)?;
        xyzi_content.write_u8(y)?;
        xyzi_content.write_u8(z)?;
        xyzi_content.write_u8(id)?;
    }

    // 3. Build RGBA chunk content
    let mut rgba_content = Vec::new();
    for i in 1..=256 {
        let col = if i <= 255 {
            palette.get(i as u8)
        } else {
            Color::BLACK
        };
        rgba_content.write_u8(col.r)?;
        rgba_content.write_u8(col.g)?;
        rgba_content.write_u8(col.b)?;
        rgba_content.write_u8(col.a)?;
    }

    // Assemble MAIN children
    let mut main_children = Vec::new();

    // Write SIZE
    main_children.write_all(b"SIZE")?;
    main_children.write_u32::<LittleEndian>(size_content.len() as u32)?;
    main_children.write_u32::<LittleEndian>(0)?;
    main_children.write_all(&size_content)?;

    // Write XYZI
    main_children.write_all(b"XYZI")?;
    main_children.write_u32::<LittleEndian>(xyzi_content.len() as u32)?;
    main_children.write_u32::<LittleEndian>(0)?;
    main_children.write_all(&xyzi_content)?;

    // Write RGBA
    main_children.write_all(b"RGBA")?;
    main_children.write_u32::<LittleEndian>(rgba_content.len() as u32)?;
    main_children.write_u32::<LittleEndian>(0)?;
    main_children.write_all(&rgba_content)?;

    // Write MAIN
    writer.write_all(b"MAIN")?;
    writer.write_u32::<LittleEndian>(0)?; // MAIN content size = 0
    writer.write_u32::<LittleEndian>(main_children.len() as u32)?;
    writer.write_all(&main_children)?;

    Ok(())
}
