
mod tga;
mod model;
mod math;

use std::f32;
use std::path::Path;

use tga::{TgaImage,RgbaColor};
use model::Model;
use math::{Vec2f,Vec3f};

#[derive(Clone,Copy)]
struct Vertex {
    p: Vec3f,
    t: Vec2f,
    i: f32 // intensity
}

enum Shading {
    Flat,
    Gouraud
}

struct Renderer {
    image: TgaImage,
    diffuse: Option<TgaImage>,
    zbuffer: Vec<f32>,
    color: RgbaColor,
    shading: Shading
}

impl Renderer {
    pub fn new(width: i32, height: i32) -> Renderer {
        let image = TgaImage::new(width, height);

        let size = (image.width * image.height) as usize;
        let mut zbuffer = Vec::with_capacity(size);
        for _ in 0..size { zbuffer.push(f32::NEG_INFINITY); };

        return Renderer {
            image: image,
            zbuffer: zbuffer,
            diffuse: None,
            color: RgbaColor::new(1.0, 1.0, 1.0, 1.0),
            shading: Shading::Flat
        };
    }

    pub fn set_diffuse(&mut self, diffuse: TgaImage) {
        self.diffuse = Some(diffuse);
    }

    pub fn set_shading(&mut self, shading: Shading) {
        self.shading = shading;
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
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

        for _ in 0..steps {
            self.image.set_pixel(x, y, &self.color);

            xa += xs;
            if xa > 0.5 { x += xis; xa -= 1.0 }
            ya += ys;
            if ya > 0.5 { y += yis; ya -= 1.0 }
        }

        self.image.set_pixel(x, y, &self.color);
    }

    pub fn triangle(&mut self, v0: Vertex, v1: Vertex, v2: Vertex) {
        let mut verts = vec![v0, v1, v2];
        {
            let mut slice = &mut verts[..];
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

        let ia = verts[2].i - verts[0].i;
        let ib = verts[1].i - verts[0].i;
        let ic = verts[2].i - verts[1].i;

        let a_xlen = verts[2].p.x - verts[0].p.x;
        let b_xlen = verts[1].p.x - verts[0].p.x;
        let c_xlen = verts[2].p.x - verts[1].p.x;

        let mut pa = verts[0].p;
        let mut pbc = verts[0].p;
        let mut t = verts[0].t;
        let mut tbc = verts[0].t;
        let mut xstep = 0f32;

        let mut i = verts[0].i;
        let mut ibc = verts[0].i;

        while xstep < a_xlen {
            let a_coef = xstep / a_xlen;
            pa = verts[0].p + a * a_coef;
            t = verts[0].t + ta * a_coef;
            i = verts[0].i + ia * a_coef;

            if xstep <= b_xlen && b_xlen != 0.0 {
                let b_coef = xstep / b_xlen;
                pbc = verts[0].p + b * b_coef;
                tbc = verts[0].t + tb * b_coef;
                ibc = verts[0].i + ib * b_coef;
            } else {
                let c_coef = (xstep - b_xlen) / c_xlen;
                pbc = verts[1].p + c * c_coef;
                tbc = verts[1].t + tc * c_coef;
                ibc = verts[1].i + ic * c_coef;
            }

            // Vertical sweep of triangle pixels
            let ylen = (pa.y - pbc.y).abs().round() + 1.0; // Attempt to fix what appears to be a floating precision error
            let mut ystep = 0f32;

            while ystep <= ylen {
                let y_coef = ystep / ylen;
                let p = pbc + (pa - pbc) * y_coef;
                let idx = (p.x as i32 + self.image.height * p.y as i32) as usize;

                if idx > self.zbuffer.len() || p.z < self.zbuffer[idx] {
                    ystep += 1.0;
                    continue;
                }

                self.zbuffer[idx] = p.z;

                let c = match self.diffuse.as_mut() {
                    Some(v) => {
                        let tp = tbc + (t - tbc) * y_coef;
                        v.get_pixel((tp.x * v.width as f32) as i32, (tp.y * v.height as f32) as i32)
                    },
                    None => { self.color }
                };

                let intensity = (ibc + (i - ibc) * y_coef).abs();
                self.image.set_pixel(p.x as i32, p.y as i32, &(c * intensity));

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

        for face in model.faces.iter() {
            for i in 0..3 {
                let v = model.vertices.get(face[i+2*i] as usize).unwrap();
                let t = model.texture_coords.get(face[i+2*i+1] as usize).unwrap();

                let intensity = match self.shading {
                    Shading::Gouraud => {
                        let n = *model.normals.get(face[i+2*i+2] as usize).unwrap();
                        light_dir * n
                    },
                    _ => 0.0
                };

                world_coords[i] = v;
                vertices[i] = Vertex {
                    p: Vec3f::new(
                        ((v.x + 1.0) * half_width).floor(),
                        ((v.y + 1.0) * half_height).floor(),
                        ((v.z + 1.0) * half_depth).floor()
                    ),
                    t: Vec2f::new(t.x, t.y),
                    i: intensity
                };
            }

            if let Shading::Flat = self.shading {
                let normal: Vec3f = ((world_coords[2] - world_coords[0]) ^ (world_coords[1] - world_coords[0])).normalize();
                let intensity = light_dir * normal;
                for i in 0..3 { vertices[i].i = intensity; }
            }

            self.triangle(vertices[0], vertices[1], vertices[2]);
        }
    }
}

fn main() {
    let width:i32 = 800;
    let height:i32 = 800;

    let model = Model::new_from_file(&Path::new("data/model.obj"));
    let diffuse = TgaImage::new_from_file(&Path::new("data/diffuse.tga"));

    let mut renderer = Renderer::new(width, height);
    renderer.set_diffuse(diffuse);
    renderer.set_shading(Shading::Gouraud);
    renderer.draw_model(model);

    renderer.image.write_to_file(Path::new("output.tga"));
}
