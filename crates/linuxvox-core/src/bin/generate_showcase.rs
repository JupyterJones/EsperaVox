use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;
use glam::IVec3;
use linuxvox_core::{write_vox, Palette, VoxelGrid};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = Path::new("examples");
    fs::create_dir_all(out_dir)?;

    let palette = Palette::magica_default();

    // 1. Generate Castle Keep
    println!("Generating Castle_Keep.vox...");
    let castle = create_castle_keep();
    let file = File::create(out_dir.join("Castle_Keep.vox"))?;
    write_vox(BufWriter::new(file), &castle, &palette)?;

    // 2. Generate Retro Bot
    println!("Generating Retro_Bot.vox...");
    let bot = create_retro_bot();
    let file = File::create(out_dir.join("Retro_Bot.vox"))?;
    write_vox(BufWriter::new(file), &bot, &palette)?;

    // 3. Generate Floating Island
    println!("Generating Floating_Island.vox...");
    let island = create_floating_island();
    let file = File::create(out_dir.join("Floating_Island.vox"))?;
    write_vox(BufWriter::new(file), &island, &palette)?;

    println!("All showcase models generated in examples/");
    Ok(())
}

/// Creates a detailed medieval castle fortress tower.
fn create_castle_keep() -> VoxelGrid {
    let mut grid = VoxelGrid::new(32, 40, 32);

    let stone_dark: u8 = Palette::MID_GRAY;
    let stone_mid: u8 = Palette::SANDSTONE;
    let stone_light: u8 = Palette::WHITE;
    let wood: u8 = Palette::WARM_WOOD;
    let iron: u8 = Palette::CHARCOAL;
    let roof_red: u8 = Palette::VIVID_RED;
    let gold: u8 = Palette::GOLD_YELLOW;
    let torch_fire: u8 = Palette::FLAME_ORANGE;

    // Base foundation (Layer 0 to 2)
    for y in 0..=2 {
        for z in 4..=27 {
            for x in 4..=27 {
                let col = if (x + y + z) % 3 == 0 { stone_dark } else { stone_mid };
                grid.set(x, y, z, col);
            }
        }
    }

    // Main tower walls (Layer 3 to 22)
    for y in 3..=22 {
        for z in 6..=25 {
            for x in 6..=25 {
                let is_wall = x == 6 || x == 25 || z == 6 || z == 25;
                if is_wall {
                    let col = if (x * 7 + y * 13 + z * 3) % 5 == 0 {
                        stone_light
                    } else if (x + y + z) % 2 == 0 {
                        stone_mid
                    } else {
                        stone_dark
                    };
                    grid.set(x, y, z, col);
                }
            }
        }
    }

    // Wooden gate / door at front (z = 25)
    for y in 3..=9 {
        for x in 13..=18 {
            grid.set(x, y, 25, wood);
        }
    }
    // Iron door studs & arch
    for y in 4..=8 {
        grid.set(13, y, 26, iron);
        grid.set(18, y, 26, iron);
    }
    for x in 13..=18 {
        grid.set(x, 10, 26, stone_dark);
    }

    // Arrow slit windows
    for y in 14..=17 {
        grid.set(10, y, 25, 0); // Window front
        grid.set(21, y, 25, 0);
        grid.set(6, y, 15, 0);  // Window left
        grid.set(25, y, 15, 0); // Window right
    }

    // Torches by the door
    grid.set(11, 8, 26, wood);
    grid.set(11, 9, 26, torch_fire);
    grid.set(20, 8, 26, wood);
    grid.set(20, 9, 26, torch_fire);

    // Battlements / Parapet overhang (Layer 23 to 26)
    for y in 23..=25 {
        for z in 5..=26 {
            for x in 5..=26 {
                let is_parapet = x == 5 || x == 26 || z == 5 || z == 26;
                if is_parapet {
                    grid.set(x, y, z, stone_mid);
                }
            }
        }
    }

    // Crenellations (teeth on top of parapet)
    for z in 5..=26 {
        for x in 5..=26 {
            let on_edge = x == 5 || x == 26 || z == 5 || z == 26;
            if on_edge && (x % 3 == 0 || z % 3 == 0) {
                grid.set(x, 26, z, stone_light);
            }
        }
    }

    // Inner Watch Spire (Layer 23 to 34)
    for y in 23..=30 {
        for z in 11..=20 {
            for x in 11..=20 {
                if x == 11 || x == 20 || z == 11 || z == 20 {
                    grid.set(x, y, z, stone_dark);
                }
            }
        }
    }

    // Pyramidal Steeple Roof
    for step in 0..6 {
        let y = 30 + step;
        let min_c = 11 + step;
        let max_c = 20 - step;
        if min_c <= max_c {
            for z in min_c..=max_c {
                for x in min_c..=max_c {
                    grid.set(x, y, z, roof_red);
                }
            }
        }
    }

    // Flagpole and Banner
    for y in 35..=39 {
        grid.set(15, y, 15, wood);
    }
    // Red and Gold Banner
    for y in 37..=39 {
        for x in 16..=19 {
            let col = if (x + y) % 2 == 0 { roof_red } else { gold };
            grid.set(x, y, 15, col);
        }
    }

    grid
}

