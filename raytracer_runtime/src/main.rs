use rayon::prelude::*;
use image::*;
use minifb::MouseButton;
use minifb::{Key, Window, WindowOptions};
use std::time::Instant;
use raytracer::math::*;
use raytracer::scene::*;
use raytracer::camera::Camera;
use raytracer::materials::*;

fn vec3_to_rgb(v: &Vec3) -> image::Rgb<u8> {
	Rgb([(v.x * 255.0) as u8, (v.y * 255.0) as u8, (v.z * 255.0) as u8])
}

fn rgb_to_u32(r: u32, g: u32, b: u32) -> u32 {
	(r << 16) | (g << 8) | b
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

fn get_camera_rotation(yaw: f64, pitch: f64) -> Vec3 {
	let pitch_radians = pitch.to_radians();
	let yaw_radians = yaw.to_radians();
	Vec3::new(
		yaw_radians.cos() * pitch_radians.cos(),
		pitch_radians.sin(),
		yaw_radians.sin() * pitch_radians.cos()
	)
}

fn main() {
	let width = 800;//2560;
	let height = 600;//1440;//(width * (16 / 9)) as usize;
	let max_depth = 50;

	let mut accum_image: Vec<Vec3> = Vec::new();
	accum_image.resize(width * height, Vec3::zero());
	let mut frame_count = 1;
	let mut final_image = image::RgbImage::new(width as u32, height as u32);

	let mut window = Window::new("Raytracer - Runtime", width, height,
	WindowOptions {
		resize: true,
		scale: minifb::Scale::X1,
		..WindowOptions::default()
	}).unwrap_or_else(|e| {
		panic!("Unable to create window: {}", e);
	});

	let mut yaw = 0.0;
	let mut pitch = 0.0;
	let mut camera = Camera::new(
		Vec3::new(1.0, 1.5, 3.0), 
		get_camera_rotation(yaw, pitch), 
		90.0,
		10.0, 0.6,
		width, height);
	


	let mut scene = Scene::new();

	let material_ground = Material::Lambertain{ albedo: Vec3::new(0.5, 0.5, 0.5) };
	scene.add_cube(Vec3::new(0.0,-1000.0,0.0), Vec3::uniform(1000.0), material_ground);

	let mat1 = Material::Dielectric{ ir: 1.5 };
	let mat2 = Material::Lambertain{ albedo: Vec3::new(0.4, 0.2, 0.1) };
	let mat3 = Material::Metal{ albedo: Vec3::new(0.7, 0.6, 0.5), fuzz: 0.0 };
	scene.add_sphere(Vec3::new(0.0, 1.0, 0.0), 1.0, mat1);
	scene.add_sphere(Vec3::new(0.0, 1.0, 0.0), -0.98, mat1);
	scene.add_sphere(Vec3::new(4.0, 1.0, 0.0), 1.0, mat2);
	scene.add_sphere(Vec3::new(-4.0, 1.0, 0.0), 1.0, mat3);
	scene.add_cube(Vec3::new(-4.0, 0.5, 2.5), Vec3::uniform(0.8), mat2);

	let mut last_mouse_pos: (f32, f32) = window.get_mouse_pos(minifb::MouseMode::Clamp).unwrap();
	let mut last_update = Instant::now();

	while window.is_open() && !window.is_key_down(Key::Escape) {
		let now = Instant::now();
		let delta_time = (now - last_update).as_secs_f64();
		last_update = now;

		accum_image
			.par_chunks_exact_mut(width)
			.enumerate()
			.for_each(|(y, row)| {
				let mut rand = rand::thread_rng();
				for (x, output_color) in row.iter_mut().enumerate() {
					let ray = camera.get_ray(x as f64, y as f64, &mut rand);
					let mut final_color = ray_color(&ray, &scene, max_depth);
					final_color = Vec3::new(linear_to_gamma(final_color.x), linear_to_gamma(final_color.y), linear_to_gamma(final_color.z));
					*output_color = *output_color + final_color;
				}
			});

		for y in 0..height {
			for x in 0..width {
				let image_index = y * width + x;
				final_image.put_pixel(x as u32, y as u32, vec3_to_rgb(&(accum_image[image_index] / (frame_count as f64))));
			}
		}
		for (index, pixel) in final_image.pixels_mut().enumerate() {
			*pixel = vec3_to_rgb(&(accum_image[index] / (frame_count as f64)));
		}
		frame_count += 1;

		let mouse_pos = window.get_mouse_pos(minifb::MouseMode::Clamp).unwrap();
			
		if window.get_mouse_down(MouseButton::Right) {
			accum_image.resize(0, Vec3::zero());
			accum_image.resize(width * height, Vec3::zero());

			// camera rotation
			yaw += (mouse_pos.0 - last_mouse_pos.0) as f64 * 0.25;
			pitch += (last_mouse_pos.1 - mouse_pos.1) as f64 * 0.25;
			pitch = pitch.clamp(-90.0, 90.0);
			let direction = get_camera_rotation(yaw, pitch);

			let mut forward = 0.0;
			if window.is_key_down(Key::W) {
				forward += 1.0;
			}
			if window.is_key_down(Key::S) {
				forward -= 1.0;
			}
			let mut left = 0.0;
			if window.is_key_down(Key::A) {
				left -= 1.0;
			}
			if window.is_key_down(Key::D) {
				left += 1.0;
			}
			let mut up = 0.0;
			if window.is_key_down(Key::E) {
				up += 1.0;
			}
			if window.is_key_down(Key::Q) {
				up -= 1.0;
			}
			let mut move_dir = direction * forward + direction.cross(Vec3::new(0.0, 1.0, 0.0)) * left;
			move_dir = if move_dir == Vec3::new(0.0, 0.0, 0.0) { Vec3::new(0.0, 0.0, 0.0) } else { move_dir.normalize() };
			camera.origin = camera.origin + move_dir * delta_time * 5.0;
			camera.origin.y += up * delta_time * 5.0;
			camera = Camera::new(
				camera.origin, 
				direction, 
				90.0,
				10.0, 0.6,
				width, height);
			frame_count = 1;
		}
		last_mouse_pos = mouse_pos;

		// Update the window
		let mut buffer: Vec<u32> = vec![0; width * height];
		for (i, pixel) in final_image.as_raw().chunks(3).enumerate() {
			buffer[i] = rgb_to_u32(pixel[0] as u32, pixel[1] as u32, pixel[2] as u32)
		}
		window.update_with_buffer(&buffer, width, height).unwrap();
	}
}