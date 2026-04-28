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


}