use glam::Vec3;
use rand::Rng;

use crate::{Cube, Material, Sphere};

#[allow(unused)]
pub fn create_simple_scene() -> (Vec<Sphere>, Vec<Cube>) {
	let spheres = vec![
		Sphere::new(
			Vec3::new(0.0, 1.0, -2.0),
			0.5,
			Vec3::new(1.0, 0.0, 0.0),
			Material::Lambertain { emission: 0.5 },
		),
		Sphere::new(
			Vec3::new(1.0, 0.5, -3.0),
			0.8,
			Vec3::new(0.75, 0.75, 0.75),
			Material::Metalic { fuzz: 0.05 },
		),
		Sphere::new(
			Vec3::new(-1.0, -0.5, -4.0),
			1.0,
			Vec3::new(0.75, 0.75, 0.75),
			Material::Metalic { fuzz: 0.1 },
		),
		Sphere::new(
			Vec3::new(-1.0, 1.0, -4.0),
			0.4,
			Vec3::new(0.75, 0.75, 0.75),
			Material::Metalic { fuzz: 0.4 },
		),
		// sun
		Sphere::new(
			Vec3::new(10000.0, 5000.0, 10000.0),
			5000.0,
			Vec3::new(0.8, 0.4, 0.2),
			Material::Lambertain { emission: 15.0 },
		),
	];

	let cubes = vec![
		// ground
		Cube::new(
			Vec3::new(0.0, -100002.0, 0.0),
			Vec3::splat(100000.0),
			Vec3::new(0.5, 0.5, 0.5),
			Material::Lambertain { emission: 0.0 },
		),
	];

	(spheres, cubes)
}

#[allow(unused)]
pub fn create_glass_scene() -> (Vec<Sphere>, Vec<Cube>) {
	let mut spheres = Vec::new();
	let mut cubes = Vec::new();
	for i in 0..5 {
		spheres.push(Sphere::new(
			Vec3::new(i as f32 - 5.0, -1.0, -3.0),
			0.5,
			Vec3::splat(1.0),
			Material::Dielectric { ir: 1.5 },
		));
		spheres.push(Sphere::new(
			Vec3::new(i as f32 - 5.0, -1.0, -3.0),
			-0.49,
			Vec3::splat(1.0),
			Material::Dielectric { ir: 1.5 },
		));
	}
	for i in 0..5 {
		cubes.push(Cube::new(
			Vec3::new(1.5 * i as f32 - 5.0, -1.0, -6.0),
			Vec3::splat(0.5),
			Vec3::splat(1.0),
			Material::Dielectric { ir: 1.5 },
		));
	}

	// sun
	spheres.push(Sphere::new(
		Vec3::new(10000.0, 5000.0, 10000.0),
		5000.0,
		Vec3::new(0.8, 0.4, 0.2),
		Material::Lambertain { emission: 15.0 },
	));

	// ground
	cubes.push(Cube::new(
		Vec3::new(0.0, -100002.0, 0.0),
		Vec3::splat(100000.0),
		Vec3::new(0.5, 0.5, 0.5),
		Material::Lambertain { emission: 0.0 },
	));
	(spheres, cubes)
}

#[allow(unused)]
pub fn create_10_metallic_scene() -> (Vec<Sphere>, Vec<Cube>) {
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
	];

	let cubes = vec![
		// ground
		Cube::new(
			Vec3::new(0.0, -100002.0, 0.0),
			Vec3::splat(100000.0),
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

	(spheres, cubes)
}

#[allow(unused)]
pub fn create_sample_scene() -> (Vec<Sphere>, Vec<Cube>) {
	let mut spheres = Vec::new();
	let mut cubes = Vec::new();

	let material_ground = Material::Lambertain { emission: 0.0 };
	cubes.push(Cube::new(
		Vec3::new(0.0, -100000.0, 0.0),
		Vec3::splat(100000.0),
		Vec3::new(0.5, 0.5, 0.5),
		material_ground,
	));

	let mat1 = Material::Dielectric { ir: 1.5 };
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
	spheres.push(Sphere::new(
		Vec3::new(0.0, 1.0, 0.0),
		1.0,
		Vec3::splat(1.0),
		mat1,
	));
	spheres.push(Sphere::new(
		Vec3::new(0.0, 1.0, 0.0),
		-0.98,
		Vec3::splat(1.0),
		mat1,
	));
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
	cubes.push(Cube::new(
		Vec3::new(-4.0, 0.5, 2.5),
		Vec3::splat(0.8),
		Vec3::new(0.4, 0.2, 0.1),
		mat2,
	));

	let mut rand = rand::rng();
	let mut random_vec3 = |range: core::ops::Range<f32>| -> Vec3 {
		Vec3::new(
			rand.random_range(range.clone()),
			rand.random_range(range.clone()),
			rand.random_range(range),
		)
	};

	for a in -6..6 {
		for b in -6..6 {
			let random_mat = rand::rng().random_range(0.0..1.0);
			let center = Vec3::new(
				2.0 * a as f32 + 0.9 * rand::rng().random_range(0.0..1.0),
				0.4,
				2.0 * b as f32 + 0.9 * rand::rng().random_range(0.0..1.0),
			);

			if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
				if random_mat < 0.35 {
					// diffuse
					let albedo = random_vec3(0.0..1.0) * random_vec3(0.0..1.0);
					let material = Material::Lambertain {
						//albedo,
						emission: random_mat,
					};
					if rand::rng().random_range(0.0..1.0) > 0.5 {
						spheres.push(Sphere::new(center, 0.4, albedo, material));
					} else {
						cubes.push(Cube::new(center, Vec3::splat(0.4), albedo, material));
					}
				} else if random_mat < 0.85 {
					// metal
					let albedo = random_vec3(0.5..1.0);
					let fuzz = rand::rng().random_range(0.0..0.3);
					let material = Material::Metalic { fuzz };
					if rand::rng().random_range(0.0..1.0) > 0.5 {
						spheres.push(Sphere::new(center, 0.4, albedo, material));
					} else {
						cubes.push(Cube::new(center, Vec3::splat(0.4), albedo, material));
					}
				} else {
					// glass
					let material = Material::Dielectric { ir: 1.5 };
					if rand::rng().random_range(0.0..1.0) > 0.5 {
						spheres.push(Sphere::new(center, 0.4, Vec3::splat(1.0), material));
						spheres.push(Sphere::new(center, -0.38, Vec3::splat(1.0), material));
					} else {
						cubes.push(Cube::new(
							center,
							Vec3::splat(0.4),
							Vec3::splat(1.0),
							material,
						));
						cubes.push(Cube::new(
							center,
							Vec3::splat(-0.38),
							Vec3::splat(1.0),
							material,
						));
					}
				}
			}
		}
	}

	(spheres, cubes)
}
