use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use raytracer::math::*;
use raytracer::scene::*;
use raytracer::camera::Camera;
use raytracer::materials::*;


fn vec3_to_rgb(v: &Vec3) -> image::Rgb<u8> {
	image::Rgb([(v.x * 255.0) as u8, (v.y * 255.0) as u8, (v.z * 255.0) as u8])
}

fn linear_to_gamma(linear: f64) -> f64 {
	linear.sqrt()
}

fn ray_color(ray: &Ray, scene: &Scene, depth: i32) -> Vec3 {
	if depth <= 0 {
		return Vec3::zero();
	}
	
	if let Some(hit) = scene.trace(ray) {
		if let Some(scattered) = hit.material.scatter(ray, &hit) {
			return scattered.attenuation * ray_color(&scattered.scattered, scene, depth - 1);
		}
		return Vec3::zero();
	}
	
	// sky
	let unit_dir = ray.dir.normalize();
	let a = 0.5 * (unit_dir.y + 1.0);
	Vec3::one() * (1.0-a) + Vec3::new(0.5, 0.7, 1.0) * a
}

fn main() {
	let width = 2560;
	let height = 1440;
	let max_depth = 50;
	let samples = 250;

	let camera = Camera::new(
		Vec3::new(13.0, 1.5, 3.0),
		-Vec3::new(13.0, 1.5, 3.0).normalize(), 
		20.0,
		10.0, 0.6,
		width, height);
	
	let mut scene = Scene::new();

	let material_ground = Material::Lambertain{ albedo: Vec3::new(0.5, 0.5, 0.5) };
	scene.add_cube(Vec3::new(0.0,-1000.0,0.0), Vec3::uniform(1000.0), material_ground);

	
	for a in -11..11 {
		for b in -11..11 {
			let random_mat = rand::random::<f64>();
			let center = Vec3::new(a as f64 + 0.9 * rand::random::<f64>(), 0.2, b as f64 + 0.9 * rand::random::<f64>());

			if (center - Vec3::new(4.0, 0.2, 0.0)).length() > 0.9 {
				if random_mat < 0.35 {
					// diffuse
					let albedo = Vec3::random(0.0, 1.0) * Vec3::random(0.0, 1.0);
					let material = Material::Lambertain{ albedo };
					if random(0.0, 1.0) > 0.5 {
						scene.add_sphere(center, 0.2, material);
					}
					else {
						scene.add_cube(center, Vec3::uniform(0.2), material);
					}
				} else if random_mat < 0.85 {
					// metal
					let albedo = Vec3::random(0.5, 1.0);
					let fuzz = random(0.0, 0.3);
					let material = Material::Metal{ albedo, fuzz };
					if random(0.0, 1.0) > 0.5 {
						scene.add_sphere(center, 0.2, material);
					}
					else {
						scene.add_cube(center, Vec3::uniform(0.2), material);
					}
				} else {
					// glass
					let material = Material::Dielectric{ ir: 1.5 };
					if random(0.0, 1.0) > 0.5 {
						scene.add_sphere(center, 0.2, material.clone());
						scene.add_sphere(center, -0.19, material)
					}
					else {
						scene.add_cube(center, Vec3::uniform(0.2), material.clone());
						scene.add_cube(center, Vec3::uniform(-0.19), material);
					}
				}
			}
		}
	}
	

	let mat1 = Material::Dielectric{ ir: 1.5 };
	let mat2 = Material::Lambertain{ albedo: Vec3::new(0.4, 0.2, 0.1) };
	let mat3 = Material::Metal{ albedo: Vec3::new(0.7, 0.6, 0.5), fuzz: 0.0 };
	scene.add_sphere(Vec3::new(0.0, 1.0, 0.0), 1.0, mat1.clone());
	scene.add_sphere(Vec3::new(0.0, 1.0, 0.0), -0.98, mat1.clone());
	scene.add_sphere(Vec3::new(4.0, 1.0, 0.0), 1.0, mat2.clone());
	scene.add_sphere(Vec3::new(-4.0, 1.0, 0.0), 1.0, mat3.clone());
	scene.add_cube(Vec3::new(-4.0, 0.5, 2.5), Vec3::uniform(0.8), mat2.clone());

	let mut output = vec![Vec3::zero(); width * height];
	output
		.par_chunks_exact_mut(width)
		.progress()
		.enumerate()
		.for_each(|(y, row)| {
			for (x, output_color) in row.iter_mut().enumerate() {
				let mut final_color = Vec3::zero();
				for _ in 0..samples {
					let ray = camera.get_ray(x as f64, y as f64);
					final_color = final_color + ray_color(&ray, &scene, max_depth);
				}
				final_color = final_color / samples as f64;
				*output_color = Vec3::new(linear_to_gamma(final_color.x), linear_to_gamma(final_color.y), linear_to_gamma(final_color.z));
			}
		});

	let mut image = image::RgbImage::new(width as u32, height as u32);
	for y in 0..height {
		for x in 0..width {
			image.put_pixel(x as u32, y as u32, vec3_to_rgb(&output[y * width + x]));
		}
	}
	image.save("output.png").expect("failed to save to output.png");
}