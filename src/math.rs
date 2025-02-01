use rand::Rng;
use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Sub};

pub type Scalar = f64;
pub const PI: Scalar = std::f64::consts::PI;

#[inline(always)]
pub fn radians(degrees: Scalar) -> Scalar {
    degrees * PI / 180.0
}

#[inline(always)]
pub fn random(min: Scalar, max: Scalar, rand: &mut rand::prelude::ThreadRng) -> Scalar {
    rand.gen_range(min..max)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
}

impl Vec3 {
    #[inline(always)]
    pub fn new(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Vec3 { x, y, z }
    }

    #[inline(always)]
    pub fn uniform(v: Scalar) -> Self {
        Vec3 { x: v, y: v, z: v }
    }

    #[inline(always)]
    pub fn zero() -> Self {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    #[inline(always)]
    pub fn one() -> Self {
        Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }

    #[inline(always)]
    pub fn dot(&self, other: Self) -> Scalar {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline(always)]
    pub fn cross(&self, other: Self) -> Self {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    #[inline(always)]
    pub fn length_squared(&self) -> Scalar {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[inline(always)]
    pub fn length(&self) -> Scalar {
        self.length_squared().sqrt()
    }

    #[inline(always)]
    pub fn normalize(&self) -> Self {
        let length = self.length();
        Vec3 {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }

    #[inline(always)]
    pub fn abs(&self) -> Self {
        Vec3 {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }

    #[inline(always)]
    pub fn reflect(&self, normal: Vec3) -> Self {
        *self - (normal * 2.0 * self.dot(normal))
    }

    #[inline(always)]
    pub fn refract(uv: Vec3, n: Vec3, etai_over_etat: Scalar) -> Vec3 {
        let cos_theta = (-uv).dot(n).min(1.0);
        let r_out_perp = etai_over_etat * (uv + cos_theta * n);
        let r_out_parallel = -((1.0 - r_out_perp.length_squared()).abs().sqrt()) * n;
        r_out_perp + r_out_parallel
    }

    #[inline(always)]
    pub fn near_zero_tolerance(&self, tolerance: Scalar) -> bool {
        (self.x.abs() <= tolerance) && (self.y.abs() <= tolerance) && (self.z.abs() <= tolerance)
    }

    #[inline(always)]
    pub fn near_zero(&self) -> bool {
        self.near_zero_tolerance(0.00000001)
    }

    #[inline(always)]
    pub fn largest_component(&self) -> usize {
        if self.x > self.y && self.x > self.z {
            0 // X-axis
        } else if self.y > self.z {
            1 // Y-axis
        } else {
            2 // Z-axis
        }
    }

    #[inline(always)]
    pub fn rotate_x(&mut self, theta: Scalar) {
        let new_y = self.y * theta.cos() + self.z * theta.sin();
        let new_z = -self.y * theta.sin() + self.z * theta.cos();
        self.y = new_y;
        self.z = new_z;
    }

    #[inline(always)]
    pub fn rotate_y(&mut self, theta: Scalar) {
        let new_x = self.x * theta.cos() + self.z * theta.sin();
        let new_z = -self.x * theta.sin() + self.z * theta.cos();
        self.x = new_x;
        self.z = new_z;
    }

    #[inline(always)]
    pub fn linear_to_gamma(&self) -> Self {
        Vec3 {
            x: self.x.sqrt(),
            y: self.y.sqrt(),
            z: self.z.sqrt(),
        }
    }

    #[inline(always)]
    pub fn random(min: Scalar, max: Scalar, rand: &mut rand::prelude::ThreadRng) -> Self {
        Vec3 {
            x: min + (max - min) * rand.gen::<Scalar>(),
            y: min + (max - min) * rand.gen::<Scalar>(),
            z: min + (max - min) * rand.gen::<Scalar>(),
        }
    }

    #[inline(always)]
    pub fn random_unit_vector(rand: &mut rand::prelude::ThreadRng) -> Self {
        let p = Self::random(-1.0, 1.0, rand);
        p.normalize()
    }

    #[inline(always)]
    pub fn random_in_unit_disk(rand: &mut rand::prelude::ThreadRng) -> Self {
        let p = Vec3::new(random(-1.0, 1.0, rand), random(-1.0, 1.0, rand), 0.0);
        p.normalize()
    }
}

impl Add for Vec3 {
    type Output = Self;

    #[inline(always)]
    fn add(self, other: Self) -> Self {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, other: Self) -> Self {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<Scalar> for Vec3 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, scalar: Scalar) -> Self {
        Vec3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Mul<Vec3> for Scalar {
    type Output = Vec3;

    #[inline(always)]
    fn mul(self, v: Vec3) -> Vec3 {
        Vec3 {
            x: v.x * self,
            y: v.y * self,
            z: v.z * self,
        }
    }
}

impl Mul<Vec3> for Vec3 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, other: Vec3) -> Self {
        Vec3 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl Div<Vec3> for Vec3 {
    type Output = Self;

    #[inline(always)]
    fn div(self, other: Vec3) -> Self {
        Vec3 {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }
}

impl Div<Scalar> for Vec3 {
    type Output = Self;

    #[inline(always)]
    fn div(self, scalar: Scalar) -> Self {
        Vec3 {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

impl Div<Vec3> for Scalar {
    type Output = Vec3;

    #[inline(always)]
    fn div(self, v: Vec3) -> Vec3 {
        Vec3 {
            x: self / v.x,
            y: self / v.y,
            z: self / v.z,
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Index<usize> for Vec3 {
    type Output = Scalar;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("can't access Vec3 axis over 2!"),
        }
    }
}

impl IndexMut<usize> for Vec3 {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("can't access Vec3 axis over 2!"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    #[inline(always)]
    pub fn new(origin: Vec3, dir: Vec3) -> Self {
        Self { origin, dir }
    }

    #[inline(always)]
    pub fn at(&self, t: Scalar) -> Vec3 {
        self.origin + self.dir * t
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    #[inline(always)]
    pub fn new(min: Vec3, max: Vec3) -> Self {
        AABB { min, max }
    }

    #[inline(always)]
    pub fn center(&self) -> Vec3 {
        self.min + (self.max - self.min) / 2.0
    }

    #[inline(always)]
    pub fn hit(&self, ray: &Ray) -> bool {
        for a in 0..3 {
            let inv_d = 1.0 / ray.dir[a];
            let mut t0 = (self.min[a] - ray.origin[a]) * inv_d;
            let mut t1 = (self.max[a] - ray.origin[a]) * inv_d;
            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }
            let min = if t0 > 0.001 { t0 } else { 0.001 };
            let max = if t1 < Scalar::MAX { t1 } else { Scalar::MAX };
            if max <= min {
                return false;
            }
        }
        true
    }

    // returns axis with the largest size (x = 0, y = 1, z = 2)
    #[inline(always)]
    pub fn largest_axis(&self) -> usize {
        let extent = self.max - self.min;
        let max_extent = extent.x.max(extent.y).max(extent.z);

        if max_extent == extent.x {
            0 // x-axis
        } else if max_extent == extent.y {
            1 // y-axis
        } else {
            2 // z-axis
        }
    }

    #[inline(always)]
    pub fn surrounding(a: AABB, b: AABB) -> Self {
        Self::new(
            Vec3::new(
                Scalar::min(a.min.x, b.min.x),
                Scalar::min(a.min.y, b.min.y),
                Scalar::min(a.min.z, b.min.z),
            ),
            Vec3::new(
                Scalar::max(a.max.x, b.max.x),
                Scalar::max(a.max.y, b.max.y),
                Scalar::max(a.max.z, b.max.z),
            ),
        )
    }
}

impl Default for AABB {
    fn default() -> Self {
        Self {
            min: Vec3::new(0.0, 0.0, 0.0),
            max: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}
