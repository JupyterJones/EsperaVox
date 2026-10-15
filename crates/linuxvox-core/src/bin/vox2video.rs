use std::env;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use std::process::Command;
use glam::Vec3;
use linuxvox_core::{
    generate_culled_mesh, read_vox, render_mesh_to_image, BackgroundMode, CameraKeyframe, CameraTrack,
};

fn print_help() {
    println!("EsperaVox // vox2video - Cinematic Spline Camera Video Generator");
    println!("Usage: vox2video <input.vox> <output.mp4> [options]\n");
    println!("Options:");
    println!("  --fps <frames_per_sec>   Framerate (default: 30)");
    println!("  --seconds <float>        Total video duration (default: 4.0)");
    println!("  --width <pixels>         Video width (default: 1280)");
    println!("  --height <pixels>        Video height (default: 720)");
    println!("  --mode <crane|orbit>     Cinematic flight style (default: crane)");
    println!("  --help                   Print this help information");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args.contains(&"--help".to_string()) {
        print_help();
        return Ok(());
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let mut fps = 30u32;
    let mut seconds = 4.0f32;
    let mut width = 1280u32;
    let mut height = 720u32;
    let mut mode = "crane".to_string();
    let mut bg_mode = BackgroundMode::StudioGradient;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--fps" => {
                if i + 1 < args.len() {
                    fps = args[i + 1].parse().unwrap_or(30);
                    i += 1;
                }
            }
            "--seconds" => {
                if i + 1 < args.len() {
                    seconds = args[i + 1].parse().unwrap_or(4.0);
                    i += 1;
                }
            }
            "--width" => {
                if i + 1 < args.len() {
                    width = args[i + 1].parse().unwrap_or(1280);
                    i += 1;
                }
            }
            "--height" => {
                if i + 1 < args.len() {
                    height = args[i + 1].parse().unwrap_or(720);
                    i += 1;
                }
            }
            "--mode" => {
                if i + 1 < args.len() {
                    mode = args[i + 1].clone();
                    i += 1;
                }
            }
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
                            println!("[vox2video] Loaded backdrop image: {}", img_path);
                            bg_mode = BackgroundMode::Image(dyn_img.to_rgba8());
                        }
                        Err(e) => eprintln!("[vox2video] Failed to load backdrop image: {}", e),
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    println!("[vox2video] Loading voxel model: {}", input_path);
    let file = File::open(input_path)?;
    let model = read_vox(BufReader::new(file))?;
    let mesh = generate_culled_mesh(&model.grid, &model.palette);

    println!(
        "[vox2video] Model loaded ({} voxels, {} triangles)",
        model.grid.count_active(),
        mesh.triangle_count()
    );

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
    let base_dist = max_dim * 1.7;

    // Define Camera Flight Keyframes based on mode
    let track = if mode == "orbit" {
        // Continuous 360-degree hero orbit
        CameraTrack::new(
            vec![
                CameraKeyframe::new(0.0, 0.45, base_dist, target, 0.0),
                CameraKeyframe::new(std::f32::consts::FRAC_PI_2, 0.45, base_dist, target, 0.25),
                CameraKeyframe::new(std::f32::consts::PI, 0.45, base_dist, target, 0.5),
                CameraKeyframe::new(std::f32::consts::PI * 1.5, 0.45, base_dist, target, 0.75),
            ],
            true, // Perfectly looping
        )
    } else {
        // Dramatic cinematic crane & dolly swoop
        CameraTrack::new(
            vec![
                // Shot 1: Low-angle dramatic entry looking up at gate/base
                CameraKeyframe::new(0.2, 0.15, base_dist * 1.25, target - Vec3::new(0.0, 4.0, 0.0), 0.0),
                // Shot 2: Swooping upward around the left flank
                CameraKeyframe::new(1.8, 0.65, base_dist * 1.05, target + Vec3::new(0.0, 2.0, 0.0), 0.33),
                // Shot 3: High dramatic crane overlooking spire/roof from behind
                CameraKeyframe::new(3.5, 0.95, base_dist * 1.4, target + Vec3::new(0.0, 4.0, 0.0), 0.66),
                // Shot 4: Banking right and descending
                CameraKeyframe::new(5.2, 0.35, base_dist * 1.15, target, 0.85),
            ],
            true, // Seamless loop
        )
    };

    let total_frames = (fps as f32 * seconds).round() as usize;
    println!(
        "[vox2video] Generating {} frames ({}s @ {} FPS, {}x{})...",
        total_frames, seconds, fps, width, height
    );

    let temp_dir = Path::new("temp_render_frames");
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir)?;
    }
    fs::create_dir_all(temp_dir)?;

    for frame in 0..total_frames {
        let t = frame as f32 / total_frames as f32;
        let mut opts = track.evaluate(t);
        opts.width = width;
        opts.height = height;
        opts.background = bg_mode.clone();

        let img = render_mesh_to_image(&mesh, &opts);
        let frame_file = temp_dir.join(format!("frame_{:04}.png", frame));
        img.save(&frame_file)?;

        if (frame + 1) % 15 == 0 || frame + 1 == total_frames {
            print!("\r[vox2video] Rendered frame {}/{} ({:.0}%)", frame + 1, total_frames, ((frame + 1) as f32 / total_frames as f32) * 100.0);
            use std::io::Write;
            std::io::stdout().flush().ok();
        }
    }
    println!();

    println!("[vox2video] Encoding MP4 with FFmpeg...");
    let ffmpeg_status = Command::new("ffmpeg")
        .args([
            "-y",
            "-framerate",
            &fps.to_string(),
            "-i",
            "temp_render_frames/frame_%04d.png",
            "-c:v",
            "libx264",
            "-preset",
            "medium",
            "-crf",
            "18",
            "-pix_fmt",
            "yuv420p",
            output_path,
        ])
        .output()?;

    if !ffmpeg_status.status.success() {
        let err_str = String::from_utf8_lossy(&ffmpeg_status.stderr);
        eprintln!("[vox2video] FFmpeg error: {}", err_str);
        return Err("FFmpeg encoding failed".into());
    }

    // Clean up temporary frame files
    fs::remove_dir_all(temp_dir).ok();

    let meta = fs::metadata(output_path)?;
    println!(
        "[vox2video] Done! Saved video to '{}' ({:.2} MB)",
        output_path,
        meta.len() as f32 / (1024.0 * 1024.0)
    );

    Ok(())
}
