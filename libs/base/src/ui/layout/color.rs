#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {

    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const RED: Self = Self::new(255, 0, 0, 255);
    pub const GREEN: Self = Self::new(0, 255, 0, 255);
    pub const BLUE: Self = Self::new(0, 0, 255, 255);

    pub const GREY_20:  Self = Self::new(20,  20,  20,  255);
    pub const GREY_30:  Self = Self::new(30,  30,  30,  255);
    pub const GREY_40:  Self = Self::new(40,  40,  40,  255);
    pub const GREY_50:  Self = Self::new(50,  50,  50,  255);
    pub const GREY_60:  Self = Self::new(60,  60,  60,  255);
    pub const GREY_80:  Self = Self::new(80,  80,  80,  255);
    pub const GREY_100: Self = Self::new(100, 100, 100, 255);
    pub const GREY_120: Self = Self::new(120, 120, 120, 255);
    pub const GREY_140: Self = Self::new(140, 140, 140, 255);
    pub const GREY_160: Self = Self::new(160, 160, 160, 255);
    pub const GREY_180: Self = Self::new(180, 180, 180, 255);
    pub const GREY_200: Self = Self::new(200, 200, 200, 255);
    pub const GREY_220: Self = Self::new(220, 220, 220, 255);
    pub const GREY_240: Self = Self::new(240, 240, 240, 255);

    pub const RAINBOW: [Color; 6] = [
        Color { r: 255, g: 0,   b: 0,   a: 255 }, // Vibrant Red
        Color { r: 255, g: 127, b: 0,   a: 255 }, // Vibrant Orange
        Color { r: 255, g: 255, b: 0,   a: 255 }, // Vibrant Yellow
        Color { r: 0,   g: 255, b: 0,   a: 255 }, // Vibrant Green
        Color { r: 0,   g: 0,   b: 255, a: 255 }, // Vibrant Blue
        Color { r: 139, g: 0,   b: 255, a: 255 }, // Vibrant Purple / Violet
    ];

    pub const WATER_OCEANIC: [Color; 7] = [
        Color { r: 10,  g: 35,  b: 75,  a: 255 }, // Saturated Midnight Blue (Lifted shadow)
        Color { r: 0,   g: 64,  b: 128, a: 255 }, // Deep Sapphire
        Color { r: 0,   g: 110, b: 180, a: 255 }, // Classic Clear Water
        Color { r: 0,   g: 150, b: 210, a: 255 }, // Electric Cyan
        Color { r: 64,  g: 224, b: 208, a: 255 }, // Vibrant Turquoise (Kept your excellent choice)
        Color { r: 192, g: 240, b: 255, a: 255 }, // Shallow Aqua / Spray
        Color { r: 255, g: 255, b: 255, a: 255 }, // Foam White
    ];

    pub const WATER_TROPICAL: [Color; 7] = [
        Color { r: 0,   g: 55,  b: 120, a: 255 }, // Strong Ocean Blue (No muddy blacks)
        Color { r: 0,   g: 96,  b: 192, a: 255 }, // Classic Water Blue
        Color { r: 0,   g: 145, b: 220, a: 255 }, // Bright Tropical Blue
        Color { r: 30,  g: 190, b: 230, a: 255 }, // Vivid Sky Blue
        Color { r: 75,  g: 230, b: 215, a: 255 }, // Vibrant Turquoise
        Color { r: 200, g: 245, b: 255, a: 255 }, // Shallow Aqua / Spray
        Color { r: 255, g: 255, b: 255, a: 255 }, // Foam White
    ];



    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn is_visible(self) -> bool {
        self.a > 0
    }

    pub fn to_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn from_array(arr: [u8; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }

    pub fn as_f32_array(self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

   
    // Blend two u8 colors together by a percentage float (t is 0.0 to 1.0)
    #[inline]
    fn blend_colors(c1: &Color, c2: &Color, t: f64) -> Color {
        // 1 - t is pre-calculated to reuse across all channels
        let one_minus_t = 1.0 - t;

        // Direct linear interpolation using float math. 
        // Truncating with `as u8` is significantly faster than `.round()` 
        // and visually indistinguishable for fast-moving particles.
        Color {
            r: (c1.r as f64 * one_minus_t + c2.r as f64 * t) as u8,
            g: (c1.g as f64 * one_minus_t + c2.g as f64 * t) as u8,
            b: (c1.b as f64 * one_minus_t + c2.b as f64 * t) as u8,
            a: (c1.a as f64 * one_minus_t + c2.a as f64 * t) as u8,
        }
    }

    // Get the blended color at any specific percentage (0.0 to 1.0) along the Vec
    #[inline]
    pub fn get_color_at_percentage(colors: &[Color], percentage: f64) -> Color {
        let count = colors.len();
        
        // Handle edge cases safely
        if count == 0 { return Color::default(); }
        if count == 1 { return colors[0]; }
  
        let p = percentage.clamp(0.0, 1.0);
        let scaled_p = p * (count - 1) as f64;

        // Clamp the index to the second-to-last element to handle p = 1.0 safely
        let index = (scaled_p.floor() as usize).min(count - 2);
        let t = (scaled_p - index as f64).clamp(0.0, 1.0);

        Self::blend_colors(&colors[index], &colors[index + 1], t) 
    }



}