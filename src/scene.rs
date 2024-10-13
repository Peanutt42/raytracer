use crate::{Sphere, Cube, Ray, AABB, Vec3, Material, random};

pub trait Hittable {
	fn hit(&self, ray: &Ray) -> Option<f64>;
}

pub trait Bounded {
	fn get_aabb(&self) -> AABB;
}

pub trait Renderable {
	// Returns normal and if it is front-/back-face
	fn get_normal(&self, p: &Vec3, ray: &Ray) -> Vec3;

	fn get_material(&self) -> Option<&Material>;
}


pub struct RayHit<'a> {
	pub point: Vec3,
	pub normal: Vec3,
	pub material: &'a Material,
	pub front_face: bool,
}

impl<'a> RayHit<'a> {
	pub fn new(point: Vec3, normal: Vec3, material: &'a Material, front_face: bool) -> Self {
		RayHit {
			point,
			normal,
			material,
			front_face
		}
	}
}

#[derive(Copy, Clone, Debug)]
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

	fn get_material(&self) -> Option<&Material> {
		match self {
			Self::Sphere(sphere) => sphere.get_material(),
			Self::Cube(cube) => cube.get_material(),
		}
	}
}

#[derive(Debug)]
pub struct Scene {
	pub objects: Vec<Object>,
}

impl Scene {
	pub fn new(objects: Vec<Object>) -> Self {
		Self {
			objects,
		}
	}

	pub fn add_sphere(&mut self, center: Vec3, radius: f64, material: Material) {
		self.objects.push(Object::Sphere(Sphere::new(center, radius, material)));
	}

	pub fn add_cube(&mut self, center: Vec3, size: Vec3, material: Material) {
		self.objects.push(Object::Cube(Cube::new(center, size, material)));
	}

	pub fn hit(&self, ray: &Ray) -> Option<(f64, &Object)> {
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

		closest_object.map(|object| (closest_hit_distance, object))
	}

	pub fn get_sky_color(ray_dir: Vec3) -> Vec3 {
		let unit_dir = ray_dir.normalize();
		let a = 0.5 * (unit_dir.y + 1.0);
		Vec3::one() * (1.0-a) + Vec3::new(0.5, 0.7, 1.0) * a
	}

	pub fn create_sample_scene() -> Self {
		let mut scene = Scene::new(Vec::new());

		let material_ground = Material::Lambertain{ albedo: Vec3::new(0.5, 0.5, 0.5), emission: 0.0 };
		scene.add_cube(Vec3::new(0.0,-1000.0,0.0), Vec3::uniform(1000.0), material_ground);

		let mat1 = Material::Dielectric{ ir: 1.5 };
		let mat2 = Material::Lambertain{ albedo: Vec3::new(0.4, 0.2, 0.1), emission: 3.0 };
		let mat3 = Material::Metal{ albedo: Vec3::new(0.7, 0.6, 0.5), fuzz: 0.0 };
		let sun_mat = Material::Lambertain { albedo: Vec3::new(0.8, 0.4, 0.2), emission: 20.0 };
		scene.add_sphere(Vec3::new(0.0, 1.0, 0.0), 1.0, mat1);
		scene.add_sphere(Vec3::new(0.0, 1.0, 0.0), -0.98, mat1);
		scene.add_sphere(Vec3::new(4.0, 1.0, 0.0), 1.0, mat2);
		scene.add_sphere(Vec3::new(-4.0, 1.0, 0.0), 1.0, mat3);
		scene.add_sphere(Vec3::new(10000.0, 10000.0, 10000.0), 5000.0, sun_mat);
		scene.add_cube(Vec3::new(-4.0, 0.5, 2.5), Vec3::uniform(0.8), mat2);

		let mut rand = rand::thread_rng();
		for a in -11..11 {
			for b in -11..11 {
				let random_mat = random(0.0, 1.0, &mut rand);
				let center = Vec3::new(a as f64 + 0.9 * random(0.0, 1.0, &mut rand), 0.2, b as f64 + 0.9 * random(0.0, 1.0, &mut rand));

				if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
					if random_mat < 0.35 {
						// diffuse
						let albedo = Vec3::random(0.0, 1.0) * Vec3::random(0.0, 1.0);
						let material = Material::Lambertain{ albedo, emission: random_mat };
						if random(0.0, 1.0, &mut rand) > 0.5 {
							scene.add_sphere(center, 0.2, material);
						}
						else {
							scene.add_cube(center, Vec3::uniform(0.2), material);
						}
					} else if random_mat < 0.85 {
						// metal
						let albedo = Vec3::random(0.5, 1.0);
						let fuzz = random(0.0, 0.3, &mut rand);
						let material = Material::Metal{ albedo, fuzz };
						if random(0.0, 1.0, &mut rand) > 0.5 {
							scene.add_sphere(center, 0.2, material);
						}
						else {
							scene.add_cube(center, Vec3::uniform(0.2), material);
						}
					} else {
						// glass
						let material = Material::Dielectric{ ir: 1.5 };
						if random(0.0, 1.0, &mut rand) > 0.5 {
							scene.add_sphere(center, 0.2, material);
							scene.add_sphere(center, -0.19, material)
						}
						else {
							scene.add_cube(center, Vec3::uniform(0.2), material);
							scene.add_cube(center, Vec3::uniform(-0.19), material);
						}
					}
				}
			}
		}

		scene
	}
}

impl Default for Scene {
	fn default() -> Self {
		Self::new(Vec::default())
	}
}