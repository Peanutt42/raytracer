use std::ops::{Add, Sub, Mul, Div, Neg};

pub fn radians(degrees: f64) -> f64 {
	degrees * std::f64::consts::PI / 180.0
}

pub fn random(min: f64, max: f64) -> f64 {
	min + (max - min) * rand::random::<f64>()
}


#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
	pub x: f64,
	pub y: f64,
	pub z: f64
}

impl Vec3 {
	pub fn new(x: f64, y: f64, z: f64) -> Self {
		Vec3 { x, y, z }
	}

	pub fn zero() -> Self {
		Vec3 { x: 0.0, y: 0.0, z: 0.0 }
	}

	pub fn one() -> Self {
		Vec3 { x: 1.0, y: 1.0, z: 1.0 }
	}

	pub fn dot(&self, other: &Self) -> f64 {
		self.x * other.x + self.y * other.y + self.z * other.z
	}

	pub fn cross(&self, other: &Self) -> Self {
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

	pub fn reflect(&self, normal: &Vec3) -> Self {
		*self - (*normal * 2.0 * self.dot(&normal))
	}

	pub fn refract(uv: &Vec3, n: &Vec3, etai_over_etat: f64) -> Vec3 {
		let cos_theta = (-(*uv)).dot(n).min(1.0);
		let r_out_perp = etai_over_etat * (*uv + cos_theta*(*n));
		let r_out_parallel = -((1.0 - r_out_perp.length_squared()).abs().sqrt()) * (*n);
		r_out_perp + r_out_parallel
	}

	pub fn near_zero_tolerance(&self, tolerance: f64) -> bool {
		(self.x.abs() <= tolerance) && (self.y.abs() <= tolerance) && (self.z.abs() <= tolerance)
	}

	pub fn near_zero(&self) -> bool {
		self.near_zero_tolerance(0.00000001)
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

	pub fn random_in_unit_disk() -> Self {
		let p = Vec3::new(random(-1.0, 1.0), random(-1.0, 1.0), 0.0);
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


#[derive(Debug, Copy, Clone)]
pub struct Ray {
	pub origin: Vec3,
	pub dir: Vec3
}

impl Ray {
	pub fn new(origin: Vec3, dir: Vec3) -> Self {
		Ray {
			origin: origin,
			dir: dir,
		}
	}

	pub fn at(&self, t: f64) -> Vec3 {
		self.origin + self.dir * t
	}
}