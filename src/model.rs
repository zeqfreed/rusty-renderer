extern crate core;

use std::io::{File,BufferedReader};
use std::io::BufferPrelude;
use std::str::FromStr;
use math::Vec3f;

pub struct Model {
    pub vertices: Vec<Vec3f>,
    pub faces: Vec<[i32;3]>
}

impl Model {
    pub fn load_from_file(filename: Path) -> Model {
        let file = match File::open(&filename) {
            Err(why) => panic!("couldn't read {}: {}", filename.display(), why.desc),
            Ok(file) => file
        };

        let mut vertices:Vec<Vec3f> = Vec::with_capacity(2000);
        let mut faces:Vec<[i32;3]> = Vec::with_capacity(500);

        let mut reader = BufferedReader::new(file);
        'line: for line in reader.lines().filter_map(|res| res.ok()) {
            let mut coords = [0f32; 3];
            let mut indices = [0i32; 3];

            if line.starts_with("v ") {
                let mut iter = line.slice_from(2).words();
                for i in range(0, 3) {
                    coords[i] = match iter.next() {
                        Some(v) => match FromStr::from_str(v) { Some(v) => v, None => continue 'line },
                        None => continue 'line
                    };
                }
                
                //println!("x: {}, y: {}, z: {}", coords[0], coords[1], coords[2]);
                vertices.push(Vec3f { x: coords[0], y: coords[1], z: coords[2] });
            } else if line.starts_with("f ") {
                let mut i = 0;
                for word in line.slice_from(2).words().take(3) {
                    indices[i] = match word.split('/').next() {
                        Some(v) => match FromStr::from_str(v) { Some(v) => v, None => continue 'line },
                        None => continue 'line
                    };
                    i += 1;
                }

                if (i < 2) { continue 'line }
            
                //println!("face {} / {} / {}", indices[0], indices[1], indices[2]);
                faces.push([indices[0] - 1, indices[1] - 1, indices[2] - 1]);
            }
        }

        return Model { vertices: vertices, faces: faces };
    }
}
