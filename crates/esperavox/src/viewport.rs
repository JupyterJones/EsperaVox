use eframe::egui::{self, Color32, Pos2, Rect, Stroke};
use glam::{Vec3, Vec4};
use linuxvox_core::{Mesh, Palette, RayHit, VoxelGrid};
use crate::camera::OrbitCamera;

pub struct ViewportRenderParams<'a> {
    pub camera: &'a OrbitCamera,
    pub mesh: &'a Mesh,
    #[allow(dead_code)]
    pub palette: &'a Palette,
    pub grid: &'a VoxelGrid,
    pub hover_hit: Option<RayHit>,
    pub ghost_pos: Option<glam::IVec3>,
    pub show_grid: bool,
    pub show_box: bool,
    pub bg_color: Color32,
    pub is_transparent: bool,
    pub backdrop_texture: Option<egui::TextureId>,
}

/// Renders the 3D voxel viewport onto an egui painter.
pub fn render_3d_viewport(ui: &egui::Ui, rect: Rect, params: ViewportRenderParams) {
    let painter = ui.painter_at(rect);

    // 0. Render Background (Image plane, solid color, or checkerboard for transparent)
    if let Some(tex_id) = params.backdrop_texture {
        painter.image(
            tex_id,
            rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    } else if params.is_transparent {
        let check_size = 16.0;
        let c1 = Color32::from_rgb(32, 36, 44);
        let c2 = Color32::from_rgb(22, 26, 32);
        let cols = (rect.width() / check_size).ceil() as usize;
        let rows = (rect.height() / check_size).ceil() as usize;
        for r in 0..rows {
            for c in 0..cols {
                let col = if (r + c) % 2 == 0 { c1 } else { c2 };
                let cell_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x + c as f32 * check_size, rect.min.y + r as f32 * check_size),
                    egui::vec2(check_size, check_size),
                );
                painter.rect_filled(cell_rect.intersect(rect), 0.0, col);
            }
        }
    } else {
        painter.rect_filled(rect, 0.0, params.bg_color);
    }

    let aspect = (rect.width() / rect.height()).max(0.001);

    let view = params.camera.view_matrix();
    let proj = params.camera.proj_matrix(aspect);
    let view_proj = proj * view;
    let eye = params.camera.eye_position();

    let screen_w = rect.width();
    let screen_h = rect.height();
    let rect_min = rect.min;

    // Project a 3D point to 2D screen coordinates
    let project_point = |p: Vec3| -> Option<(Pos2, f32)> {
        let clip = view_proj * Vec4::new(p.x, p.y, p.z, 1.0);
        if clip.w <= 0.05 {
            return None;
        }
        let ndc = Vec3::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w);
        if ndc.z < -1.0 || ndc.z > 1.0 {
            return None;
        }
        let sx = rect_min.x + (ndc.x * 0.5 + 0.5) * screen_w;
        let sy = rect_min.y + ((1.0 - (ndc.y * 0.5 + 0.5)) * screen_h);
        Some((Pos2::new(sx, sy), clip.w))
    };

    // 1. Draw 3D Ground Grid (Floor at Y = 0)
    if params.show_grid {
        let grid_size = params.grid.size_x.max(params.grid.size_z) as i32;
        let grid_stroke = Stroke::new(1.0, Color32::from_rgb(45, 52, 64));
        let center_stroke = Stroke::new(1.5, Color32::from_rgb(70, 80, 95));

        for i in 0..=grid_size {
            let z = i as f32;
            if let (Some((p1, _)), Some((p2, _))) = (
                project_point(Vec3::new(0.0, 0.0, z)),
                project_point(Vec3::new(grid_size as f32, 0.0, z)),
            ) {
                let stroke = if i == 0 || i == grid_size { center_stroke } else { grid_stroke };
                painter.line_segment([p1, p2], stroke);
            }

            let x = i as f32;
            if let (Some((p1, _)), Some((p2, _))) = (
                project_point(Vec3::new(x, 0.0, 0.0)),
                project_point(Vec3::new(x, 0.0, grid_size as f32)),
            ) {
                let stroke = if i == 0 || i == grid_size { center_stroke } else { grid_stroke };
                painter.line_segment([p1, p2], stroke);
            }
        }
    }

    // 2. Draw Bounding Box of Volume
    if params.show_box {
        let max_p = Vec3::new(
            params.grid.size_x as f32,
            params.grid.size_y as f32,
            params.grid.size_z as f32,
        );
        let box_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 120, 150, 100));

        let corners = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(max_p.x, 0.0, 0.0),
            Vec3::new(max_p.x, 0.0, max_p.z),
            Vec3::new(0.0, 0.0, max_p.z),
            Vec3::new(0.0, max_p.y, 0.0),
            Vec3::new(max_p.x, max_p.y, 0.0),
            Vec3::new(max_p.x, max_p.y, max_p.z),
            Vec3::new(0.0, max_p.y, max_p.z),
        ];

        let edges = [
            (0, 1), (1, 2), (2, 3), (3, 0), // bottom
            (4, 5), (5, 6), (6, 7), (7, 4), // top
            (0, 4), (1, 5), (2, 6), (3, 7), // pillars
        ];

        for (i, j) in edges {
            if let (Some((p1, _)), Some((p2, _))) = (
                project_point(corners[i]),
                project_point(corners[j]),
            ) {
                painter.line_segment([p1, p2], box_stroke);
            }
        }
    }

    // 3. Render 3D Voxel Triangles with Lighting & Backface Culling
    if !params.mesh.is_empty() {
        let light_dir = Vec3::new(0.4, 0.8, 0.5).normalize();

        struct TriRender {
            pts: [Pos2; 3],
            depth: f32,
            color: Color32,
        }

        let mut tris: Vec<TriRender> = Vec::with_capacity(params.mesh.indices.len() / 3);

        for chunk in params.mesh.indices.chunks_exact(3) {
            let v0 = &params.mesh.vertices[chunk[0] as usize];
            let v1 = &params.mesh.vertices[chunk[1] as usize];
            let v2 = &params.mesh.vertices[chunk[2] as usize];

            let p0 = Vec3::from_slice(&v0.position);
            let p1 = Vec3::from_slice(&v1.position);
            let p2 = Vec3::from_slice(&v2.position);
            let normal = Vec3::from_slice(&v0.normal);

            // Backface culling: Vector from eye to face centroid
            let centroid = (p0 + p1 + p2) / 3.0;
            let view_dir = (centroid - eye).normalize();
            if normal.dot(view_dir) >= 0.0 {
                continue; // Cull backface
            }

            if let (Some((sp0, d0)), Some((sp1, d1)), Some((sp2, d2))) = (
                project_point(p0),
                project_point(p1),
                project_point(p2),
            ) {
                let depth = (d0 + d1 + d2) / 3.0;

                // Vibrant MagicaVoxel-style lighting with high ambient and gamma lift
                let diffuse = (normal.dot(light_dir)).max(0.0) * 0.45 + 0.55;
                let top_boost = if normal.y > 0.5 { 0.12 } else { 0.0 };
                let total = (diffuse + top_boost).min(1.25);
                let r = ((v0.color[0] * total).clamp(0.0, 1.0).powf(0.85) * 255.0).round() as u8;
                let g = ((v0.color[1] * total).clamp(0.0, 1.0).powf(0.85) * 255.0).round() as u8;
                let b = ((v0.color[2] * total).clamp(0.0, 1.0).powf(0.85) * 255.0).round() as u8;
                let col = Color32::from_rgb(r, g, b);

                tris.push(TriRender {
                    pts: [sp0, sp1, sp2],
                    depth,
                    color: col,
                });
            }
        }

        // Depth sorting (Painters algorithm for crisp rendering)
        tris.sort_unstable_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));

        let mut epaint_mesh = egui::Mesh::default();
        for tri in tris {
            let base_idx = epaint_mesh.vertices.len() as u32;
            for pt in tri.pts {
                epaint_mesh.vertices.push(egui::epaint::Vertex {
                    pos: pt,
                    uv: egui::epaint::WHITE_UV,
                    color: tri.color,
                });
            }
            epaint_mesh.indices.extend_from_slice(&[
                base_idx,
                base_idx + 1,
                base_idx + 2,
            ]);
        }

        painter.add(egui::Shape::Mesh(epaint_mesh));
    }

    // 4. Draw Ghost/Placement Guide Outline
    if let Some(ghost) = params.ghost_pos {
        draw_cube_wireframe(
            &project_point,
            &painter,
            ghost.as_vec3(),
            Stroke::new(2.0, Color32::from_rgb(100, 255, 120)),
        );
    } else if let Some(hit) = params.hover_hit {
        draw_cube_wireframe(
            &project_point,
            &painter,
            hit.voxel_coord.as_vec3(),
            Stroke::new(2.0, Color32::from_rgb(255, 230, 80)),
        );
    }
}

fn draw_cube_wireframe<F>(
    project: &F,
    painter: &egui::Painter,
    origin: Vec3,
    stroke: Stroke,
) where
    F: Fn(Vec3) -> Option<(Pos2, f32)>,
{
    let corners = [
        origin + Vec3::new(0.0, 0.0, 0.0),
        origin + Vec3::new(1.0, 0.0, 0.0),
        origin + Vec3::new(1.0, 0.0, 1.0),
        origin + Vec3::new(0.0, 0.0, 1.0),
        origin + Vec3::new(0.0, 1.0, 0.0),
        origin + Vec3::new(1.0, 1.0, 0.0),
        origin + Vec3::new(1.0, 1.0, 1.0),
        origin + Vec3::new(0.0, 1.0, 1.0),
    ];

    let edges = [
        (0, 1), (1, 2), (2, 3), (3, 0),
        (4, 5), (5, 6), (6, 7), (7, 4),
        (0, 4), (1, 5), (2, 6), (3, 7),
    ];

    for (i, j) in edges {
        if let (Some((p1, _)), Some((p2, _))) = (project(corners[i]), project(corners[j])) {
            painter.line_segment([p1, p2], stroke);
        }
    }
}
