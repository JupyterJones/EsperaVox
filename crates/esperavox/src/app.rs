use eframe::egui::{self, Color32, Key, Modifiers, Sense, Stroke, Vec2};
use glam::{IVec3, Vec3};
use linuxvox_core::{
    generate_culled_mesh, raycast_grid, read_vox, write_vox, EditCommand, HistoryManager, Mesh,
    Palette, RayHit, VoxelDiff, VoxelGrid, EMPTY_VOXEL,
    CameraKeyframe, CameraTrack, render_mesh_to_image,
};
use crate::camera::OrbitCamera;
use crate::viewport::{render_3d_viewport, ViewportRenderParams};
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMode {
    Voxel,
    Erase,
    Paint,
}

pub struct EsperaVoxApp {
    grid: VoxelGrid,
    palette: Palette,
    camera: OrbitCamera,
    mesh: Mesh,
    history: HistoryManager,

    selected_palette_id: u8,
    active_tool: ToolMode,
    show_grid: bool,
    show_box: bool,

    bg_color: Color32,
    is_transparent_bg: bool,
    backdrop_texture: Option<egui::TextureHandle>,
    backdrop_name: Option<String>,

    // Cinematic Keyframe Timeline
    show_timeline: bool,
    camera_track: CameraTrack,
    timeline_progress: f32,
    timeline_duration: f32,
    is_playing: bool,

    hover_hit: Option<RayHit>,
    ghost_pos: Option<IVec3>,
    status_message: String,
}

impl Default for EsperaVoxApp {
    fn default() -> Self {
        let mut grid = VoxelGrid::new(32, 32, 32);
        // Start with a small default pedestal so the user immediately has something to sculpt
        let center = 16;
        for dx in -2..=2 {
            for dz in -2..=2 {
                grid.set(center + dx, 0, center + dz, 12);
            }
        }
        grid.set(center, 1, center, 24);

        let palette = Palette::magica_default();
        let mesh = generate_culled_mesh(&grid, &palette);

        Self {
            grid,
            palette,
            camera: OrbitCamera::default(),
            mesh,
            history: HistoryManager::new(100),
            selected_palette_id: 24,
            active_tool: ToolMode::Voxel,
            show_grid: true,
            show_box: true,
            bg_color: Color32::from_rgb(20, 24, 30),
            is_transparent_bg: false,
            backdrop_texture: None,
            backdrop_name: None,
            show_timeline: true,
            camera_track: CameraTrack::default(),
            timeline_progress: 0.0,
            timeline_duration: 4.0,
            is_playing: false,
            hover_hit: None,
            ghost_pos: None,
            status_message: "Welcome to EsperaVox! Right-click drag to orbit, Left-click to sculpt.".to_string(),
        }
    }
}

impl EsperaVoxApp {
    pub fn rebuild_mesh(&mut self) {
        self.mesh = generate_culled_mesh(&self.grid, &self.palette);
    }

