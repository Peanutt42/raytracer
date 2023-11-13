use crate::math::{Vec3, Ray};

#[derive(Clone, Copy)]
pub struct Sphere {
	pub center: Vec3,
	pub radius: f64,
}

impl Sphere {
	pub fn new(center: Vec3, radius: f64) -> Self {
		Sphere {
			center: center,
			radius: radius
		}
	}

	pub fn hit(&self, ray: &Ray) -> Option<f64> {
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

	pub fn get_normal(&self, p: &Vec3) -> Vec3 {
		(*p - self.center) / self.radius
	}
}

pub struct RayHit<'a> {
	pub point: Vec3,
	pub normal: Vec3,
	pub object: &'a Sphere
}

impl<'a> RayHit<'a> {
	pub fn new(point: Vec3, normal: Vec3, object: &'a Sphere) -> Self {
		RayHit {
			point: point,
			normal: normal,
			object: object
		}
	}
}

pub struct Scene {
	pub spheres: Vec<Sphere>
}

impl Scene {
	pub fn new() -> Self {
		Scene{ spheres: Vec::new() }
	}

	pub fn trace(&self, ray: &Ray) -> Option<RayHit> {
		let mut closest_hit_distance = f64::MAX;
		let mut closest_sphere: Option<&Sphere> = None;
		for sphere in self.spheres.iter() {
			if let Some(t) = sphere.hit(&ray) {
				if t > 0.0 && t < closest_hit_distance {
					closest_hit_distance = t;
					closest_sphere = Some(&sphere);
				}
			}
		}

		if let Some(sphere) = closest_sphere {            
			let hit_point = ray.at(closest_hit_distance);

			let normal = sphere.get_normal(&hit_point);
	
			return Some(RayHit::new(hit_point, normal, sphere));
		}

		None
	}
}