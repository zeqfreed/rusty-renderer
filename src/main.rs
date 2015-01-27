use std::num::SignedInt;
use std::num::Float;
use std::rand::{thread_rng, Rng};

mod tga;
mod model;
mod math;

use tga::{TgaImage,RgbaColor};
use model::Model;
use math::{Vec3i,Vec3f};

mod colors {
    use tga::RgbaColor;

    pub static RED:RgbaColor = RgbaColor(0xFF000000);
    pub static GREEN:RgbaColor = RgbaColor(0x00FF0000);
    pub static BLUE:RgbaColor = RgbaColor(0x0000FF00);
    pub static WHITE:RgbaColor = RgbaColor(0xFFFFFF00);
    pub static YELLOW:RgbaColor = RgbaColor(0xFFFF0000);
}

struct Renderer {
    image: TgaImage,
    zbuffer: Vec<i32>
}

impl Renderer {
    pub fn new(image: TgaImage) -> Renderer {
        let size = (image.width * image.height) as usize;
        let mut zbuffer = Vec::with_capacity(size);
        for _ in range(0, size) { zbuffer.push(-1000000); };
        return Renderer { image: image, zbuffer: zbuffer };
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: &RgbaColor) {
        let dx:i32 = x1 - x0;
        let dy:i32 = y1 - y0;

        let steps = std::cmp::max(dx.abs(), dy.abs());
        let xs = (dx.abs() as f32 / steps as f32).abs();
        let ys = (dy.abs() as f32 / steps as f32).abs();
        let xis:i32 = dx.signum();
        let yis:i32 = dy.signum();
        let mut xa = 0f32;
        let mut ya = 0f32;
        let mut x = x0;
        let mut y = y0;

        for _ in range(0, steps) {
            self.image.set_pixel(x, y, color);

            xa += xs;
            if xa > 0.5 { x += xis; xa -= 1.0 }
            ya += ys;
            if ya > 0.5 { y += yis; ya -= 1.0 }
        }

        self.image.set_pixel(x, y, color);
    }

    pub fn triangle(&mut self, p0: Vec3f, p1: Vec3f, p2: Vec3f, color: &RgbaColor) {
        if p0.x == p1.x && p1.x == p2.x {
            return;
        }

        let mut points = vec![p0, p1, p2];
        {
            let mut slice = points.as_mut_slice();
            if slice[2].x < slice[1].x { slice.swap(1, 2) }
            if slice[1].x < slice[0].x { slice.swap(1, 0) }
            if slice[2].x < slice[1].x { slice.swap(1, 2) }
        }

        let a = points[2] - points[0]; // a - long side vector, b and c - short sides
        let b = points[1] - points[0];
        let c = points[2] - points[1];

        let mut pa = points[0];
        let mut pbc = points[0];

        let a_xlen = (points[2].x - points[0].x);
        let b_xlen = (points[1].x - points[0].x);
        let c_xlen = (points[2].x - points[1].x);
        let mut x = points[0].x.ceil() as i32;
        let mut step = 0f32;

        while x < points[2].x as i32 {
            pa = points[0] + a * (step / a_xlen);

            if step <= b_xlen && b_xlen != 0.0 {
                pbc = points[0] + b * (step / b_xlen);
            } else {
                pbc = points[1] + c * ((step - b_xlen) / c_xlen);
            }

            // Vertical sweep of triangle pixels
            let mut y = pbc.y as i32;
            let dz = pbc.z - pa.z;
            let ylen = (pa.y - pbc.y).abs().ceil() + 1.0;
            let ystep = (pa.y - pbc.y).signum() as i32;
            for i in range(0, ylen as i32) {
                let z = (pa.z + dz * (i as f32 / ylen)) as i32;
                let idx = (x + self.image.height * y) as usize;

                if (idx < self.zbuffer.len() && self.zbuffer[idx] < z) {
                    self.zbuffer[idx] = z;
                    self.image.set_pixel(x, y, color);
                }

                y += ystep;
            }

            x += 1;
            step += 1.0;
        }
    }

    pub fn draw_model(&mut self, model: Model) {
        let half_width = (self.image.width as f32) / 2.0;
        let half_height = (self.image.height as f32) / 2.0;
        let half_depth = 255f32 / 2f32;

        let mut screen_coords: [Vec3f; 3] = unsafe { std::mem::uninitialized() };
        let mut world_coords: [&Vec3f; 3] = unsafe { std::mem::uninitialized() };
        let light_dir = Vec3f::new(0f32, 0f32, -1f32).normalize();

        for face in model.faces.iter() {
            for i in range(0, 3) {
                let v = model.vertices.get(face[i] as usize).unwrap();
                world_coords[i] = v;
                screen_coords[i] = Vec3f::new(((v.x + 1.0) * half_width).floor(), ((v.y + 1.0) * half_height).floor(), ((v.z + 1.0) * half_depth).floor());
            }

            let normal: Vec3f = ((world_coords[2] - world_coords[0]) ^ (world_coords[1] - world_coords[0])).normalize();
            let intensity:f32 = light_dir * normal;

            if (intensity > 0f32) {
                let grey = (255f32 * intensity) as u32;
                let color = RgbaColor(grey << 8 | grey << 16 | grey << 24);
                self.triangle(screen_coords[0], screen_coords[1], screen_coords[2], &color);
            }
        }
    }
}

fn main() {
    let width:i32 = 1000;
    let height:i32 = 1000;

    let model = Model::load_from_file(Path::new("data/model.obj"));

    let mut renderer = Renderer::new(TgaImage::new(width, height));
    renderer.draw_model(model);
    //renderer.triangle(Vec3f::new(0.0, 0.0, 0.0), Vec3f::new(0.0, 50.0, 50.0), Vec3f::new(50.0, 50.0, 0.0), &colors::RED);
    //renderer.triangle(Vec3f::new(0.0, 0.0, 0.0), Vec3f::new(50.0, 50.0, 50.0), Vec3f::new(50.0, 0.0, 0.0), &colors::BLUE);
    renderer.image.write_to_file(Path::new("output.tga"));
}
