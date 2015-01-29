use std::default::Default;
use std::io::{File,BufferedWriter,BufferedReader};
use std::ops::{Add,Mul};
use std::borrow::ToOwned;
use std::num::Float;

macro_rules! clamp(
    ($a:expr, $min:expr, $max:expr) => ($a.min($max).max($min));
);

#[derive(Copy)]
pub struct RgbaColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

impl RgbaColor {
    #[inline(always)]
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> RgbaColor {
        RgbaColor {r: r, g: g, b: b, a: a}
    }

    pub fn new_from_u8(r: u8, g: u8, b: u8, a: u8) -> RgbaColor {
        RgbaColor::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0)
    }

    pub fn clamp(&mut self) -> RgbaColor {
        self.r = clamp!(self.r, 0.0, 1.0);
        self.g = clamp!(self.g, 0.0, 1.0);
        self.b = clamp!(self.b, 0.0, 1.0);
        self.a = clamp!(self.a, 0.0, 1.0);
        *self
    }
}

impl ToOwned<RgbaColor> for RgbaColor {
    fn to_owned(&self) -> RgbaColor { *self.clone() }
}

impl Mul<f32> for RgbaColor {
    type Output = RgbaColor;

    #[inline(always)]
    fn mul(self, rhs: f32) -> RgbaColor {
        let r = self.r * rhs;
        let g = self.g * rhs;
        let b = self.b * rhs;

        let mut c = RgbaColor {r: r, g: g, b: b, a: self.a};
        c.clamp()
    }
}

impl Add<RgbaColor> for RgbaColor {
    type Output = RgbaColor;

    #[inline(always)]
    fn add(self, rhs: RgbaColor) -> RgbaColor {
        let mut result = RgbaColor::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b, self.a);
        result.clamp()
    }
}

#[derive(Default)]
pub struct TgaPixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8
}

impl TgaPixel {
    pub fn set_color(&mut self, color: &RgbaColor) {
        let c = color.to_owned().clamp();
        self.r = (c.r * 255.0) as u8;
        self.g = (c.g * 255.0) as u8;
        self.b = (c.b * 255.0) as u8;
        self.a = (c.a * 255.0) as u8;
    }

    pub fn get_color(&self) -> RgbaColor {
        RgbaColor::new_from_u8(self.r, self.g, self.b, self.a)
    }
}

pub struct TgaImage {
    pub width: i32,
    pub height: i32,
    pixels: Vec<TgaPixel>
}

impl TgaImage {
    pub fn new(width: i32, height: i32) -> TgaImage {
        assert!(width > 0, "width must be positive");
        assert!(height > 0, "height must be positive");

        let mut pixels: Vec<TgaPixel> = Vec::with_capacity((width * height) as usize);
        for _ in range(0, width * height) {
            pixels.push(Default::default())
        }

        return TgaImage { width: width, height: height, pixels: pixels };
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: &RgbaColor) {
        match self.pixels.get_mut((x + self.height * y) as usize) {
            Some(pixel) => pixel.set_color(color),
            None => return
        };
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> RgbaColor {
        match self.pixels.get((x + self.height * y) as usize) {
            Some(pixel) => { pixel.get_color() },
            None => { panic!("Can't read pixel at x: {}, y: {}", x, y) }
        }
    }

    pub fn write_to_file(&self, filename: Path) {
        let file = match File::create(&filename) {
            Err(why) => panic!("couldn't create {}: {}", filename.display(), why.desc),
            Ok(file) => file
        };

        // Write TGA Header, data type 2, 24 bytes per pixel
        let mut writer = BufferedWriter::new(file);
        
        writer.write(&[0,0,2,0,0,0,0,0,0,0,0,0]).unwrap();
        writer.write_le_u16(self.width as u16).unwrap();
        writer.write_le_u16(self.height as u16).unwrap();
        writer.write(&[24,0]).unwrap();

        for p in self.pixels.iter() {
            writer.write_u8(p.b).unwrap();
            writer.write_u8(p.g).unwrap();
            writer.write_u8(p.r).unwrap();
        }

        writer.flush().unwrap();
    }

    pub fn new_from_file(filename: &Path) -> TgaImage {
        let file = match File::open(filename) {
            Err(why) => panic!("couldn't open {}: {}", filename.display(), why.desc),
            Ok(file) => file
        };

        let mut reader = BufferedReader::new(file);

        let id_len = reader.read_u8().unwrap();
        let color_map_type = reader.read_u8().unwrap();
        let data_type = reader.read_u8().unwrap();
        reader.read_exact(5).unwrap(); // skip color map info
        let x_origin = reader.read_le_i16().unwrap();
        let y_origin = reader.read_le_i16().unwrap();
        let width = reader.read_le_i16().unwrap() as i32;
        let height = reader.read_le_i16().unwrap() as i32;
        let bpp = reader.read_u8().unwrap();
        let img_desc = reader.read_u8().unwrap();

        if (color_map_type != 0) {
            panic!("Can't read files with color map");
        }

        reader.read_exact(id_len as usize);
        // TODO: Read/skip color map data?

        let mut image = TgaImage::new(width, height);
        let mut pixel:i32 = 0;

        while pixel < width * height {
            let packet = reader.read_u8().unwrap();
            let count = (packet & 127) + 1;
            let mut bgr:Vec<u8> = vec![0, 0, 0];

            if packet & 128 > 0 {
                bgr = reader.read_exact(3).unwrap();
                for _ in range(0, count) {
                    image.set_pixel(pixel % width, pixel / width, &RgbaColor::new_from_u8(bgr[2], bgr[1], bgr[0], 255));
                    pixel += 1;
                }
            } else {
                for _ in range(0, count) {
                    bgr = reader.read_exact(3).unwrap();
                    image.set_pixel(pixel % width, pixel / width, &RgbaColor::new_from_u8(bgr[2], bgr[1], bgr[0], 255));
                    pixel += 1;
                }
            }
        }
        
        return image;
    }
}
