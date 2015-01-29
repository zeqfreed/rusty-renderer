use std::num::SignedInt;
use std::num::Float;
use std::rand::{thread_rng, Rng};

mod tga;
mod model;
mod math;

use tga::{TgaImage,RgbaColor};
use model::Model;
use math::{Vec2f,Vec3i,Vec3f};

#[derive(Copy)]
struct Vertex {
    p: Vec3f,
    t: Vec2f
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Vertex {
        Vertex { p: Vec3f::new(x, y, z), t: Vec2f::new(0.0, 0.0) }
    }
}

struct Renderer {
    image: TgaImage,
    diffuse: Option<TgaImage>,
    zbuffer: Vec<i32>
}

impl Renderer {
    pub fn new(width: i32, height: i32) -> Renderer {
        let image = TgaImage::new(width, height);

        let size = (image.width * image.height) as usize;
        let mut zbuffer = Vec::with_capacity(size);
        for _ in range(0, size) { zbuffer.push(-1000000); };

        return Renderer { image: image, zbuffer: zbuffer, diffuse: None };
    }

    pub fn set_diffuse(&mut self, diffuse: TgaImage) {
        self.diffuse = Some(diffuse);
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

    pub fn triangle(&mut self, v0: Vertex, v1: Vertex, v2: Vertex, color: &RgbaColor, intensity: f32) {
        if v0.p.x == v1.p.x && v1.p.x == v2.p.x {
            return;
        }

        let mut verts = vec![v0, v1, v2];
        {
            let mut slice = verts.as_mut_slice();
            if slice[2].p.x < slice[1].p.x { slice.swap(1, 2) }
            if slice[1].p.x < slice[0].p.x { slice.swap(1, 0) }
            if slice[2].p.x < slice[1].p.x { slice.swap(1, 2) }
        }

        // Vectors representing triangle sides, used for interpolation
        // a - long side
        // b and c - short sides
        let a = verts[2].p - verts[0].p;  // position vectors
        let b = verts[1].p - verts[0].p;
        let c = verts[2].p - verts[1].p;

        let ta = verts[2].t - verts[0].t; // texture coords vectors
        let tb = verts[1].t - verts[0].t;
        let tc = verts[2].t - verts[1].t;

        let a_xlen = verts[2].p.x - verts[0].p.x;
        let b_xlen = verts[1].p.x - verts[0].p.x;
        let c_xlen = verts[2].p.x - verts[1].p.x;

        let mut pa = verts[0].p;
        let mut pbc = verts[0].p;
        let mut t = verts[0].t;
        let mut tbc = verts[0].t;
        let mut xstep = 0f32;

        while xstep < a_xlen {
            let a_coef = xstep / a_xlen;
            pa = verts[0].p + a * a_coef;
            t = verts[0].t + ta * a_coef;

            if xstep <= b_xlen && b_xlen != 0.0 {
                let b_coef = xstep / b_xlen;
                pbc = verts[0].p + b * b_coef;
                tbc = verts[0].t + tb * b_coef;
            } else {
                let c_coef = (xstep - b_xlen) / c_xlen;
                pbc = verts[1].p + c * c_coef;
                tbc = verts[1].t + tc * c_coef;
            }

            // Vertical sweep of triangle pixels
            let ylen = (pa.y - pbc.y).abs().round() + 1.0; // Attempt to fix what appears to be a floating precision error
            let mut ystep = 0f32;

            while ystep <= ylen {
                let y_coef = ystep / ylen;
                let p = pbc + (pa - pbc) * y_coef;
                let idx = (p.x as i32 + self.image.height * p.y as i32) as usize;

                if (idx < self.zbuffer.len() && self.zbuffer[idx] < p.z as i32) {
                    self.zbuffer[idx] = p.z as i32;

                    let c = match self.diffuse.as_mut() {
                        Some(v) => {
                            let tp = tbc + (t - tbc) * y_coef;
                            v.get_pixel((tp.x * v.width as f32) as i32, (tp.y * v.height as f32) as i32)
                        },
                        None => { *color }
                    };

                    self.image.set_pixel(p.x as i32, p.y as i32, &(c * intensity));
                }

                ystep += 1.0;
            }

            xstep += 1.0;
        }
    }

    pub fn draw_model(&mut self, model: Model) {
        let half_width = (self.image.width as f32) / 2.0;
        let half_height = (self.image.height as f32) / 2.0;
        let half_depth = 255f32 / 2f32;

        let mut vertices: [Vertex; 3] = unsafe { std::mem::uninitialized() };
        let mut world_coords: [&Vec3f; 3] = unsafe { std::mem::uninitialized() };
        let light_dir = Vec3f::new(0f32, 0f32, -1f32).normalize();

        let white = RgbaColor::new(1.0, 1.0, 1.0, 1.0);

        for face in model.faces.iter() {
            for i in range(0, 3) {
                let v = model.vertices.get(face[i+2*i] as usize).unwrap();
                let t = model.texture_coords.get(face[i+2*i+1] as usize).unwrap();

                world_coords[i] = v;
                vertices[i] = Vertex {
                    p: Vec3f::new(
                        ((v.x + 1.0) * half_width).floor(),
                        ((v.y + 1.0) * half_height).floor(),
                        ((v.z + 1.0) * half_depth).floor()
                    ),
                    t: Vec2f::new(t.x, t.y)
                };
            }

            let normal: Vec3f = ((world_coords[2] - world_coords[0]) ^ (world_coords[1] - world_coords[0])).normalize();
            let mut intensity:f32 = light_dir * normal;

            if (intensity > 0.0) {
                self.triangle(vertices[0], vertices[1], vertices[2], &white, intensity);
            }
        }
    }
}

fn main() {
    let width:i32 = 800;
    let height:i32 = 800;

    let model = Model::load_from_file(Path::new("data/model.obj"));
    println!("read model; vertices: {}, texture coordinates: {}, faces: {}", model.vertices.len(), model.texture_coords.len(), model.faces.len());

    let diffuse = TgaImage::new_from_file(Path::new("data/diffuse.tga"));

    let mut renderer = Renderer::new(width, height);
    renderer.set_diffuse(diffuse);
    renderer.draw_model(model);
    //renderer.triangle(Vertex::new(0.0, 0.0, 0.0), Vertex::new(0.0, 50.0, 50.0), Vertex::new(50.0, 50.0, 0.0), &colors::RED, 1.0);
    //renderer.triangle(Vertex::new(0.0, 0.0, 0.0), Vertex::new(50.0, 50.0, 50.0), Vertex::new(50.0, 0.0, 0.0), &colors::BLUE, 1.0);
    renderer.image.write_to_file(Path::new("output.tga"));
}
