use indicatif::{ProgressStyle, ParallelProgressIterator};
use image::*;
use minifb::MouseButton;
use rayon::iter::ParallelIterator;
use rayon::iter::IntoParallelIterator;
use minifb::{Key, Window, WindowOptions};
use std::sync::{Arc, Mutex};
use itertools::Itertools;
use rust_tracer::math::*;
use rust_tracer::scene::*;
use rust_tracer::camera::Camera;
use rust_tracer::materials::*;

fn vec3_to_rgb(v: &Vec3) -> image::Rgb<u8> {
	Rgb([(v.x * 255.0) as u8, (v.y * 255.0) as u8, (v.z * 255.0) as u8])
}

fn rgb_to_u32(r: u32, g: u32, b: u32) -> u32 {
    (r << 16) | (g << 8) | b
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
	let width = 2560;
	let height = 1440;//(width * (16 / 9)) as usize;
	let mut image = Arc::new(Mutex::new(image::RgbImage::new(width, height)));
	let image_buffer_clone = Arc::clone(&image);

	let mut window = Window::new("Rust Raytracer",
	width as usize,
	height as usize,
	WindowOptions {
		resize: true,
		scale: minifb::Scale::X1,
		..WindowOptions::default()
	}).unwrap_or_else(|e| {
        panic!("Unable to create window: {}", e);
    });

	let render_finished = Arc::new(Mutex::new(false));
	let render_finished_clone = render_finished.clone();

	// Spawn a thread for image generation
	std::thread::spawn(move || {
		loop {
			if !*render_finished.lock().unwrap() {
				render(width, height, &image_buffer_clone);
				*render_finished.lock().unwrap() = true;
			}
		}
	});

	while window.is_open() && !window.is_key_down(Key::Escape) {
        // Update the window
		let mut buffer: Vec<u32> = vec![0; width as usize * height as usize];
		for (i, pixel) in image.lock().unwrap().as_raw().chunks(3).enumerate() {
			buffer[i] = rgb_to_u32(pixel[0] as u32, pixel[1] as u32, pixel[2] as u32)
		}
        window.update_with_buffer(&buffer, width as usize, height as usize).unwrap();

		if window.get_mouse_down(MouseButton::Right) {
			*render_finished_clone.lock().unwrap() = false;
		}
	}
}

fn render(width: u32, height: u32, image_buffer: &Arc<Mutex<RgbImage>>) {
	let samples_per_pixel = 500;
	let max_depth = 30;

	let camera = Camera::new(
		Vec3::new(13.0, 1.5, 3.0), 
		20.0,
		&Vec3::new(0.0, 0.0, 0.0), 
		&Vec3::new(0.0, 1.0, 0.0),
		10.0, 0.6,
		width as usize, height as usize);
	
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

	//let progress_bar_style = ProgressStyle::with_template("{elapsed} {percent}% {wide_bar:.green/white}").unwrap();
	let inv_samples_per_pixel = 1.0 / samples_per_pixel as f64;
	(0..height as usize)
		.cartesian_product(0..width as usize)
		.collect::<Vec<(usize, usize)>>()
		.into_par_iter()
		//.progress_count(width as u64 * height as u64).with_style(progress_bar_style).with_finish(indicatif::ProgressFinish::Abandon)
		.for_each(|(y, x)| {
			let mut final_color = Vec3::zero();
			for _ in 0..samples_per_pixel {
				let ray = camera.get_ray(x as f64, y as f64);
				final_color = final_color + ray_color(&ray, &scene, max_depth);
			}
			final_color = final_color * inv_samples_per_pixel as f64;
			final_color = Vec3::new(linear_to_gamma(final_color.x), linear_to_gamma(final_color.y), linear_to_gamma(final_color.z));
			let mut image = image_buffer.lock().unwrap();
			image.put_pixel(x as u32, y as u32, vec3_to_rgb(&final_color));
		});

	image_buffer.lock().unwrap().save("output.png").unwrap();
}