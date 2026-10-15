use glam::Vec3;
use crate::renderer::RenderOptions;

/// A single camera waypoint in 3D cinematic space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraKeyframe {
    pub yaw: f32,       // Radians
    pub pitch: f32,     // Radians
    pub distance: f32,  // Units from target
    pub target: Vec3,   // Focus point
    pub time: f32,      // Timestamp (seconds or normalized 0.0 - 1.0)
}

impl CameraKeyframe {
    pub fn new(yaw: f32, pitch: f32, distance: f32, target: Vec3, time: f32) -> Self {
        Self {
            yaw,
            pitch,
            distance,
            target,
            time,
        }
    }
}

/// A smooth spline-based camera track through multiple keyframes.
#[derive(Debug, Clone)]
pub struct CameraTrack {
    pub keyframes: Vec<CameraKeyframe>,
    pub is_looping: bool,
}

impl Default for CameraTrack {
    fn default() -> Self {
        Self {
            keyframes: Vec::new(),
            is_looping: true,
        }
    }
}

impl CameraTrack {
    pub fn new(keyframes: Vec<CameraKeyframe>, is_looping: bool) -> Self {
        Self {
            keyframes,
            is_looping,
        }
    }

    pub fn add_keyframe(&mut self, kf: CameraKeyframe) {
        self.keyframes.push(kf);
        // Sort by time
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn remove_keyframe(&mut self, idx: usize) {
        if idx < self.keyframes.len() {
            self.keyframes.remove(idx);
        }
    }

    pub fn clear(&mut self) {
        self.keyframes.clear();
    }

    pub fn len(&self) -> usize {
        self.keyframes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }

    /// Evaluates the camera state at normalized progress t in [0.0, 1.0].
    pub fn evaluate(&self, mut t: f32) -> RenderOptions {
        if self.keyframes.is_empty() {
            return RenderOptions::default();
        }
        if self.keyframes.len() == 1 {
            let k = &self.keyframes[0];
            return RenderOptions {
                yaw: k.yaw,
                pitch: k.pitch,
                distance: k.distance,
                target: k.target,
                ..Default::default()
            };
        }

        t = t.clamp(0.0, 1.0);

        let n = self.keyframes.len();
        // If looping, the path wraps around to keyframe 0
        let segment_count = if self.is_looping { n } else { n - 1 };
        let scaled_t = t * segment_count as f32;
        let seg_idx = (scaled_t.floor() as usize).min(segment_count - 1);
        let seg_frac = scaled_t - seg_idx as f32;

        // Apply smooth ease-in-out curve to segment fraction
        let smooth_frac = smoothstep(seg_frac);

        // Get 4 control points for Catmull-Rom spline (p0, p1, p2, p3)
        let (p0, p1, p2, p3) = if self.is_looping {
            let i0 = (seg_idx + n - 1) % n;
            let i1 = seg_idx % n;
            let i2 = (seg_idx + 1) % n;
            let i3 = (seg_idx + 2) % n;
            (
                &self.keyframes[i0],
                &self.keyframes[i1],
                &self.keyframes[i2],
                &self.keyframes[i3],
            )
        } else {
            let i0 = if seg_idx > 0 { seg_idx - 1 } else { 0 };
            let i1 = seg_idx;
            let i2 = (seg_idx + 1).min(n - 1);
            let i3 = (seg_idx + 2).min(n - 1);
            (
                &self.keyframes[i0],
                &self.keyframes[i1],
                &self.keyframes[i2],
                &self.keyframes[i3],
            )
        };

        // Angular wrapping interpolation for yaw
        let yaw = catmull_rom_angle(p0.yaw, p1.yaw, p2.yaw, p3.yaw, smooth_frac);
        let pitch = catmull_rom(p0.pitch, p1.pitch, p2.pitch, p3.pitch, smooth_frac);
        let distance = catmull_rom(p0.distance, p1.distance, p2.distance, p3.distance, smooth_frac);
        let target = Vec3::new(
            catmull_rom(p0.target.x, p1.target.x, p2.target.x, p3.target.x, smooth_frac),
            catmull_rom(p0.target.y, p1.target.y, p2.target.y, p3.target.y, smooth_frac),
            catmull_rom(p0.target.z, p1.target.z, p2.target.z, p3.target.z, smooth_frac),
        );

        RenderOptions {
            yaw,
            pitch,
            distance,
            target,
            ..Default::default()
        }
    }
}

/// Standard cubic Catmull-Rom spline evaluation.
pub fn catmull_rom(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;

    0.5 * (
        (2.0 * p1) +
        (-p0 + p2) * t +
        (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2 +
        (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3
    )
}

/// Angular-aware Catmull-Rom spline handling wrap-around for rotation angles.
fn catmull_rom_angle(mut a0: f32, a1: f32, mut a2: f32, mut a3: f32, t: f32) -> f32 {
    use std::f32::consts::TAU;

    // Normalize differences relative to a1
    while a0 - a1 > std::f32::consts::PI { a0 -= TAU; }
    while a0 - a1 < -std::f32::consts::PI { a0 += TAU; }

    while a2 - a1 > std::f32::consts::PI { a2 -= TAU; }
    while a2 - a1 < -std::f32::consts::PI { a2 += TAU; }

    while a3 - a2 > std::f32::consts::PI { a3 -= TAU; }
    while a3 - a2 < -std::f32::consts::PI { a3 += TAU; }

    catmull_rom(a0, a1, a2, a3, t)
}

/// Cubic smoothstep easing: 3t^2 - 2t^3
#[inline]
fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