/// Creates a stylized retro sci-fi robot with glowing visor.
fn create_retro_bot() -> VoxelGrid {
    let mut grid = VoxelGrid::new(32, 36, 32);

    let metal_dark: u8 = Palette::CHARCOAL;
    let metal_blue: u8 = Palette::COBALT_BLUE;
    let metal_light: u8 = Palette::WHITE;
    let accent_orange: u8 = Palette::BRIGHT_ORANGE;
    let visor_cyan: u8 = Palette::ELECTRIC_CYAN;
    let joint_gray: u8 = Palette::MID_GRAY;

    // Feet (Layer 0 to 2)
    grid.fill_box(IVec3::new(8, 0, 11), IVec3::new(13, 2, 19), metal_dark);
    grid.fill_box(IVec3::new(18, 0, 11), IVec3::new(23, 2, 19), metal_dark);

    // Legs (Layer 3 to 11)
    grid.fill_box(IVec3::new(9, 3, 13), IVec3::new(12, 10, 17), metal_blue);
    grid.fill_box(IVec3::new(19, 3, 13), IVec3::new(22, 10, 17), metal_blue);
    // Knee armor plates
    grid.fill_box(IVec3::new(9, 6, 18), IVec3::new(12, 8, 18), metal_light);
    grid.fill_box(IVec3::new(19, 6, 18), IVec3::new(22, 8, 18), metal_light);

    // Pelvis / Hip Joint (Layer 11 to 13)
    grid.fill_box(IVec3::new(10, 11, 13), IVec3::new(21, 13, 17), joint_gray);

    // Main Torso Chassis (Layer 14 to 24)
    grid.fill_box(IVec3::new(7, 14, 11), IVec3::new(24, 24, 19), metal_blue);
    // Chest Armor Plate
    grid.fill_box(IVec3::new(9, 16, 20), IVec3::new(22, 22, 20), metal_light);
    // Chest Power Core (Glowing dials)
    grid.fill_box(IVec3::new(13, 18, 21), IVec3::new(18, 20, 21), accent_orange);

    // Jetpack on back
    grid.fill_box(IVec3::new(9, 15, 8), IVec3::new(13, 23, 10), metal_dark);
    grid.fill_box(IVec3::new(18, 15, 8), IVec3::new(22, 23, 10), metal_dark);
    // Jet thruster flames
    grid.fill_box(IVec3::new(10, 13, 9), IVec3::new(12, 14, 9), accent_orange);
    grid.fill_box(IVec3::new(19, 13, 9), IVec3::new(21, 14, 9), accent_orange);

    // Arms & Shoulder Pauldrons (Layer 16 to 23)
    // Left Shoulder
    grid.fill_box(IVec3::new(3, 21, 12), IVec3::new(6, 25, 18), metal_light);
    grid.fill_box(IVec3::new(4, 14, 13), IVec3::new(6, 20, 17), metal_blue);
    grid.fill_box(IVec3::new(4, 11, 14), IVec3::new(6, 13, 19), metal_dark); // Hand claw

    // Right Shoulder
    grid.fill_box(IVec3::new(25, 21, 12), IVec3::new(28, 25, 18), metal_light);
    grid.fill_box(IVec3::new(25, 14, 13), IVec3::new(27, 20, 17), metal_blue);
    grid.fill_box(IVec3::new(25, 11, 14), IVec3::new(27, 13, 19), metal_dark); // Hand claw

    // Neck
    grid.fill_box(IVec3::new(14, 25, 14), IVec3::new(17, 25, 16), joint_gray);

    // Head (Layer 26 to 32)
    grid.fill_box(IVec3::new(9, 26, 11), IVec3::new(22, 32, 19), metal_blue);
    // Glowing Visor
    grid.fill_box(IVec3::new(11, 28, 20), IVec3::new(20, 30, 20), visor_cyan);

    // Antennae on ears
    grid.fill_box(IVec3::new(8, 28, 14), IVec3::new(8, 34, 16), accent_orange);
    grid.fill_box(IVec3::new(23, 28, 14), IVec3::new(23, 34, 16), accent_orange);
    grid.set(8, 35, 15, visor_cyan);
    grid.set(23, 35, 15, visor_cyan);

    grid
}

