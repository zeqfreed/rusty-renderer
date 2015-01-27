use std::num::SignedInt;
use std::num::Float;
use std::rand::{thread_rng, Rng};

mod tga;
mod model;
mod math;

use tga::{TgaImage,RgbaColor};
use model::Model;
use math::{Point,Vec3f};

mod colors {
    use tga::RgbaColor;

    pub static RED:RgbaColor = RgbaColor(0xFF000000);
    pub static GREEN:RgbaColor = RgbaColor(0x00FF0000);
    pub static BLUE:RgbaColor = RgbaColor(0x0000FF00);
    pub static WHITE:RgbaColor = RgbaColor(0xFFFFFF00);
    pub static YELLOW:RgbaColor = RgbaColor(0xFFFF0000);
}

struct Renderer {
    image: TgaImage
}

impl Renderer {
    pub fn new(image: TgaImage) -> Renderer {
        return Renderer { image: image };
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

    pub fn triangle(&mut self, p0: Point, p1: Point, p2: Point, color: &RgbaColor) {
        let mut points = vec![p0, p1, p2];

        {
            let mut slice = points.as_mut_slice();
            if slice[2].x < slice[1].x { slice.swap(1, 2) }
            if slice[1].x < slice[0].x { slice.swap(1, 0) }
            if slice[2].x < slice[1].x { slice.swap(1, 2) }
        }

        let ys = (points[2].y - points[0].y) as f32 / (points[2].x - points[0].x) as f32;
        let ys0 = (points[1].y - points[0].y) as f32 / (points[1].x - points[0].x) as f32;
        let ys1 = (points[2].y - points[1].y) as f32 / (points[2].x - points[1].x) as f32;

        let mut x = points[0].x;
        let mut y = points[0].y;
        let mut yc = y;

        let s2 = (points[1].x - points[0].x) as f32;
        let mut step = 0f32;

        while x < points[2].x {
            y = (points[0].y as f32 + step * ys) as i32;
            if x < points[1].x {
                yc = (points[0].y as f32 + step * ys0) as i32;
            } else {
                yc = (points[1].y as f32 + (step - s2) * ys1) as i32;
            }

            for ly in range(std::cmp::min(y, yc), std::cmp::max(y, yc) + 1) {
                self.image.set_pixel(x, ly, color);
            }

            x += 1;
            step += 1.0;
        }
    }

    pub fn draw_model(&mut self, model: Model) {
        let half_width = self.image.width as f32 / 2.0;
        let half_height = self.image.height as f32 / 2.0;

        let mut points: [Point; 3] = unsafe { std::mem::uninitialized() };
        let mut vertices: [&Vec3f; 3] = unsafe { std::mem::uninitialized() };
        let light_dir = Vec3f::new(0f32, 0f32, -1f32).normalize();

        for face in model.faces.iter() {
            for i in range(0, 3) {
                let v = model.vertices.get(face[i] as usize).unwrap();
                vertices[i] = v;
                points[i] = Point {x: ((v.x + 1.0) * half_width) as i32, y: ((v.y + 1.0) * half_height) as i32};
            }

            let normal: Vec3f = ((vertices[2] - vertices[0]) ^ (vertices[1] - vertices[0])).normalize();
            let intensity:f32 = light_dir * normal;

            if (intensity > 0f32) {
                let grey = (255f32 * intensity) as u32;
                self.triangle(points[0], points[1], points[2], &RgbaColor(grey << 8 | grey << 16 | grey << 24));
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
    renderer.image.write_to_file(Path::new("output.tga"));
}
