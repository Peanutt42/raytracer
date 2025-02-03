use glam::Vec3;
use rand::Rng;

pub const SPHERE_BUFFER_BIND_GROUP: u32 = 0;

pub enum Material {
	Lambertain { emission: f32 },
	Metalic { fuzz: f32 },
}

impl Material {
	fn get_type(&self) -> MaterialType {
		match self {
			Self::Lambertain { .. } => MaterialType::LAMBERTAIN,
			Self::Metalic { .. } => MaterialType::METALIC,
		}
	}

	fn get_param1(self) -> f32 {
		match self {
			Self::Lambertain { emission } => emission,
			Self::Metalic { fuzz } => fuzz,
		}
	}
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MaterialType {
	material_type: u32,
}
impl MaterialType {
	const LAMBERTAIN: Self = Self { material_type: 0 };
	const METALIC: Self = Self { material_type: 1 };
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
	position: Vec3,
	radius: f32,
	albedo: Vec3,
	material_type: MaterialType,
	material_param1: f32,
	_padding: Vec3,
}

impl Sphere {
	pub fn new(position: Vec3, radius: f32, albedo: Vec3, material: Material) -> Self {
		Self {
			position,
			radius,
			albedo,
			material_type: material.get_type(),
			material_param1: material.get_param1(),
			_padding: Vec3::ZERO,
		}
	}
}

#[allow(unused)]
pub fn create_10_metalics_scene() -> Vec<Sphere> {
	let mut spheres = vec![
		// glowing red
		Sphere::new(
			Vec3::new(0.0, 1.0, -2.0),
			1.5,
			Vec3::new(1.0, 0.0, 0.0),
			Material::Lambertain { emission: 0.5 },
		),
		// sun
		Sphere::new(
			Vec3::new(10000.0, 10000.0, 10000.0),
			2500.0,
			Vec3::new(0.8, 0.4, 0.2),
			Material::Lambertain { emission: 30.0 },
		),
		// ground
		Sphere::new(
			Vec3::new(0.0, -100002.0, 0.0),
			100000.0,
			Vec3::new(0.5, 0.5, 0.5),
			Material::Lambertain { emission: 0.0 },
		),
	];

	for i in 0..10 {
		spheres.push(Sphere::new(
			Vec3::new(i as f32 - 5.0, -1.0, -3.0),
			0.5,
			Vec3::new(0.75, 0.75, 0.75),
			Material::Metalic {
				fuzz: i as f32 / 10.0,
			},
		));
	}

	spheres
}

pub fn create_sample_scene() -> Vec<Sphere> {
	let mut spheres = Vec::new();

	/*let material_ground = Material::Lambertain {
		//albedo: Vec3::new(0.5, 0.5, 0.5),
		emission: 0.0,
	};
	scene.add_cube(
		Vec3::new(0.0, -1000.0, 0.0),
		Vec3::uniform(1000.0),
		material_ground,
	);*/
	spheres.push(Sphere::new(
		Vec3::new(0.0, -100000.0, 0.0),
		100000.0,
		Vec3::new(0.5, 0.5, 0.5),
		Material::Lambertain { emission: 0.0 },
	));

	//let mat1 = Material::Dielectric { ir: 1.5 };
	let mat2 = Material::Lambertain {
		//albedo: Vec3::new(0.4, 0.2, 0.1),
		emission: 3.0,
	};
	let mat3 = Material::Metalic {
		//albedo: Vec3::new(0.7, 0.6, 0.5),
		fuzz: 0.0,
	};
	let sun_mat = Material::Lambertain {
		//albedo: Vec3::new(0.8, 0.4, 0.2),
		emission: 15.0,
	};
	//spheres.push(Sphere::new(Vec3::new(0.0, 1.0, 0.0), 1.0, mat1));
	//spheres.push(Sphere::new(Vec3::new(0.0, 1.0, 0.0), -0.98, mat1));
	spheres.push(Sphere::new(
		Vec3::new(4.0, 1.0, 0.0),
		1.0,
		Vec3::new(0.4, 0.2, 0.1),
		mat2,
	));
	spheres.push(Sphere::new(
		Vec3::new(-4.0, 1.0, 0.0),
		1.0,
		Vec3::new(0.7, 0.6, 0.5),
		mat3,
	));
	spheres.push(Sphere::new(
		Vec3::new(10000.0, 5000.0, 10000.0),
		7500.0,
		Vec3::new(0.8, 0.4, 0.2),
		sun_mat,
	));
	//scene.add_cube(Vec3::new(-4.0, 0.5, 2.5), Vec3::uniform(0.8), mat2);

	let mut rand = rand::rng();
	let mut random_vec3 = |range: core::ops::Range<f32>| -> Vec3 {
		Vec3::new(
			rand.random_range(range.clone()),
			rand.random_range(range.clone()),
			rand.random_range(range),
		)
	};

	for a in -11..11 {
		for b in -11..11 {
			let random_mat = rand::rng().random_range(0.0..1.0);
			let center = Vec3::new(
				a as f32 + 0.9 * rand::rng().random_range(0.0..1.0),
				0.2,
				b as f32 + 0.9 * rand::rng().random_range(0.0..1.0),
			);

			if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
				if random_mat < 0.35 {
					// diffuse
					let albedo = random_vec3(0.0..1.0) * random_vec3(0.0..1.0);
					let material = Material::Lambertain {
						//albedo,
						emission: random_mat,
					};
					//if rand.random_range(0.0..1.0) > 0.5 {
					spheres.push(Sphere::new(center, 0.2, albedo, material));
					/*} else {
						scene.add_cube(center, Vec3::uniform(0.2), material);
					}*/
				} else if random_mat < 0.85 {
					// metal
					let albedo = random_vec3(0.5..1.0);
					let fuzz = rand::rng().random_range(0.0..0.3);
					let material = Material::Metalic { fuzz };
					//if rand.random_range(0.0..1.0) > 0.5 {
					spheres.push(Sphere::new(center, 0.2, albedo, material));
					/*} else {
						scene.add_cube(center, Vec3::uniform(0.2), material);
					}*/
				} /* else {
					 // glass
					 let material = Material::Dielectric { ir: 1.5 };
					 if random(0.0, 1.0, &mut rand) > 0.5 {
						 scene.add_sphere(center, 0.2, material);
						 scene.add_sphere(center, -0.19, material)
					 } else {
						 scene.add_cube(center, Vec3::uniform(0.2), material);
						 scene.add_cube(center, Vec3::uniform(-0.19), material);
					 }
				 }*/
			}
		}
	}

	spheres
}
