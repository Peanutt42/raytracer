use indicatif::{ProgressStyle, ParallelProgressIterator};
use image::{RgbImage, Rgb};
use rayon::iter::ParallelIterator;
use rayon::iter::IntoParallelIterator;
use itertools::Itertools;
use rust_tracer::math::*;
use rust_tracer::scene::*;
use rust_tracer::camera::Camera;
use rust_tracer::materials::*;


fn vec3_to_rgb(v: &Vec3) -> Rgb<u8> {
	Rgb([(v.x * 256.0) as u8, (v.y * 256.0) as u8, (v.z * 256.0) as u8])
}

fn linear_to_gamma(linear: f64) -> f64 {
	linear.sqrt()
}

fn ray_color(ray: &Ray, scene: &Scene, depth: usize) -> Vec3 {
	if depth <= 0 {
		return Vec3::zero();
	}
	
	if let Some(hit) = scene.trace(&ray) {
		if let Some(scattered) = hit.material.scatter(&ray, &hit) {
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
	let width = 650;//2560;
	let height = 300;//1440;//(width * (16 / 9)) as usize;
	let samples_per_pixel = 70;//500;
	let max_depth = 30;

	let camera = Camera::new(
		Vec3::new(13.0, 2.0, 3.0), 
		20.0,
		&Vec3::new(0.0, 0.0, 0.0), 
		&Vec3::new(0.0, 1.0, 0.0),
		10.0, 0.6,
		width, height);
	
	let mut scene = Scene::new();

	let material_ground = Material::Lambertain{ albedo: Vec3::new(0.5, 0.5, 0.5) };
	scene.add_sphere(Vec3::new(0.0,-1000.0,0.0), 1000.0, material_ground);

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
					scene.add_sphere(center, 0.2, material.clone());
					scene.add_sphere(center, -0.19, material)
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
	scene.add_cube(Vec3::new(-4.0, 0.5, 2.5), Vec3::uniform(0.8), mat3.clone());

	let progress_bar_style = ProgressStyle::with_template("{elapsed} {percent}% {wide_bar:.green/white}").unwrap();
	let inv_samples_per_pixel = 1.0 / samples_per_pixel as f64;
	let image_buffer = (0..height)
		.cartesian_product(0..width)
		.collect::<Vec<(usize, usize)>>()
		.into_par_iter()
		.progress_count(width as u64 * height as u64).with_style(progress_bar_style).with_finish(indicatif::ProgressFinish::Abandon)
		.map(|(y, x)| {
			let mut final_color = Vec3::zero();
			for _ in 0..samples_per_pixel {
				let ray = camera.get_ray(x as f64, y as f64);
				final_color = final_color + ray_color(&ray, &scene, max_depth);
			}
			final_color = final_color * inv_samples_per_pixel as f64;
			final_color = Vec3::new(linear_to_gamma(final_color.x), linear_to_gamma(final_color.y), linear_to_gamma(final_color.z));
			vec3_to_rgb(&final_color)
		})
		.collect::<Vec<Rgb<u8>>>();

	let mut image = RgbImage::new(width as u32, height as u32);
	for y in 0..height {
		for x in 0..width {
			image.put_pixel(x as u32, y as u32, image_buffer[y * width + x]);
		}
	}
	if let Err(error) = image.save("output.png") {
		println!("\nFailed to save to output.png: {}", error.to_string());
	}
	else {
		println!("\nFinished");
	}

}
