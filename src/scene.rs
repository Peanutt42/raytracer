use crate::math::*;
use crate::materials::*;
use crate::shapes::{Sphere, Cube};

pub trait Hittable {
	fn hit(&self, ray: &Ray) -> Option<f64>;
}

pub trait Bounded {
	fn get_aabb(&self) -> AABB;
}

pub trait Renderable {
	// Returns normal and if it is front-/back-face
	fn get_normal(&self, p: &Vec3, ray: &Ray) -> Vec3;

	fn get_material(&self) -> Option<Material>;
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

#[derive(Copy, Clone)]
pub enum Object {
	Sphere(Sphere),
    Cube(Cube),
}

impl Hittable for Object {
	fn hit(&self, ray: &Ray) -> Option<f64> {
		match self {
			Self::Sphere(sphere) => sphere.hit(ray),
			Self::Cube(cube) => cube.hit(ray),
		}
	}
}

impl Bounded for Object {
	fn get_aabb(&self) -> AABB {
		match self {
			Self::Sphere(sphere) => sphere.get_aabb(),
			Self::Cube(cube) => cube.get_aabb(),
		}
	}
}

impl Renderable for Object {
	fn get_normal(&self, p: &Vec3, _ray: &Ray) -> Vec3 {
		match self {
			Self::Sphere(sphere) => sphere.get_normal(p, _ray),
			Self::Cube(cube) => cube.get_normal(p, _ray),
		}
	}

	fn get_material(&self) -> Option<Material> {
		match self {
			Self::Sphere(sphere) => sphere.get_material(),
			Self::Cube(cube) => cube.get_material(),
		}
	}
}

pub struct Scene {
	pub objects: Vec<Object>,
}

impl Scene {
	pub fn new() -> Self {
		Self {
			objects: Vec::new(),
		}
	}

	pub fn add_sphere(&mut self, center: Vec3, radius: f64, material: Material) {
		self.objects.push(Object::Sphere(Sphere::new(center, radius, material)));
	}

	pub fn add_cube(&mut self, center: Vec3, size: Vec3, material: Material) {
		self.objects.push(Object::Cube(Cube::new(center, size, material)));
	}

	pub fn trace(&self, ray: &Ray) -> Option<RayHit> {
		let mut closest_hit_distance = f64::MAX;
		let mut closest_object: Option<&Object> = None;
		for object in self.objects.iter() {
			if let Some(t) = object.hit(ray) {
				if t > 0.001 && t < closest_hit_distance {
					closest_hit_distance = t;
					closest_object = Some(object);
				}
			}
		}

		if let Some(object) = closest_object {            
			return get_ray_hit(object, closest_hit_distance, ray);
		}

		None
	}
}

impl Default for Scene {
	fn default() -> Self {
		Self::new()
	}
}

fn get_ray_hit(object: &dyn Renderable, distance: f64, ray: &Ray) -> Option<RayHit> {
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