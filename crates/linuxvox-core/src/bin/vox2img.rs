use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use glam::Vec3;
use linuxvox_core::{generate_culled_mesh, read_vox, render_mesh_to_image, BackgroundMode, RenderOptions};

fn print_help() {
    println!("EsperaVox // vox2img - Headless Voxel-to-Image Renderer");
    println!("Usage: vox2img <input.vox> <output.png> [options]\n");
    println!("Options:");
    println!("  --width <pixels>       Image width (default: 800)");
    println!("  --height <pixels>      Image height (default: 800)");
    println!("  --yaw <degrees>        Camera orbit yaw angle (default: 45)");
    println!("  --pitch <degrees>      Camera elevation pitch angle (default: 30)");
    println!("  --iso                  Use orthographic isometric projection");
    println!("  --transparent          Render with transparent background");
    println!("  --turntable <frames>   Render full 360-degree rotation sequence");
    println!("  --help                 Print this help information");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let mut width = 800u32;
    let mut height = 800u32;
    let mut yaw_deg = 45.0f32;
    let mut pitch_deg = 30.0f32;
    let mut is_iso = false;
    let mut bg_mode = BackgroundMode::StudioGradient;
    let mut turntable_frames: Option<usize> = None;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--width" => {
                if i + 1 < args.len() {
                    width = args[i + 1].parse().unwrap_or(800);
                    i += 1;
                }
            }
            "--height" => {
                if i + 1 < args.len() {
                    height = args[i + 1].parse().unwrap_or(800);
                    i += 1;
                }
            }
            "--yaw" => {
                if i + 1 < args.len() {
                    yaw_deg = args[i + 1].parse().unwrap_or(45.0);
                    i += 1;
                }
            }
            "--pitch" => {
                if i + 1 < args.len() {
                    pitch_deg = args[i + 1].parse().unwrap_or(30.0);
                    i += 1;
                }
            }
            "--iso" => is_iso = true,
            "--transparent" => bg_mode = BackgroundMode::Transparent,
            "--bg" => {
                if i + 1 < args.len() {
                    let val = &args[i + 1];
                    match val.to_lowercase().as_str() {
                        "transparent" => bg_mode = BackgroundMode::Transparent,
                        "studio" => bg_mode = BackgroundMode::StudioGradient,
                        "white" => bg_mode = BackgroundMode::Solid([255, 255, 255, 255]),
                        "black" => bg_mode = BackgroundMode::Solid([0, 0, 0, 255]),
                        hex if hex.starts_with('#') && hex.len() == 7 => {
                            if let (Ok(r), Ok(g), Ok(b)) = (
                                u8::from_str_radix(&hex[1..3], 16),
                                u8::from_str_radix(&hex[3..5], 16),
                                u8::from_str_radix(&hex[5..7], 16),
                            ) {
                                bg_mode = BackgroundMode::Solid([r, g, b, 255]);
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }
            "--bg-image" => {
                if i + 1 < args.len() {
                    let img_path = &args[i + 1];
                    match image::open(img_path) {
                        Ok(dyn_img) => {
                            println!("[vox2img] Loaded background image: {}", img_path);
                            bg_mode = BackgroundMode::Image(dyn_img.to_rgba8());
                        }
                        Err(e) => eprintln!("[vox2img] Failed to load background image: {}", e),
                    }
                    i += 1;
                }
            }
            "--turntable" => {
                if i + 1 < args.len() {
                    turntable_frames = Some(args[i + 1].parse().unwrap_or(16));
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    println!("[vox2img] Loading model: {}", input_path);
    let file = File::open(input_path)?;
    let model = read_vox(BufReader::new(file))?;
    println!(
        "[vox2img] Loaded volume {}x{}x{} ({} active voxels)",
        model.grid.size_x,
        model.grid.size_y,
        model.grid.size_z,
        model.grid.count_active()
    );

    let mesh = generate_culled_mesh(&model.grid, &model.palette);
    println!("[vox2img] Generated mesh: {} triangles", mesh.triangle_count());

    // Center the camera on the model's actual bounds
    let target = if let Some((min, max)) = model.grid.bounding_box() {
        (min.as_vec3() + max.as_vec3()) * 0.5
    } else {
        Vec3::new(
            model.grid.size_x as f32 * 0.5,
            model.grid.size_y as f32 * 0.5,
            model.grid.size_z as f32 * 0.5,
        )
    };

    let max_dim = model.grid.size_x.max(model.grid.size_y).max(model.grid.size_z) as f32;
    let distance = max_dim * 1.8;

    if let Some(frames) = turntable_frames {
        let base_stem = Path::new(output_path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let parent = Path::new(output_path).parent().unwrap_or_else(|| Path::new("."));

        println!("[vox2img] Rendering {} turntable frames...", frames);
        for frame in 0..frames {
            let angle = (frame as f32 / frames as f32) * 360.0;
            let options = RenderOptions {
                width,
                height,
                yaw: angle.to_radians(),
                pitch: pitch_deg.to_radians(),
                distance,
                target,
                is_isometric: is_iso,
                background: bg_mode.clone(),
            };
            let img = render_mesh_to_image(&mesh, &options);
            let frame_path = parent.join(format!("{}_{:03}.png", base_stem, frame));
            img.save(&frame_path)?;
            println!("  -> Frame {}/{} saved to {:?}", frame + 1, frames, frame_path);
        }
        println!("[vox2img] Done! All turntable frames rendered.");
    } else {
        println!("[vox2img] Rendering image to {}...", output_path);
        let options = RenderOptions {
            width,
            height,
            yaw: yaw_deg.to_radians(),
            pitch: pitch_deg.to_radians(),
            distance,
            target,
            is_isometric: is_iso,
            background: bg_mode,
        };
        let img = render_mesh_to_image(&mesh, &options);
        img.save(output_path)?;
        println!("[vox2img] Successfully created {}", output_path);
    }

    Ok(())
}
