use glam::{Mat4, Vec2, Vec3};

/// A 3D Orbit camera for smooth inspection and sculpting.
#[derive(Debug, Clone)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub fov_y: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Vec3::new(16.0, 16.0, 16.0),
            yaw: std::f32::consts::FRAC_PI_4, // 45 degrees
            pitch: 0.5,                       // ~28 degrees
            distance: 60.0,
            fov_y: 45.0f32.to_radians(),
        }
    }
}

impl OrbitCamera {
    pub fn eye_position(&self) -> Vec3 {
        let cos_p = self.pitch.cos();
        let offset = Vec3::new(
            self.distance * cos_p * self.yaw.sin(),
            self.distance * self.pitch.sin(),
            self.distance * self.cos_pitch_cos_yaw(),
        );
        self.target + offset
    }

    #[inline]
    fn cos_pitch_cos_yaw(&self) -> f32 {
        self.pitch.cos() * self.yaw.cos()
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye_position(), self.target, Vec3::Y)
    }

    pub fn proj_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, aspect, 0.1, 1000.0)
    }

    pub fn forward(&self) -> Vec3 {
        (self.target - self.eye_position()).normalize()
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    pub fn up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalize()
    }

    pub fn orbit(&mut self, delta: Vec2) {
        self.yaw += delta.x * 0.008;
        self.pitch = (self.pitch - delta.y * 0.008).clamp(-1.52, 1.52);
    }

    pub fn pan(&mut self, delta: Vec2) {
        let factor = self.distance * 0.0015;
        let right = self.right();
        let up = self.up();
        self.target -= right * (delta.x * factor);
        self.target += up * (delta.y * factor);
    }

    pub fn zoom(&mut self, scroll: f32) {
        let zoom_factor = (1.0 - scroll * 0.1).clamp(0.05, 5.0);
        self.distance = (self.distance * zoom_factor).clamp(2.0, 500.0);
    }

    /// Casts a ray from screen coordinates (relative to viewport rect) into world space.
    pub fn screen_to_ray(&self, screen_pos: Vec2, viewport_size: Vec2) -> (Vec3, Vec3) {
        let ndc_x = (2.0 * screen_pos.x / viewport_size.x) - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y / viewport_size.y);

        let aspect = (viewport_size.x / viewport_size.y).max(0.001);
        let view = self.view_matrix();
        let proj = self.proj_matrix(aspect);
        let inv_view_proj = (proj * view).inverse();

        let near_clip = inv_view_proj.project_point3(Vec3::new(ndc_x, ndc_y, -1.0));
        let far_clip = inv_view_proj.project_point3(Vec3::new(ndc_x, ndc_y, 1.0));

        let ray_dir = (far_clip - near_clip).normalize();
        (near_clip, ray_dir)
    }
}
