use std::ops::{Add, Sub, Mul, Div, Neg, Index, IndexMut};

use rand::Rng;

pub fn radians(degrees: f64) -> f64 {
	degrees * std::f64::consts::PI / 180.0
}

pub fn random(min: f64, max: f64, rand: &mut rand::prelude::ThreadRng) -> f64 {
	min + (max - min) * rand.gen::<f64>()
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
	pub x: f64,
	pub y: f64,
	pub z: f64
}

impl Vec3 {
	pub fn new(x: f64, y: f64, z: f64) -> Self {
		Vec3 { x, y, z }
	}

	pub fn uniform(v: f64) -> Self {
		Vec3 { x: v, y: v, z: v }
	}

	pub fn zero() -> Self {
		Vec3 { x: 0.0, y: 0.0, z: 0.0 }
	}

	pub fn one() -> Self {
		Vec3 { x: 1.0, y: 1.0, z: 1.0 }
	}

	pub fn dot(&self, other: Self) -> f64 {
		self.x * other.x + self.y * other.y + self.z * other.z
	}

	pub fn cross(&self, other: Self) -> Self {
		Vec3 {
			x: self.y * other.z - self.z * other.y,
			y: self.z * other.x - self.x * other.z,
			z: self.x * other.y - self.y * other.x
		}
	}

	pub fn length_squared(&self) -> f64 {
		self.x * self.x + self.y * self.y + self.z * self.z
	}

	pub fn length(&self) -> f64 {
		self.length_squared().sqrt()
	}

	pub fn normalize(&self) -> Self {
		let length = self.length();
		Vec3 {
			x: self.x / length,
			y: self.y / length,
			z: self.z / length,
		}
	}

	pub fn abs(&self) -> Self {
		Vec3 {
			x: self.x.abs(),
			y: self.y.abs(),
			z: self.z.abs()
		}
	}

	pub fn reflect(&self, normal: Vec3) -> Self {
		*self - (normal * 2.0 * self.dot(normal))
	}

	pub fn refract(uv: Vec3, n: Vec3, etai_over_etat: f64) -> Vec3 {
		let cos_theta = (-uv).dot(n).min(1.0);
		let r_out_perp = etai_over_etat * (uv + cos_theta * n);
		let r_out_parallel = -((1.0 - r_out_perp.length_squared()).abs().sqrt()) * n;
		r_out_perp + r_out_parallel
	}

	pub fn near_zero_tolerance(&self, tolerance: f64) -> bool {
		(self.x.abs() <= tolerance) && (self.y.abs() <= tolerance) && (self.z.abs() <= tolerance)
	}

	pub fn near_zero(&self) -> bool {
		self.near_zero_tolerance(0.00000001)
	}

	pub fn largest_component(&self) -> usize {
		if self.x > self.y && self.x > self.z {
			0 // X-axis
		} else if self.y > self.z {
			1 // Y-axis
		} else {
			2 // Z-axis
		}
	}

	pub fn rotate_x(&mut self, theta: f64) {
		let new_y = self.y * theta.cos() + self.z * theta.sin();
		let new_z = -self.y * theta.sin() + self.z * theta.cos();
		self.y = new_y;
		self.z = new_z;
	}

	pub fn rotate_y(&mut self, theta: f64) {
		let new_x = self.x * theta.cos() + self.z * theta.sin();
		let new_z = -self.x * theta.sin() + self.z * theta.cos();
		self.x = new_x;
		self.z = new_z;
	}

	pub fn random(min: f64, max: f64) -> Self {
		Vec3 {
			x: min + (max-min) * rand::random::<f64>(),
			y: min + (max-min) * rand::random::<f64>(),
			z: min + (max-min) * rand::random::<f64>()
		}
	}

	pub fn random_unit_vector() -> Self {
		let p = Self::random(-1.0, 1.0);
		p.normalize()
	}

	pub fn random_in_unit_disk(rand: &mut rand::prelude::ThreadRng) -> Self {
		let p = Vec3::new(random(-1.0, 1.0, rand), random(-1.0, 1.0, rand), 0.0);
		p.normalize()
	}
}

impl Add for Vec3 {
	type Output = Self;

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

	fn sub(self, other: Self) -> Self {
		Vec3 {
			x: self.x - other.x,
			y: self.y - other.y,
			z: self.z - other.z,
		}
	}
}

impl Mul<f64> for Vec3 {
	type Output = Self;

	fn mul(self, scalar: f64) -> Self {
		Vec3 {
			x: self.x * scalar,
			y: self.y * scalar,
			z: self.z * scalar,
		}
	}
}

impl Mul<Vec3> for f64 {
	type Output = Vec3;

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

	fn div(self, other: Vec3) -> Self {
		Vec3 {
			x: self.x / other.x,
			y: self.y / other.y,
			z: self.z / other.z,
		}
	}
}

impl Div<f64> for Vec3 {
	type Output = Self;

	fn div(self, scalar: f64) -> Self {
		Vec3 {
			x: self.x / scalar,
			y: self.y / scalar,
			z: self.z / scalar,
		}
	}
}

impl Div<Vec3> for f64 {
	type Output = Vec3;

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

	fn neg(self) -> Self {
		Vec3 {
			x: -self.x,
			y: -self.y,
			z: -self.z
		}
	}
}

impl Index<usize> for Vec3 {
	type Output = f64;

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
	fn index_mut(&mut self, index: usize) -> &mut f64 {
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
	pub dir: Vec3
}

impl Ray {
	pub fn new(origin: Vec3, dir: Vec3) -> Self {
		Self {
			origin,
			dir,
		}
	}

	pub fn at(&self, t: f64) -> Vec3 {
		self.origin + self.dir * t
	}
}



#[derive(Copy, Clone, PartialEq, Debug)]
pub struct AABB {
	pub min: Vec3,
	pub max: Vec3,
}

impl AABB {
	pub fn new(min: Vec3, max: Vec3) -> Self {
		AABB { min, max }
	}

	pub fn center(&self) -> Vec3 {
		self.min + (self.max - self.min) / 2.0
	}

	pub fn hit(&self, ray: &Ray) -> bool {
		for a in 0..3 {
			let inv_d = 1.0 / ray.dir[a];
			let mut t0 = (self.min[a] - ray.origin[a]) * inv_d;
			let mut t1 = (self.max[a] - ray.origin[a]) * inv_d;
			if inv_d < 0.0 {
				std::mem::swap(&mut t0, &mut t1);
			}
			let min = if t0 > 0.001 {t0} else {0.001};
			let max = if t1 < f64::MAX {t1} else {f64::MAX};
			if max <= min {
				return false;
			}
		}
		true
	}

	// returns axis with the largest size (x = 0, y = 1, z = 2)
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

	pub fn surrounding(a: AABB, b: AABB) -> Self {
		Self::new(
			Vec3::new(
				f64::min(a.min.x, b.min.x),
				f64::min(a.min.y, b.min.y),
				f64::min(a.min.z, b.min.z)
			),
			Vec3::new(
				f64::max(a.max.x, b.max.x),
				f64::max(a.max.y, b.max.y),
				f64::max(a.max.z, b.max.z)
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