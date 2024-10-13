use rayon::prelude::*;
use minifb::MouseButton;
use minifb::{Key, Window, WindowOptions};
use std::time::Instant;
use raytracer::{render, Camera, Scalar, Scene, Vec3, BVH};

fn vec3_to_rgb(v: &Vec3) -> [u8; 3] {
	[
		(v.x * 255.0) as u8,
		(v.y * 255.0) as u8,
		(v.z * 255.0) as u8
	]
}

fn rgb_to_u32(r: u32, g: u32, b: u32) -> u32 {
	(r << 16) | (g << 8) | b
}

fn get_camera_rotation(yaw: Scalar, pitch: Scalar) -> Vec3 {
	let pitch_radians = pitch.to_radians();
	let yaw_radians = yaw.to_radians();
	Vec3::new(
		yaw_radians.cos() * pitch_radians.cos(),
		pitch_radians.sin(),
		yaw_radians.sin() * pitch_radians.cos()
	)
}

fn main() {
	let width = 300;//2560;
	let height = 200;//1440;//(width * (16 / 9)) as usize;
	let max_depth = 10;//50;

	let mut accum_image: Vec<Vec3> = Vec::new();
	accum_image.resize(width * height, Vec3::zero());
	let mut frame_count = 1;
	let mut final_image: Vec<Vec3> = Vec::new();
	final_image.resize(width * height, Vec3::zero());

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

	let bvh = BVH::new(Scene::create_sample_scene()).unwrap();

	let mut last_mouse_pos: (Scalar, Scalar) = window.get_mouse_pos(minifb::MouseMode::Clamp)
		.map(|(x, y)| (x as Scalar, y as Scalar))
		.unwrap();
	let mut last_update = Instant::now();

	while window.is_open() && !window.is_key_down(Key::Escape) {
		let now = Instant::now();
		let delta_time = (now - last_update).as_secs_f64() as Scalar;
		last_update = now;

		accum_image
			.par_chunks_exact_mut(width)
			.enumerate()
			.for_each(|(y, row)| {
				let mut rand = rand::thread_rng();
				for (x, output_color) in row.iter_mut().enumerate() {
					*output_color = *output_color + render(
							x as Scalar,
							y as Scalar,
							&camera,
							&bvh,
							max_depth,
							&mut rand
						)
						.linear_to_gamma();
				}
			});

		for y in 0..height {
			for x in 0..width {
				let image_index = y * width + x;
				final_image[image_index] = accum_image[image_index] / (frame_count as Scalar);
			}
		}
		frame_count += 1;

		let mouse_pos = window.get_mouse_pos(minifb::MouseMode::Clamp)
			.map(|(x, y)| (x as Scalar, y as Scalar))
			.unwrap();

		if window.get_mouse_down(MouseButton::Right) {
			accum_image.resize(0, Vec3::zero());
			accum_image.resize(width * height, Vec3::zero());

			// camera rotation
			yaw += (mouse_pos.0 - last_mouse_pos.0) * 0.25;
			pitch += (last_mouse_pos.1 - mouse_pos.1) * 0.25;
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
		let buffer: Vec<u32> = final_image.iter().map(|color| {
			let rgb = vec3_to_rgb(color);
			rgb_to_u32(rgb[0] as u32, rgb[1] as u32, rgb[2] as u32)
		})
		.collect();
		window.update_with_buffer(&buffer, width, height).unwrap();
	}
}