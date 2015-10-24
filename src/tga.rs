use std::default::Default;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::ops::{Add,Mul};
use std::path::Path;

macro_rules! clamp(
    ($a:expr, $min:expr, $max:expr) => ($a.min($max).max($min));
);

#[derive(Clone,Copy)]
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
        for _ in 0..width*height {
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

    pub fn write_to_file(&self, filename: &Path) {
        let mut file = match File::create(filename) {
            Err(e) => panic!("couldn't create {}: {:?}", filename.display(), e),
            Ok(file) => file
        };

        let mut data = Vec::<u8>::with_capacity((3 * self.width * self.height + 20) as usize);
        
        // TGA Header, data type 2, 24 bytes per pixel
        data.extend(&[0,0,2,0,0,0,0,0,0,0,0,0]);
        data.push((self.width & 0xFF) as u8);
        data.push((self.width >> 8 & 0xFF) as u8);
        data.push((self.height & 0xFF) as u8);
        data.push((self.height >> 8 & 0xFF) as u8);
        data.push(24);
        data.push(0);

        // pixel data
        for p in self.pixels.iter() {
            data.push(p.b);
            data.push(p.g);
            data.push(p.r);
        }

        file.write_all(&data[..]);
    }

    pub fn new_from_file(filename: &Path) -> TgaImage {
        let mut file = match File::open(filename) {
            Err(e) => panic!("couldn't open {}: {:?}", filename.display(), e),
            Ok(file) => file
        };
        
        let mut buffer = Vec::<u8>::new();
        file.read_to_end(&mut buffer);
        
        let mut data = buffer.into_iter();

        let id_len = data.next().unwrap();
        let color_map_type = data.next().unwrap();
        let data_type = data.next().unwrap();
        
        for _ in 0..5 { let _ = data.next().unwrap(); } // skip color map info
        
        let x_origin = data.next().unwrap() | (data.next().unwrap() << 2);
        let y_origin = data.next().unwrap() | (data.next().unwrap() << 2);
        let width:i32 = (data.next().unwrap() as i32) | ((data.next().unwrap() as i32) << 8);
        let height:i32 = (data.next().unwrap() as i32) | ((data.next().unwrap() as i32) << 8);
        let bpp = data.next().unwrap();
        let img_desc = data.next().unwrap();

        if (color_map_type != 0) {
            panic!("Can't read files with color map");
        }

        for _ in 0..id_len { let _ = data.next().unwrap(); }
        // TODO: Read/skip color map data?

        let mut image = TgaImage::new(width, height);
        let mut pixel:i32 = 0;

        while pixel < width * height {
            let packet = data.next().unwrap();
            let count = (packet & 127) + 1;

            if packet & 128 > 0 {
                let b = data.next().unwrap();
                let g = data.next().unwrap();
                let r = data.next().unwrap();
                
                for _ in 0..count {
                    image.set_pixel(pixel % width, pixel / width, &RgbaColor::new_from_u8(r, g, b, 255));
                    pixel += 1;
                }
            } else {
                for _ in 0..count {
                    let b = data.next().unwrap();
                    let g = data.next().unwrap();
                    let r = data.next().unwrap();
                
                    image.set_pixel(pixel % width, pixel / width, &RgbaColor::new_from_u8(r, g, b, 255));
                    pixel += 1;
                }
            }
        }
        
        return image;
    }
}
