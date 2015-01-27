use std::default::Default;
use std::io::{File,BufferedWriter};

pub struct RgbaColor(pub u32);

impl RgbaColor {
    #[inline]
    pub fn get_r(&self) -> u8 { match *self { RgbaColor(v) => ((v & 0xFF000000) >> 24) as u8 } }

    #[inline]
    pub fn get_g(&self) -> u8 { match *self { RgbaColor(v) => ((v & 0x00FF0000) >> 16) as u8 } }

    #[inline]
    pub fn get_b(&self) -> u8 { match *self { RgbaColor(v) => ((v & 0x0000FF00) >>  8) as u8 } }

    #[inline]
    pub fn get_a(&self) -> u8 { match *self { RgbaColor(v) => ((v & 0x000000FF)) as u8 } }
}

impl Clone for RgbaColor {
    fn clone(&self) -> RgbaColor {
        match *self { RgbaColor(v) => RgbaColor(v) }
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
        self.r = color.get_r();
        self.g = color.get_g();
        self.b = color.get_b();
        self.a = color.get_a();
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
}
