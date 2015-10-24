
use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;
use math::{Vec2f,Vec3f};

pub struct Model {
    pub vertices: Vec<Vec3f>,
    pub normals: Vec<Vec3f>,
    pub texture_coords: Vec<Vec2f>,
    pub faces: Vec<[i32;9]>
}

fn extract<T: FromStr + Copy + Debug>(str: &str, arr: &mut [T], default: T){
    for (i, word) in str.split_whitespace().enumerate() {
        arr[i] = FromStr::from_str(word).unwrap_or(default);
        if i >= arr.len() { break; }
    }
}

fn extract_faces(str: &str, arr: &mut [i32]) {
    for (fi, face) in str.split_whitespace().enumerate() {
        for (i, index) in face.split('/').enumerate() {
            let ii = fi * 3 + i;
            if ii > arr.len() { break; }

            let v:i32 = FromStr::from_str(index).unwrap();
            arr[ii] = v - 1;
        }
    }
}

impl Model {
    pub fn new_from_file(filename: &Path) -> Model {
        let file = match File::open(filename) {
            Err(e) => panic!("couldn't read {}: {:?}", filename.display(), e),
            Ok(file) => file
        };

        let mut vertices:Vec<Vec3f> = Vec::with_capacity(2000);
        let mut normals:Vec<Vec3f> = Vec::with_capacity(2000);
        let mut faces:Vec<[i32;9]> = Vec::with_capacity(500);
        let mut texture_coords:Vec<Vec2f> = Vec::with_capacity(500);

        let mut reader = BufReader::new(file);
        'line: for line in reader.lines().filter_map(|res| res.ok()) {
            let mut coords = [0f32; 3];
            let mut indices = [0i32; 9];

            if line.starts_with("v ") {
                extract::<f32>(&line[2..], &mut coords, 0.0);
                vertices.push(Vec3f::new(coords[0], coords[1], coords[2]));

            } else if line.starts_with("vt ") {
                extract::<f32>(&line[3..], &mut coords, 0.0);
                texture_coords.push(Vec2f::new(coords[0], coords[1]));

            } else if line.starts_with("vn ") {
                extract::<f32>(&line[2..], &mut coords, 0.0);
                normals.push(Vec3f::new(coords[0], coords[1], coords[2]));

            } else if line.starts_with("f ") {
                extract_faces(&line[2..], &mut indices);
                faces.push(indices);
            }
        }

        return Model {
            vertices: vertices,
            normals: normals,
            faces: faces,
            texture_coords: texture_coords
        };
    }
}