    fn open_vox_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("MagicaVoxel Model", &["vox"])
            .pick_file()
        {
            match File::open(&path) {
                Ok(file) => match read_vox(BufReader::new(file)) {
                    Ok(model) => {
                        self.grid = model.grid;
                        self.palette = model.palette;
                        self.rebuild_mesh();
                        self.history.clear();
                        self.camera.target = Vec3::new(
                            self.grid.size_x as f32 / 2.0,
                            self.grid.size_y as f32 / 2.0,
                            self.grid.size_z as f32 / 2.0,
                        );
                        self.status_message = format!(
                            "Loaded '{}' ({} active voxels)",
                            path.file_name().unwrap_or_default().to_string_lossy(),
                            self.grid.count_active()
                        );
                    }
                    Err(e) => self.status_message = format!("Error loading .vox: {}", e),
                },
                Err(e) => self.status_message = format!("Could not open file: {}", e),
            }
        }
    }

    fn save_vox_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("MagicaVoxel Model", &["vox"])
            .set_file_name("model.vox")
            .save_file()
        {
            match File::create(&path) {
                Ok(file) => match write_vox(BufWriter::new(file), &self.grid, &self.palette) {
                    Ok(_) => {
                        self.status_message = format!(
                            "Saved to '{}'",
                            path.file_name().unwrap_or_default().to_string_lossy()
                        );
                    }
                    Err(e) => self.status_message = format!("Error saving .vox: {}", e),
                },
                Err(e) => self.status_message = format!("Could not create file: {}", e),
            }
        }
    }

    fn export_obj_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Wavefront OBJ", &["obj"])
            .set_file_name("model.obj")
            .save_file()
        {
            let obj_data = self.mesh.to_obj();
            match std::fs::write(&path, obj_data) {
                Ok(_) => {
                    self.status_message = format!(
                        "Exported OBJ to '{}' ({} triangles)",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        self.mesh.triangle_count()
                    );
                }
                Err(e) => self.status_message = format!("Failed to export OBJ: {}", e),
            }
        }
    }

    fn load_backdrop_dialog(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Image Plane", &["png", "jpg", "jpeg"])
            .pick_file()
        {
            match image::open(&path) {
                Ok(dyn_img) => {
                    let rgba = dyn_img.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    let color_img = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                    let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let handle = ctx.load_texture(&filename, color_img, Default::default());
                    self.backdrop_texture = Some(handle);
                    self.backdrop_name = Some(filename.clone());
                    self.is_transparent_bg = false;
                    self.status_message = format!("Loaded backdrop image: {}", filename);
                }
                Err(e) => {
                    self.status_message = format!("Failed to open image plane: {}", e);
                }
            }
        }
    }

    fn export_video_dialog(&mut self) {
        if self.camera_track.len() < 2 {
            self.status_message = "Please capture at least 2 keyframes before exporting video!".to_string();
            return;
        }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("MP4 Video", &["mp4"])
            .set_file_name("render.mp4")
            .save_file()
        {
            let temp_dir = std::path::Path::new("temp_gui_frames");
            if temp_dir.exists() {
                let _ = std::fs::remove_dir_all(temp_dir);
            }
            if let Err(e) = std::fs::create_dir_all(temp_dir) {
                self.status_message = format!("Failed to create render dir: {}", e);
                return;
            }

            let fps = 30u32;
            let total_frames = (fps as f32 * self.timeline_duration).round() as usize;
            let width = 1280u32;
            let height = 720u32;

            for frame in 0..total_frames {
                let t = frame as f32 / total_frames as f32;
                let mut opts = self.camera_track.evaluate(t);
                opts.width = width;
                opts.height = height;

                let img = render_mesh_to_image(&self.mesh, &opts);
                let frame_file = temp_dir.join(format!("frame_{:04}.png", frame));
                if let Err(e) = img.save(&frame_file) {
                    self.status_message = format!("Failed to save frame: {}", e);
                    let _ = std::fs::remove_dir_all(temp_dir);
                    return;
                }
            }

            let ffmpeg_status = std::process::Command::new("ffmpeg")
                .args([
                    "-y",
                    "-framerate",
                    &fps.to_string(),
                    "-i",
                    "temp_gui_frames/frame_%04d.png",
                    "-c:v",
                    "libx264",
                    "-preset",
                    "medium",
                    "-crf",
                    "18",
                    "-pix_fmt",
                    "yuv420p",
                    path.to_str().unwrap_or("output.mp4"),
                ])
                .output();

            let _ = std::fs::remove_dir_all(temp_dir);

            match ffmpeg_status {
                Ok(status) if status.status.success() => {
                    self.status_message = format!(
                        "Successfully exported video to '{}'!",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    );
                }
                Ok(status) => {
                    self.status_message = format!(
                        "FFmpeg failed with exit code: {:?}",
                        status.status.code()
                    );
                }
                Err(e) => {
                    self.status_message = format!("Could not run FFmpeg: {}", e);
                }
            }
        }
    }
}

