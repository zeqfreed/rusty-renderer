use std::default::Default;
use std::io::{File,BufferedWriter,BufferedReader};

pub struct RgbaColor(pub u32);

impl RgbaColor {
    #[inline(always)]
    pub fn get_r(&self) -> u8 { match *self { RgbaColor(v) => ((v & 0xFF000000) >> 24) as u8 } }

    #[inline(always)]
    pub fn get_g(&self) -> u8 { match *self { RgbaColor(v) => ((v & 0x00FF0000) >> 16) as u8 } }

    #[inline(always)]
    pub fn get_b(&self) -> u8 { match *self { RgbaColor(v) => ((v & 0x0000FF00) >>  8) as u8 } }

    #[inline(always)]
    pub fn get_a(&self) -> u8 { match *self { RgbaColor(v) => ((v & 0x000000FF)) as u8 } }

    #[inline(always)]
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> RgbaColor {
        RgbaColor((r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8)
    }
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

    pub fn get_color(&self) -> RgbaColor {
        RgbaColor((self.r as u32) << 24 | (self.g as u32) << 16 | (self.b as u32) << 8)
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

    pub fn new_from_file(filename: Path) -> TgaImage {
        let file = match File::open(&filename) {
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

        println!("{}x{}, {} bits per pixel; data type = {}", width, height, bpp, data_type);

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
                    image.set_pixel(pixel % width, pixel / width, &RgbaColor((bgr[2] as u32) << 24 | (bgr[1] as u32) << 16 | (bgr[0] as u32) << 8));
                    pixel += 1;
                }
            } else {
                for _ in range(0, count) {
                    bgr = reader.read_exact(3).unwrap();
                    image.set_pixel(pixel % width, pixel / width, &RgbaColor((bgr[2] as u32) << 24 | (bgr[1] as u32) << 16 | (bgr[0] as u32) << 8));
                    pixel += 1;
                }
            }
        }
        
        return image;
    }
}
