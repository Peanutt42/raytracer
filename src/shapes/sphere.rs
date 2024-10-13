use crate::{Hittable, Bounded, Renderable, Vec3, Material, AABB, Ray};

#[derive(Clone, Copy, Debug)]
pub struct Sphere {
	pub center: Vec3,
	pub material: Material,
	radius: f64,
	aabb: AABB,
}

impl Sphere {
	pub fn new(center: Vec3, radius: f64, material: Material) -> Self {
		let radius_vec3 = Vec3::new(radius, radius, radius);

		Sphere {
			center,
			radius,
			material,
			aabb: AABB::new(center - radius_vec3, center + radius_vec3),
		}
	}
}

impl Hittable for Sphere {
	fn hit(&self, ray: &Ray) -> Option<f64> {
		// a = ray origin
		// b = ray direction
		// r = radius
		// t = hit distance
		// (bx² + by²)t² + (2(axbx + ayby))t + (ax² + ay² - r²) = 0

		let origin = ray.origin - self.center;

		let a = ray.dir.dot(ray.dir);
		let b = 2.0 * origin.dot(ray.dir);
		let c = origin.dot(origin) - self.radius * self.radius;

		// Quadratic forumula discriminant
		// b² - 4ac
		let discriminant = b * b - 4.0 * a * c;
		if discriminant < 0.0 {
			return None;
		}

		// (-b +- sqrt(discriminant)) / 2a
		Some((-b - discriminant.sqrt()) / (2.0 * a))
	}
}

impl Bounded for Sphere {
	fn get_aabb(&self) -> AABB {
		self.aabb
	}
}

impl Renderable for Sphere {
	fn get_normal(&self, p: &Vec3, _ray: &Ray) -> Vec3 {
		(*p - self.center) / self.radius
	}

	fn get_material(&self) -> Option<&Material> {
		Some(&self.material)
	}
}