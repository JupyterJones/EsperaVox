use glam::{Mat4, Vec2, Vec3, Vec4};
use image::{Rgba, RgbaImage};
use crate::mesher::Mesh;

/// Background styling mode for renders.
#[derive(Debug, Clone)]
pub enum BackgroundMode {
    /// Radial studio vignette gradient.
    StudioGradient,
    /// Transparent alpha channel (RGBA [0, 0, 0, 0]).
    Transparent,
    /// Solid flat color (e.g. pure black, pure white, or custom RGBA).
    Solid([u8; 4]),
    /// A custom 2D image plane backdrop.
    Image(RgbaImage),
}

impl PartialEq for BackgroundMode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StudioGradient, Self::StudioGradient) => true,
            (Self::Transparent, Self::Transparent) => true,
            (Self::Solid(a), Self::Solid(b)) => a == b,
            (Self::Image(a), Self::Image(b)) => a.dimensions() == b.dimensions(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub width: u32,
    pub height: u32,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target: Vec3,
    pub is_isometric: bool,
    pub background: BackgroundMode,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 800,
            height: 800,
            yaw: 0.785398,   // 45 degrees
            pitch: 0.523599,  // 30 degrees (isometric angle)
            distance: 50.0,
            target: Vec3::new(16.0, 16.0, 16.0),
            is_isometric: false,
            background: BackgroundMode::StudioGradient,
        }
    }
}

