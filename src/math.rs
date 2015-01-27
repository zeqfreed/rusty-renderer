use std::num::Float;
use std::ops::{Sub,Mul,BitXor};

#[derive(Copy)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

#[derive(Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point {x: x, y: y}
    }
}

impl Vec3f {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3f {
        Vec3f { x: x, y: y, z: z }
    }

    pub fn normalize(&mut self) -> Vec3f {
        let v = Vec3f::new(self.x, self.y, self.z) * (1f32 / (self.x * self.x + self.y * self.y + self.z * self.z).sqrt());
        self.x = v.x;
        self.y = v.y;
        self.z = v.z;
        return *self;
    }
}

impl Sub<Vec3f> for Vec3f {
    type Output = Vec3f;

    fn sub(self, other: Vec3f) -> Vec3f {
        Vec3f::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl<'a,'b> Sub<&'a Vec3f> for &'b Vec3f {
    type Output = Vec3f;

    fn sub(self, other: &Vec3f) -> Vec3f {
        Vec3f::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl BitXor<Vec3f> for Vec3f {
    type Output = Vec3f;

    fn bitxor(self, other: Vec3f) -> Vec3f {
        Vec3f::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x
        )
    }
}

impl Mul<f32> for Vec3f {
    type Output = Vec3f;

    fn mul(self, f:f32) -> Vec3f {
        Vec3f::new(self.x * f, self.y * f, self.z * f)
    }
}

impl Mul<Vec3f> for Vec3f {
    type Output = f32;

    fn mul(self, other: Vec3f) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}