/// Creates a fantasy floating island diorama with cottage and shrine.
fn create_floating_island() -> VoxelGrid {
    let mut grid = VoxelGrid::new(32, 32, 32);

    let grass_top: u8 = Palette::LIME_GREEN;
    let grass_side: u8 = Palette::EMERALD_GREEN;
    let dirt: u8 = Palette::WARM_WOOD;
    let stone: u8 = Palette::MID_GRAY;
    let wood: u8 = Palette::WARM_WOOD;
    let roof: u8 = Palette::VIVID_RED;
    let crystal: u8 = Palette::HOT_MAGENTA;
    let flower_red: u8 = Palette::VIVID_RED;
    let flower_yellow: u8 = Palette::SUNSHINE_YELLOW;

    // Floating island inverted conical base (y = 2 to 14)
    let center_x = 16.0;
    let center_z = 16.0;

    for y in 2..=14 {
        let t = (y - 2) as f32 / 12.0;
        let radius = 2.0 + t * 11.5;

        for z in 0..32 {
            for x in 0..32 {
                let dx = x as f32 - center_x;
                let dz = z as f32 - center_z;
                let dist = (dx * dx + dz * dz).sqrt();

                if dist <= radius {
                    let col = if y == 14 {
                        grass_top
                    } else if y == 13 {
                        grass_side
                    } else if y >= 8 {
                        dirt
                    } else {
                        stone
                    };
                    grid.set(x, y, z, col);
                }
            }
        }
    }

    // Wildflowers on grass
    for z in 8..=24 {
        for x in 8..=24 {
            if grid.get(x, 14, z) == grass_top {
                if (x * 13 + z * 7) % 17 == 0 {
                    grid.set(x, 15, z, flower_red);
                } else if (x * 19 + z * 11) % 19 == 0 {
                    grid.set(x, 15, z, flower_yellow);
                }
            }
        }
    }

    // Miniature Cottage (y = 15 to 22)
    grid.fill_box(IVec3::new(9, 15, 9), IVec3::new(16, 18, 16), wood);
    // Cottage Door
    grid.set(12, 15, 17, 1);
    grid.set(13, 15, 17, 1);
    grid.set(12, 16, 17, 1);
    grid.set(13, 16, 17, 1);

    // Gabled Roof
    for step in 0..4 {
        let y = 19 + step;
        let min_x = 8 + step;
        let max_x = 17 - step;
        if min_x <= max_x {
            for x in min_x..=max_x {
                for z in 8..=17 {
                    grid.set(x, y, z, roof);
                }
            }
        }
    }

    // Stone Chimney with smoke
    for y in 18..=24 {
        grid.set(15, y, 10, stone);
    }
    grid.set(15, 25, 10, Palette::WHITE); // Smoke puff 1
    grid.set(16, 26, 10, Palette::WHITE); // Smoke puff 2
    grid.set(16, 27, 11, Palette::WHITE); // Smoke puff 3

    // Floating Crystal Shrine on the other side of island
    grid.fill_box(IVec3::new(20, 15, 20), IVec3::new(24, 15, 24), stone);
    grid.set(20, 16, 20, stone);
    grid.set(24, 16, 20, stone);
    grid.set(20, 16, 24, stone);
    grid.set(24, 16, 24, stone);

    // Glowing Crystal Hovering in Center
    grid.set(22, 17, 22, crystal);
    grid.fill_box(IVec3::new(21, 18, 21), IVec3::new(23, 19, 23), crystal);
    grid.set(22, 20, 22, crystal);

    grid
}
