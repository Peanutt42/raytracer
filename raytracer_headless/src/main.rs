use indicatif::ParallelProgressIterator;
use std::time::Instant;
use rayon::prelude::*;
use raytracer::{Vec3, BVH, Camera, Scene, render};

fn vec3_to_rgb(v: &Vec3) -> image::Rgb<u8> {
	image::Rgb([(v.x * 255.0) as u8, (v.y * 255.0) as u8, (v.z * 255.0) as u8])
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
					final_color = final_color + render(
						x as f64,
						y as f64,
						&camera,
						&bvh,
						max_depth,
						&mut rand
					);
				}
				*output_color = final_color.linear_to_gamma() / samples as f64;
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