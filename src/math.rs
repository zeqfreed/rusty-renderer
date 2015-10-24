use std::ops::{Add,Sub,Mul,Div,BitXor};

#[derive(Debug,Clone,Copy)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T
}

pub type Vec2f = Vec2<f32>;

#[derive(Debug,Clone,Copy)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

#[derive(Clone,Copy)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32
}

impl Vec3i {
    pub fn new(x: i32, y: i32, z: i32) -> Vec3i {
        Vec3i {x: x, y: y, z: z}
    }
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Vec2<T> {
        Vec2::<T> {x: x, y: y}
    }
}

impl<T> Mul<T> for Vec2<T>
    where T: Mul<T, Output=T> + Copy {
    type Output = Vec2<T>;

    fn mul(self, rhs: T) -> Vec2<T> {
        Vec2::<T>::new(self.x * rhs, self.y * rhs)
    }
}

impl<T> Add<Vec2<T>> for Vec2<T>
    where T: Add<T, Output=T> {
    type Output = Vec2<T>;

    fn add(self, rhs: Vec2<T>) -> Vec2<T> {
        Vec2::<T>::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<T> Sub<Vec2<T>> for Vec2<T>
    where T: Sub<T, Output=T> {
    type Output = Vec2<T>;

    fn sub(self, rhs: Vec2<T>) -> Vec2<T> {
        Vec2::<T>::new(self.x - rhs.x, self.y - rhs.y)
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

impl Add<Vec3f> for Vec3f {
    type Output = Vec3f;

    #[inline(always)]
    fn add(self, other: Vec3f) -> Vec3f {
        Vec3f::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub<Vec3f> for Vec3f {
    type Output = Vec3f;

    #[inline(always)]
    fn sub(self, other: Vec3f) -> Vec3f {
        Vec3f::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl<'a,'b> Sub<&'a Vec3f> for &'b Vec3f {
    type Output = Vec3f;

    #[inline(always)]
    fn sub(self, other: &Vec3f) -> Vec3f {
        Vec3f::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl BitXor<Vec3f> for Vec3f {
    type Output = Vec3f;

    #[inline(always)]
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

    #[inline(always)]
    fn mul(self, f:f32) -> Vec3f {
        Vec3f::new(self.x * f, self.y * f, self.z * f)
    }
}

impl Div<f32> for Vec3f {
    type Output = Vec3f;

    #[inline(always)]
    fn div(self, f:f32) -> Vec3f {
        Vec3f::new(self.x / f, self.y / f, self.z / f)
    }
}

impl Mul<Vec3f> for Vec3f {
    type Output = f32;

    #[inline(always)]
    fn mul(self, other: Vec3f) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