/// Renders a `Mesh` into an offscreen RGBA image using high-speed software rasterization.
pub fn render_mesh_to_image(mesh: &Mesh, options: &RenderOptions) -> RgbaImage {
    let width = options.width;
    let height = options.height;
    let aspect = width as f32 / height as f32;

    let cos_p = options.pitch.cos();
    let eye = options.target + Vec3::new(
        options.distance * cos_p * options.yaw.sin(),
        options.distance * options.pitch.sin(),
        options.distance * cos_p * options.yaw.cos(),
    );

    let view = Mat4::look_at_rh(eye, options.target, Vec3::Y);
    let proj = if options.is_isometric {
        let ortho_half = options.distance * 0.45;
        Mat4::orthographic_rh(
            -ortho_half * aspect,
            ortho_half * aspect,
            -ortho_half,
            ortho_half,
            0.1,
            2000.0,
        )
    } else {
        Mat4::perspective_rh(45.0f32.to_radians(), aspect, 0.1, 2000.0)
    };
    let view_proj = proj * view;

    let mut img = RgbaImage::new(width, height);
    let mut depth_buffer = vec![f32::INFINITY; (width * height) as usize];

    // Background fill (Studio gradient, transparent, solid color, or custom image plane)
    match &options.background {
        BackgroundMode::Transparent => {
            for p in img.pixels_mut() {
                *p = Rgba([0, 0, 0, 0]);
            }
        }
        BackgroundMode::Solid(c) => {
            let col = Rgba(*c);
            for p in img.pixels_mut() {
                *p = col;
            }
        }
        BackgroundMode::Image(bg_img) => {
            let bg_w = bg_img.width();
            let bg_h = bg_img.height();
            if bg_w > 0 && bg_h > 0 {
                for y in 0..height {
                    let sy = ((y as f32 / height as f32) * bg_h as f32).min((bg_h - 1) as f32) as u32;
                    for x in 0..width {
                        let sx = ((x as f32 / width as f32) * bg_w as f32).min((bg_w - 1) as f32) as u32;
                        img.put_pixel(x, y, *bg_img.get_pixel(sx, sy));
                    }
                }
            }
        }
        BackgroundMode::StudioGradient => {
            for y in 0..height {
                let ny = (y as f32 / height as f32) - 0.5;
                for x in 0..width {
                    let nx = (x as f32 / width as f32) - 0.5;
                    let dist = (nx * nx + ny * ny).sqrt();

                    let t = (dist * 1.4).clamp(0.0, 1.0);
                    let r = (27.0 * (1.0 - t) + 15.0 * t) as u8;
                    let g = (33.0 * (1.0 - t) + 18.0 * t) as u8;
                    let b = (44.0 * (1.0 - t) + 24.0 * t) as u8;
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
        }
    }

    if mesh.is_empty() {
        return img;
    }

    // Directional lighting setup
    let key_light = Vec3::new(0.5, 0.8, 0.4).normalize();
    let fill_light = Vec3::new(-0.4, 0.3, -0.6).normalize();

    let half_w = width as f32 * 0.5;
    let half_h = height as f32 * 0.5;

    // Rasterize all triangles
    for chunk in mesh.indices.chunks_exact(3) {
        let v0 = &mesh.vertices[chunk[0] as usize];
        let v1 = &mesh.vertices[chunk[1] as usize];
        let v2 = &mesh.vertices[chunk[2] as usize];

        let p0 = Vec3::from_slice(&v0.position);
        let p1 = Vec3::from_slice(&v1.position);
        let p2 = Vec3::from_slice(&v2.position);
        let normal = Vec3::from_slice(&v0.normal);

        // Project vertices to clip space
        let c0 = view_proj * Vec4::new(p0.x, p0.y, p0.z, 1.0);
        let c1 = view_proj * Vec4::new(p1.x, p1.y, p1.z, 1.0);
        let c2 = view_proj * Vec4::new(p2.x, p2.y, p2.z, 1.0);

        if c0.w <= 0.001 || c1.w <= 0.001 || c2.w <= 0.001 {
            continue;
        }

        // Screen coordinates
        let s0 = Vec2::new((c0.x / c0.w + 1.0) * half_w, (1.0 - c0.y / c0.w) * half_h);
        let s1 = Vec2::new((c1.x / c1.w + 1.0) * half_w, (1.0 - c1.y / c1.w) * half_h);
        let s2 = Vec2::new((c2.x / c2.w + 1.0) * half_w, (1.0 - c2.y / c2.w) * half_h);

        // Backface culling via 2D cross product
        let area = (s1.x - s0.x) * (s2.y - s0.y) - (s1.y - s0.y) * (s2.x - s0.x);
        if area <= 0.0 {
            continue;
        }
        let inv_area = 1.0 / area;

        // Bounding box
        let min_x = (s0.x.min(s1.x).min(s2.x).floor() as i32).max(0);
        let max_x = (s0.x.max(s1.x).max(s2.x).ceil() as i32).min(width as i32 - 1);
        let min_y = (s0.y.min(s1.y).min(s2.y).floor() as i32).max(0);
        let max_y = (s0.y.max(s1.y).max(s2.y).ceil() as i32).min(height as i32 - 1);

        if min_x > max_x || min_y > max_y {
            continue;
        }

        // Vibrant MagicaVoxel-style lighting calculation
        let key_diffuse = normal.dot(key_light).max(0.0) * 0.55;
        let fill_diffuse = normal.dot(fill_light).max(0.0) * 0.20;
        let top_boost = if normal.y > 0.5 { 0.15 } else { 0.0 };
        let ambient = 0.50; // High ambient preserves full color saturation in shadows
        let total_diffuse = (key_diffuse + fill_diffuse + top_boost + ambient).min(1.3);

        // Subtle glossy specular sheen
        let view_dir = (eye - p0).normalize();
        let half_vec = (key_light + view_dir).normalize();
        let specular = normal.dot(half_vec).max(0.0).powf(24.0) * 0.25;

        let base_col = v0.color;
        // Gamma curve lifting shadows so colors remain punchy and flashy
        let r_lin = (base_col[0] * total_diffuse + specular).clamp(0.0, 1.0);
        let g_lin = (base_col[1] * total_diffuse + specular).clamp(0.0, 1.0);
        let b_lin = (base_col[2] * total_diffuse + specular).clamp(0.0, 1.0);

        let r = (r_lin.powf(0.85) * 255.0).round().clamp(0.0, 255.0) as u8;
        let g = (g_lin.powf(0.85) * 255.0).round().clamp(0.0, 255.0) as u8;
        let b = (b_lin.powf(0.85) * 255.0).round().clamp(0.0, 255.0) as u8;
        let pixel_color = Rgba([r, g, b, 255]);

        let z0 = c0.z / c0.w;
        let z1 = c1.z / c1.w;
        let z2 = c2.z / c2.w;

        for py in min_y..=max_y {
            let p_y = py as f32 + 0.5;
            for px in min_x..=max_x {
                let p_x = px as f32 + 0.5;

                // Barycentric edge checks
                let w0 = ((s1.x - p_x) * (s2.y - p_y) - (s1.y - p_y) * (s2.x - p_x)) * inv_area;
                let w1 = ((s2.x - p_x) * (s0.y - p_y) - (s2.y - p_y) * (s0.x - p_x)) * inv_area;
                let w2 = 1.0 - w0 - w1;

                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    let depth = w0 * z0 + w1 * z1 + w2 * z2;
                    let idx = (py as u32 * width + px as u32) as usize;

                    if depth < depth_buffer[idx] {
                        depth_buffer[idx] = depth;
                        img.put_pixel(px as u32, py as u32, pixel_color);
                    }
                }
            }
        }
    }

    img
}
