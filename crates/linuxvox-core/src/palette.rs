/// An 8-bit RGBA color representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Converts color to normalized [0.0, 1.0] RGBA floats.
    #[inline]
    pub fn to_normalized(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    /// Converts to hex string (e.g. "#RRGGBB").
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// A 256-color palette matching MagicaVoxel's color structure.
/// Index 0 is reserved for Empty/Air (transparent).
/// Indices 1..=255 hold usable artistic colors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    pub colors: [Color; 256],
}

impl Default for Palette {
    fn default() -> Self {
        Self::magica_default()
    }
}

impl Palette {
    /// Creates a palette with bright, saturated, flashy colors matching MagicaVoxel's aesthetic.
    pub fn magica_default() -> Self {
        let mut colors = [Color::TRANSPARENT; 256];

        // Slot 0 is reserved for empty/air
        colors[0] = Color::TRANSPARENT;

        // Grayscale ramp from pure bright white to dark charcoal (indices 1 to 16)
        for i in 1..=16 {
            let shade = (255 - ((i - 1) * 15)).clamp(20, 255) as u8;
            colors[i] = Color::new(shade, shade, shade, 255);
        }

        // Generate full-vibrancy, punchy, saturated spectrum for remaining slots (17..=255)
        for i in 17..=255 {
            let idx = (i - 17) as f32;
            let total = (255 - 17) as f32;
            let hue = (idx / total * 360.0) % 360.0;
            
            // Full, flashy saturation and high value
            let sat_step = (i % 4) as f32;
            let sat = match sat_step as i32 {
                0 => 1.0,  // Pure vivid neon / primary
                1 => 0.85, // Rich saturated
                2 => 0.65, // Bright pastel
                _ => 0.95, // Flashy pop
            };
            
            let val_step = ((i / 4) % 3) as f32;
            let val = match val_step as i32 {
                0 => 1.0,  // Maximum brightness
                1 => 0.92, // Bright
                _ => 0.82, // Deep rich
            };

            let (r, g, b) = hsv_to_rgb(hue, sat, val);
            colors[i] = Color::new(r, g, b, 255);
        }

        Self { colors }
    }

    // Standard Named Color Palette Indices
    pub const WHITE: u8 = 1;
    pub const LIGHT_GRAY: u8 = 5;
    pub const MID_GRAY: u8 = 9;
    pub const DARK_GRAY: u8 = 13;
    pub const CHARCOAL: u8 = 16;

    pub const VIVID_RED: u8 = 17;
    pub const CRIMSON: u8 = 21;
    pub const BRIGHT_ORANGE: u8 = 37;
    pub const FLAME_ORANGE: u8 = 41;
    pub const GOLD_YELLOW: u8 = 57;
    pub const SUNSHINE_YELLOW: u8 = 61;
    pub const LIME_GREEN: u8 = 93;
    pub const EMERALD_GREEN: u8 = 105;
    pub const ELECTRIC_CYAN: u8 = 137;
    pub const SKY_BLUE: u8 = 145;
    pub const COBALT_BLUE: u8 = 177;
    pub const ROYAL_BLUE: u8 = 185;
    pub const NEON_PURPLE: u8 = 205;
    pub const HOT_MAGENTA: u8 = 217;
    pub const WARM_WOOD: u8 = 38;
    pub const SANDSTONE: u8 = 58;

    #[inline]
    pub fn get(&self, index: u8) -> Color {
        self.colors[index as usize]
    }

    #[inline]
    pub fn set(&mut self, index: u8, color: Color) {
        self.colors[index as usize] = color;
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}
