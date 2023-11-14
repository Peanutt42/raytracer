use crate::math::{Vec3, Ray};
use std::rc::Rc;

pub trait Material {
	fn scatter(&self, ray_in: &Ray, hit: &RayHit, attenuation_out: &mut Vec3, scattered_out: &mut Ray) -> bool;
}

#[derive(Clone, Copy)]
pub struct Lambertain {
	pub albedo: Vec3,
}

impl Lambertain {
	pub fn new(albedo: Vec3) -> Self {
		Lambertain { albedo: albedo }
	}
}

impl Material for Lambertain {
	fn scatter(&self, _ray_in: &Ray, hit: &RayHit, attenuation_out: &mut Vec3, scattered_out: &mut Ray) -> bool {
		let mut scatter_direction = hit.normal + Vec3::random_unit_vector();
		if scatter_direction.near_zero() {
			scatter_direction = hit.normal;
		}
		*scattered_out = Ray::new(hit.point, scatter_direction);
		*attenuation_out = self.albedo;
		true
	}
}

#[derive(Clone, Copy)]
pub struct Metal {
	pub albedo: Vec3,
	pub fuzz: f64,
}

impl Metal {
	pub fn new(albedo: Vec3, fuzz: f64) -> Self {
		Metal { albedo: albedo, fuzz: fuzz }
	}
}

impl Material for Metal {
	fn scatter(&self, ray_in: &Ray, hit: &RayHit, attenuation_out: &mut Vec3, scattered_out: &mut Ray) -> bool {
		let reflected = ray_in.dir.normalize().reflect(&hit.normal);
		*scattered_out = Ray::new(hit.point, reflected + self.fuzz * Vec3::random_unit_vector());
		*attenuation_out = self.albedo;
		scattered_out.dir.dot(&hit.normal) > 0.0
	}
}


#[derive(Clone, Copy)]
pub struct Dielectric {
	pub ir: f64, // Index of Refraction
}

impl Dielectric {
	pub fn new(ir: f64) -> Self {
		Dielectric { ir: ir }
	}

	fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
		// Use schlick's approximation for reflectance
		let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
		r0 = r0*r0;
		r0 + (1.0 - r0) * f64::powi(1.0 - cosine, 5)
	}
}

impl Material for Dielectric {
	fn scatter(&self, ray_in: &Ray, hit: &RayHit, attenuation_out: &mut Vec3, scattered_out: &mut Ray) -> bool {
		*attenuation_out = Vec3::one();
		let refration_ratio: f64;
		if hit.front_face {
			refration_ratio = 1.0 / self.ir;
		} else {
			refration_ratio = self.ir;
		}

		let unit_dir = ray_in.dir.normalize();
		let cos_theta = f64::min((-unit_dir).dot(&hit.normal), 1.0);
		let sin_theta = f64::sqrt(1.0 - (cos_theta * cos_theta));

		let cannot_refract = refration_ratio * sin_theta > 1.0;
		let direction: Vec3;
		if cannot_refract {// || (Dielectric::reflectance(cos_theta, refration_ratio) > rand::random::<f64>()) {
			direction = unit_dir.reflect(&hit.normal);
		} else {
			direction = Vec3::refract(&unit_dir, &hit.normal, refration_ratio);
		}

		*scattered_out = Ray::new(hit.point, direction);
		true
	}
}


#[derive(Clone)]
pub struct Sphere {
	pub center: Vec3,
	pub radius: f64,
	pub material: Rc<dyn Material>,
}

impl Sphere {
	pub fn new(center: Vec3, radius: f64, material: Rc<dyn Material>) -> Self {
		Sphere {
			center: center,
			radius: radius,
			material: material
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

	// Returns normal and if it is front-/back-face
	pub fn get_normal(&self, p: &Vec3, ray: &Ray) -> (Vec3, bool) {
		let outward_normal = (*p - self.center) / self.radius;
		let front_face = ray.dir.dot(&outward_normal) < 0.0;
		let normal: Vec3;
		if front_face {
			normal = outward_normal;
		} else {
			normal = -outward_normal;
		}
		(normal, front_face)
	}
}

pub struct RayHit<'a> {
	pub point: Vec3,
	pub normal: Vec3,
	pub object: &'a Sphere,
	pub front_face: bool,
}

impl<'a> RayHit<'a> {
	pub fn new(point: Vec3, normal: Vec3, object: &'a Sphere, front_face: bool) -> Self {
		RayHit {
			point: point,
			normal: normal,
			object: object,
			front_face: front_face
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
				if t > 0.001 && t < closest_hit_distance {
					closest_hit_distance = t;
					closest_sphere = Some(&sphere);
				}
			}
		}

		if let Some(sphere) = closest_sphere {            
			let hit_point = ray.at(closest_hit_distance);

			let (normal, front_face) = sphere.get_normal(&hit_point, &ray);
	
			return Some(RayHit::new(hit_point, normal, sphere, front_face));
		}

		None
	}
}