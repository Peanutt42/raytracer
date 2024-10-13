use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use raytracer::BVH;
use std::time::Instant;
use raytracer::math::*;
use raytracer::scene::*;
use raytracer::camera::Camera;


fn vec3_to_rgb(v: &Vec3) -> image::Rgb<u8> {
	image::Rgb([(v.x * 255.0) as u8, (v.y * 255.0) as u8, (v.z * 255.0) as u8])
}

fn linear_to_gamma(linear: f64) -> f64 {
	linear.sqrt()
}

fn ray_color(ray: &Ray, bvh: &BVH, contribution: &mut Vec3, depth: i32, rand: &mut rand::prelude::ThreadRng) -> Vec3 {
	if depth <= 0 {
		return Vec3::zero();
	}

	if let Some(hit) = bvh.trace(ray) {
		if let Some(scattered) = hit.material.scatter(ray, &hit, rand) {
			*contribution = (*contribution) * scattered.attenuation;
			scattered.attenuation * ray_color(&scattered.scattered, bvh, contribution, depth - 1, rand) + hit.material.emission_color()
		}
		else {
			Vec3::zero()
		}
	}
	else {
		Scene::get_sky_color(ray.dir)
	}
}

fn main() {
	let width = 2560;
	let height = 1440;
	let max_depth = 10;
	let samples = 400;

	let camera = Camera::new(
		Vec3::new(13.0, 1.5, 3.0),
		-Vec3::new(13.0, 1.5, 3.0).normalize(),
		20.0,
		10.0, 0.6,
		width, height);

	let bvh = BVH::new(Scene::create_sample_scene()).unwrap();

	let render_start = Instant::now();

	let mut output = vec![Vec3::zero(); width * height];
	output
		.par_chunks_exact_mut(width)
		.progress()
		.enumerate()
		.for_each(|(y, row)| {
			let mut rand = rand::thread_rng();
			for (x, output_color) in row.iter_mut().enumerate() {
				let mut final_color = Vec3::zero();
				for _ in 0..samples {
					let ray = camera.get_ray(x as f64, y as f64, &mut rand);
					let mut contribution = Vec3::zero();
					final_color = final_color + ray_color(&ray, &bvh, &mut contribution, max_depth, &mut rand);
				}
				final_color = final_color / samples as f64;
				*output_color = Vec3::new(linear_to_gamma(final_color.x), linear_to_gamma(final_color.y), linear_to_gamma(final_color.z));
			}
		});

	println!("Rendering took {}s", (Instant::now() - render_start).as_secs_f32());

	let mut image = image::RgbImage::new(width as u32, height as u32);
	for y in 0..height {
		for x in 0..width {
			image.put_pixel(x as u32, y as u32, vec3_to_rgb(&output[y * width + x]));
		}
	}
	image.save("output.png").expect("failed to save to output.png");
}