use crate::math::*;
use crate::materials::*;
use crate::scene::Object;

pub struct Sphere {
	pub center: Vec3,
	pub radius: f64,
	pub material: Material,
}

impl Sphere {
	pub fn new(center: Vec3, radius: f64, material: Material) -> Self {
		Sphere {
			center,
			radius,
			material
		}
	}
}

impl Object for Sphere {
	fn hit(&self, ray: &Ray) -> Option<f64> {
		// a = ray origin
		// b = ray direction
		// r = radius
		// t = hit distance
		// (bx² + by²)t² + (2(axbx + ayby))t + (ax² + ay² - r²) = 0

		let origin = ray.origin - self.center;

		let a = ray.dir.dot(&ray.dir);
		let b = 2.0 * origin.dot(&ray.dir);
		let c = origin.dot(&origin) - self.radius * self.radius;

		// Quadratic forumula discriminant
		// b² - 4ac
		let discriminant = b * b - 4.0 * a * c;
		if discriminant < 0.0 {
			return None;
		}
		
		// (-b +- sqrt(discriminant)) / 2a
		return Some((-b - discriminant.sqrt()) / (2.0 * a));
	}

	fn get_normal(&self, p: &Vec3, _ray: &Ray) -> Vec3 {
		(*p - self.center) / self.radius
	}

	fn get_material(&self) -> Option<Material> {
		Some(self.material.clone())
	}

	fn get_aabb(&self) -> AABB {
		let radius_vec3 = Vec3::new(self.radius, self.radius, self.radius);
		AABB::new(self.center - radius_vec3, self.center + radius_vec3)
	}
}