impl eframe::App for EsperaVoxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Real-Time Camera Animation Playback
        if self.is_playing {
            let dt = ctx.input(|i| i.stable_dt);
            self.timeline_progress += dt / self.timeline_duration.max(0.1);
            if self.timeline_progress >= 1.0 {
                if self.camera_track.is_looping {
                    self.timeline_progress %= 1.0;
                } else {
                    self.timeline_progress = 1.0;
                    self.is_playing = false;
                }
            }

            if !self.camera_track.is_empty() {
                let opts = self.camera_track.evaluate(self.timeline_progress);
                self.camera.yaw = opts.yaw;
                self.camera.pitch = opts.pitch;
                self.camera.distance = opts.distance;
                self.camera.target = opts.target;
            }

            ctx.request_repaint();
        }

        // Keyboard Shortcuts
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Z) || i.consume_key(Modifiers::CTRL, Key::Z)) {
            if self.history.undo(&mut self.grid) {
                self.rebuild_mesh();
                self.status_message = "Undo edit".to_string();
            }
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Y) || i.consume_key(Modifiers::CTRL, Key::Y)) {
            if self.history.redo(&mut self.grid) {
                self.rebuild_mesh();
                self.status_message = "Redo edit".to_string();
            }
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::V)) {
            self.active_tool = ToolMode::Voxel;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::E)) {
            self.active_tool = ToolMode::Erase;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::P)) {
            self.active_tool = ToolMode::Paint;
        }

        // Top Menu Bar
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New (Clear)").clicked() {
                        self.grid.clear();
                        self.history.clear();
                        self.rebuild_mesh();
                        ui.close_menu();
                    }
                    if ui.button("Open .vox (MagicaVoxel)...").clicked() {
                        self.open_vox_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Save .vox...").clicked() {
                        self.save_vox_dialog();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Export .obj...").clicked() {
                        self.export_obj_dialog();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.add_enabled(self.history.can_undo(), egui::Button::new("Undo (Ctrl+Z)")).clicked() {
                        if self.history.undo(&mut self.grid) {
                            self.rebuild_mesh();
                        }
                        ui.close_menu();
                    }
                    if ui.add_enabled(self.history.can_redo(), egui::Button::new("Redo (Ctrl+Y)")).clicked() {
                        if self.history.redo(&mut self.grid) {
                            self.rebuild_mesh();
                        }
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_grid, "Floor Grid");
                    ui.checkbox(&mut self.show_box, "Bounding Box");
                    if ui.button("Reset Camera").clicked() {
                        self.camera = OrbitCamera::default();
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.label("Background:");
                    if ui.selectable_label(!self.is_transparent_bg && self.bg_color == Color32::from_rgb(20, 24, 30), "Studio Dark").clicked() {
                        self.bg_color = Color32::from_rgb(20, 24, 30);
                        self.is_transparent_bg = false;
                        self.backdrop_texture = None;
                    }
                    if ui.selectable_label(!self.is_transparent_bg && self.bg_color == Color32::from_rgb(15, 23, 42), "Deep Slate").clicked() {
                        self.bg_color = Color32::from_rgb(15, 23, 42);
                        self.is_transparent_bg = false;
                        self.backdrop_texture = None;
                    }
                    if ui.selectable_label(!self.is_transparent_bg && self.bg_color == Color32::BLACK, "Pure Black").clicked() {
                        self.bg_color = Color32::BLACK;
                        self.is_transparent_bg = false;
                        self.backdrop_texture = None;
                    }
                    if ui.selectable_label(!self.is_transparent_bg && self.bg_color == Color32::WHITE, "Pure White").clicked() {
                        self.bg_color = Color32::WHITE;
                        self.is_transparent_bg = false;
                        self.backdrop_texture = None;
                    }
                    if ui.selectable_label(!self.is_transparent_bg && self.bg_color == Color32::from_rgb(56, 189, 248), "Sky Blue").clicked() {
                        self.bg_color = Color32::from_rgb(56, 189, 248);
                        self.is_transparent_bg = false;
                        self.backdrop_texture = None;
                    }
                    if ui.selectable_label(self.is_transparent_bg, "Transparent (Checkerboard)").clicked() {
                        self.is_transparent_bg = true;
                        self.backdrop_texture = None;
                    }
                    ui.horizontal(|ui| {
                        ui.label("Custom Color:");
                        ui.color_edit_button_srgba(&mut self.bg_color);
                    });
                    ui.separator();
                    if ui.button("Load Image Plane Backdrop...").clicked() {
                        self.load_backdrop_dialog(ctx);
                        ui.close_menu();
                    }
                    if self.backdrop_texture.is_some() {
                        if ui.button("Clear Image Backdrop").clicked() {
                            self.backdrop_texture = None;
                            self.backdrop_name = None;
                            ui.close_menu();
                        }
                    }
                    ui.separator();
                    ui.checkbox(&mut self.show_timeline, "Cinematic Timeline");
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Voxels: {} | Triangles: {}", self.grid.count_active(), self.mesh.triangle_count()));
                });
            });
        });

        // Bottom Status Bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
                if let Some(hit) = self.hover_hit {
                    ui.separator();
                    ui.label(format!("Voxel: ({}, {}, {})", hit.voxel_coord.x, hit.voxel_coord.y, hit.voxel_coord.z));
                }
            });
        });

        // Cinematic Timeline Dock
        if self.show_timeline {
            egui::TopBottomPanel::bottom("timeline_panel")
                .resizable(false)
                .default_height(78.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong("🎬 Timeline");
                        ui.separator();

                        // Play / Pause Button
                        let play_text = if self.is_playing { "⏸ Pause" } else { "▶ Play" };
                        if ui.button(play_text).clicked() {
                            if !self.camera_track.is_empty() {
                                self.is_playing = !self.is_playing;
                                if self.is_playing && self.timeline_progress >= 0.99 {
                                    self.timeline_progress = 0.0;
                                }
                            } else {
                                self.status_message = "Capture at least 1 keyframe first!".to_string();
                            }
                        }

                        // Jump to start
                        if ui.button("⏮").clicked() {
                            self.timeline_progress = 0.0;
                            if !self.camera_track.is_empty() {
                                let opts = self.camera_track.evaluate(0.0);
                                self.camera.yaw = opts.yaw;
                                self.camera.pitch = opts.pitch;
                                self.camera.distance = opts.distance;
                                self.camera.target = opts.target;
                            }
                        }

                        ui.checkbox(&mut self.camera_track.is_looping, "🔁 Loop");

                        ui.separator();

                        // Capture Keyframe Button
                        let cap_btn = egui::Button::new(
                            egui::RichText::new("➕ Capture Keyframe")
                                .color(Color32::from_rgb(80, 200, 255))
                                .strong(),
                        );
                        if ui.add(cap_btn).clicked() {
                            let kf = CameraKeyframe::new(
                                self.camera.yaw,
                                self.camera.pitch,
                                self.camera.distance,
                                self.camera.target,
                                self.timeline_progress,
                            );
                            self.camera_track.add_keyframe(kf);
                            self.status_message = format!(
                                "Captured Keyframe #{} at {:.2}s!",
                                self.camera_track.len(),
                                self.timeline_progress * self.timeline_duration
                            );
                        }

                        // Export MP4 Button
                        let export_btn = egui::Button::new(
                            egui::RichText::new("🎥 Export MP4...")
                                .color(Color32::from_rgb(100, 255, 140))
                                .strong(),
                        );
                        if ui.add(export_btn).clicked() {
                            self.export_video_dialog();
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑 Clear All").clicked() {
                                self.camera_track.clear();
                                self.is_playing = false;
                                self.status_message = "Cleared all camera keyframes".to_string();
                            }

                            ui.add(
                                egui::DragValue::new(&mut self.timeline_duration)
                                    .range(1.0..=60.0)
                                    .speed(0.1)
                                    .suffix("s"),
                            );
                            ui.label("Duration:");

                            ui.label(format!("Keyframes: {}", self.camera_track.len()));
                        });
                    });

                    ui.separator();

                    // Scrubber row with keyframe chips
                    ui.horizontal(|ui| {
                        let cur_sec = self.timeline_progress * self.timeline_duration;
                        ui.monospace(format!("{:04.2}s", cur_sec));

                        let slider = ui.add(
                            egui::Slider::new(&mut self.timeline_progress, 0.0..=1.0)
                                .show_value(false)
                        );
                        if slider.changed() && !self.camera_track.is_empty() {
                            let opts = self.camera_track.evaluate(self.timeline_progress);
                            self.camera.yaw = opts.yaw;
                            self.camera.pitch = opts.pitch;
                            self.camera.distance = opts.distance;
                            self.camera.target = opts.target;
                        }

                        // Display keyframe chips
                        for (idx, kf) in self.camera_track.keyframes.iter().enumerate() {
                            let kf_sec = kf.time * self.timeline_duration;
                            let chip_text = format!("♦ #{} ({:.1}s)", idx + 1, kf_sec);
                            if ui.small_button(chip_text).clicked() {
                                self.timeline_progress = kf.time;
                                self.camera.yaw = kf.yaw;
                                self.camera.pitch = kf.pitch;
                                self.camera.distance = kf.distance;
                                self.camera.target = kf.target;
                            }
                        }
                    });
                });
        }

        // Left Dock: MagicaVoxel 256-Color Palette
        egui::SidePanel::left("palette_dock")
            .resizable(false)
            .default_width(170.0)
            .show(ctx, |ui| {
                ui.heading("Palette");
                ui.separator();

                // Selected Color Preview
                let cur_col = self.palette.get(self.selected_palette_id);
                let cur_c32 = Color32::from_rgb(cur_col.r, cur_col.g, cur_col.b);
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::hover());
                    ui.painter().rect_filled(rect, 4.0, cur_c32);
                    ui.painter().rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::WHITE));
                    ui.vertical(|ui| {
                        ui.label(format!("ID: {}", self.selected_palette_id));
                        ui.label(cur_col.to_hex());
                    });
                });

                ui.separator();

                // 256 Color Swatches (16 columns x 16 rows)
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let swatch_size = 9.0;
                    let spacing = 1.0;

                    for row in 0..16 {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::splat(spacing);
                            for col in 0..16 {
                                let id = (row * 16 + col + 1).min(255) as u8;
                                let col_val = self.palette.get(id);
                                let c32 = Color32::from_rgb(col_val.r, col_val.g, col_val.b);

                                let (rect, resp) = ui.allocate_exact_size(Vec2::splat(swatch_size), Sense::click());
                                ui.painter().rect_filled(rect, 1.0, c32);

                                if self.selected_palette_id == id {
                                    ui.painter().rect_stroke(rect, 1.0, Stroke::new(1.5, Color32::WHITE));
                                }

                                if resp.clicked() {
                                    self.selected_palette_id = id;
                                }
                            }
                        });
                    }
                });
            });

        // Right Dock: Tool Selection & Modifiers
        egui::SidePanel::right("tools_dock")
            .resizable(false)
            .default_width(160.0)
            .show(ctx, |ui| {
                ui.heading("Tools");
                ui.separator();

                ui.selectable_value(&mut self.active_tool, ToolMode::Voxel, "Voxel (V)");
                ui.selectable_value(&mut self.active_tool, ToolMode::Erase, "Erase (E)");
                ui.selectable_value(&mut self.active_tool, ToolMode::Paint, "Paint (P)");

                ui.separator();
                ui.heading("Shortcuts");
                ui.small("• Left Click: Sculpt / Use Tool");
                ui.small("• Right Drag: Orbit Camera");
                ui.small("• Shift + Right: Pan Camera");
                ui.small("• Scroll: Zoom In / Out");
                ui.small("• Ctrl+Z / Ctrl+Y: Undo / Redo");
            });

        // Central 3D Viewport
        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());

            // Handle Camera Navigation
            if response.dragged_by(egui::PointerButton::Secondary) {
                let delta = response.drag_delta();
                if ctx.input(|i| i.modifiers.shift) {
                    self.camera.pan(glam::Vec2::new(delta.x, delta.y));
                } else {
                    self.camera.orbit(glam::Vec2::new(delta.x, delta.y));
                }
            }

            let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
            if response.hovered() && scroll.abs() > 0.01 {
                self.camera.zoom(scroll * 0.05);
            }

            // Handle Mouse Hover & Raycasting
            self.hover_hit = None;
            self.ghost_pos = None;

            if let Some(mouse_pos) = response.hover_pos() {
                let local_pos = mouse_pos - rect.min;
                let (ray_origin, ray_dir) = self.camera.screen_to_ray(
                    glam::Vec2::new(local_pos.x, local_pos.y),
                    glam::Vec2::new(rect.width(), rect.height()),
                );

                if let Some(hit) = raycast_grid(ray_origin, ray_dir, 500.0, &self.grid) {
                    self.hover_hit = Some(hit);
                    let target_pos = hit.voxel_coord + hit.face_normal;
                    if self.grid.is_in_bounds(target_pos.x, target_pos.y, target_pos.z) {
                        self.ghost_pos = Some(target_pos);
                    }
                } else {
                    // Floor raycast when hovering over empty space
                    if ray_dir.y.abs() > 1e-4 {
                        let t_floor = -ray_origin.y / ray_dir.y;
                        if t_floor > 0.0 {
                            let floor_p = ray_origin + ray_dir * t_floor;
                            let fx = floor_p.x.floor() as i32;
                            let fz = floor_p.z.floor() as i32;
                            if self.grid.is_in_bounds(fx, 0, fz) {
                                self.ghost_pos = Some(IVec3::new(fx, 0, fz));
                            }
                        }
                    }
                }

                // Handle Left-Click Editing
                if response.clicked_by(egui::PointerButton::Primary) {
                    match self.active_tool {
                        ToolMode::Voxel => {
                            if let Some(place_coord) = self.ghost_pos {
                                let old_id = self.grid.get_coord(place_coord);
                                self.grid.set_coord(place_coord, self.selected_palette_id);
                                self.history.record(EditCommand::new(
                                    "Place Voxel",
                                    vec![VoxelDiff {
                                        coord: place_coord,
                                        old_id,
                                        new_id: self.selected_palette_id,
                                    }],
                                ));
                                self.rebuild_mesh();
                                self.status_message = format!("Placed voxel at ({}, {}, {})", place_coord.x, place_coord.y, place_coord.z);
                            }
                        }
                        ToolMode::Erase => {
                            if let Some(hit) = self.hover_hit {
                                self.grid.set_coord(hit.voxel_coord, EMPTY_VOXEL);
                                self.history.record(EditCommand::new(
                                    "Erase Voxel",
                                    vec![VoxelDiff {
                                        coord: hit.voxel_coord,
                                        old_id: hit.voxel_id,
                                        new_id: EMPTY_VOXEL,
                                    }],
                                ));
                                self.rebuild_mesh();
                                self.status_message = format!("Erased voxel at ({}, {}, {})", hit.voxel_coord.x, hit.voxel_coord.y, hit.voxel_coord.z);
                            }
                        }
                        ToolMode::Paint => {
                            if let Some(hit) = self.hover_hit {
                                self.grid.set_coord(hit.voxel_coord, self.selected_palette_id);
                                self.history.record(EditCommand::new(
                                    "Paint Voxel",
                                    vec![VoxelDiff {
                                        coord: hit.voxel_coord,
                                        old_id: hit.voxel_id,
                                        new_id: self.selected_palette_id,
                                    }],
                                ));
                                self.rebuild_mesh();
                                self.status_message = format!("Painted voxel at ({}, {}, {})", hit.voxel_coord.x, hit.voxel_coord.y, hit.voxel_coord.z);
                            }
                        }
                    }
                }
            }

            // Render the 3D Viewport
            render_3d_viewport(
                ui,
                rect,
                ViewportRenderParams {
                    camera: &self.camera,
                    mesh: &self.mesh,
                    palette: &self.palette,
                    grid: &self.grid,
                    hover_hit: self.hover_hit,
                    ghost_pos: self.ghost_pos,
                    show_grid: self.show_grid,
                    show_box: self.show_box,
                    bg_color: self.bg_color,
                    is_transparent: self.is_transparent_bg,
                    backdrop_texture: self.backdrop_texture.as_ref().map(|t| t.id()),
                },
            );
        });
    }
}
