extern crate core;

use std::io::{File,BufferedReader};
use std::io::BufferPrelude;
use std::str::FromStr;
use math::{Vec2f,Vec3f};

pub struct Model {
    pub vertices: Vec<Vec3f>,
    pub texture_coords: Vec<Vec2f>,
    pub faces: Vec<[i32;9]>
}

impl Model {
    pub fn load_from_file(filename: Path) -> Model {
        let file = match File::open(&filename) {
            Err(why) => panic!("couldn't read {}: {}", filename.display(), why.desc),
            Ok(file) => file
        };

        let mut vertices:Vec<Vec3f> = Vec::with_capacity(2000);
        let mut faces:Vec<[i32;9]> = Vec::with_capacity(500);
        let mut texture_coords:Vec<Vec2f> = Vec::with_capacity(500);

        let mut reader = BufferedReader::new(file);
        'line: for line in reader.lines().filter_map(|res| res.ok()) {
            let mut coords = [0f32; 3];
            let mut indices = [0i32; 9];

            if line.starts_with("v ") {
                let mut iter = line.slice_from(2).words();
                for i in range(0, 3) {
                    coords[i] = match iter.next() {
                        Some(v) => match FromStr::from_str(v) { Some(v) => v, None => continue 'line },
                        None => continue 'line
                    };
                }
                
                //println!("v x: {}, y: {}, z: {}", coords[0], coords[1], coords[2]);
                vertices.push(Vec3f::new(coords[0], coords[1], coords[2]));

            } else if line.starts_with("vt ") {
                let mut iter = line.slice_from(3).words();
                for i in range(0, 2) {
                    coords[i] = match iter.next() {
                        Some(v) => match FromStr::from_str(v) { Some(v) => v, None => continue 'line },
                        None => continue 'line
                    };
                }

                //println!("vt x: {}, y: {}", coords[0], coords[1]);
                texture_coords.push(Vec2f::new(coords[0], coords[1]));

            } else if line.starts_with("f ") {
                let mut i = 0;
                for word in line.slice_from(2).words() {
                    for index in word.split('/') {
                        let idx:i32 = match FromStr::from_str(index) { Some(v) => { v }, None => continue 'line };
                        indices[i] = idx - 1;
                        i += 1;
                    };
                }
            
                // println!("face {} / {} / {}", indices[0], indices[1], indices[2]);
                faces.push(indices.clone());
            }
        }

        return Model { vertices: vertices, faces: faces, texture_coords: texture_coords };
    }
}
