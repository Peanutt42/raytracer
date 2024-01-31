use crate::math::*;
use crate::materials::*;
use crate::shapes::*;

pub trait Object {
	fn hit(&self, ray: &Ray) -> Option<f64>;

	// Returns normal and if it is front-/back-face
	fn get_normal(&self, p: &Vec3, ray: &Ray) -> Vec3;

	fn get_material(&self) -> Option<Material>;

	fn get_aabb(&self) -> AABB;
}


pub struct RayHit {
	pub point: Vec3,
	pub normal: Vec3,
	pub material: Material,
	pub front_face: bool,
}

impl RayHit {
	pub fn new(point: Vec3, normal: Vec3, material: Material, front_face: bool) -> Self {
		RayHit {
			point,
			normal,
			material,
			front_face
		}
	}
}

pub struct Scene {
	pub objects: Vec<Box<dyn Object + Sync>>,
}

impl Scene {
	pub fn new() -> Self {
		Self {
			objects: Vec::new(),
		}
	}

	pub fn add_sphere(&mut self, center: Vec3, radius: f64, material: Material) {
		self.objects.push(Box::new(sphere::Sphere::new(center, radius, material)));
	}

	pub fn add_cube(&mut self, center: Vec3, size: Vec3, material: Material) {
		self.objects.push(Box::new(cube::Cube::new(center, size, material)));
	}

	pub fn trace(&self, ray: &Ray) -> Option<RayHit> {
		let mut closest_hit_distance = f64::MAX;
		let mut closest_sphere: Option<&Box<dyn Object + Sync>> = None;
		for object in self.objects.iter() {
			if let Some(t) = object.hit(ray) {
				if t > 0.001 && t < closest_hit_distance {
					closest_hit_distance = t;
					closest_sphere = Some(object);
				}
			}
		}

		if let Some(object) = closest_sphere {            
			return get_ray_hit(object.as_ref(), closest_hit_distance, ray);
		}

		None
	}
}

impl Default for Scene {
	fn default() -> Self {
		Self::new()
	}
}

fn get_ray_hit(object: &(dyn Object + Sync), distance: f64, ray: &Ray) -> Option<RayHit> {
	if let Some(material) = object.get_material() {
		let p = ray.at(distance);
		let mut normal = object.get_normal(&p, ray);
		let front_face = ray.dir.dot(&normal) < 0.0;
		if !front_face {
			normal = -normal;
		}
		return Some(RayHit::new(p, normal, material, front_face));
	}
	None
}