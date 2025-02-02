use glam::Vec3;
use notify::{RecursiveMode, Watcher};
use raytracer_gpu::{Camera, Material, Renderer, Sphere};
use std::{
	collections::HashSet,
	path::PathBuf,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Instant,
};
use winit::{
	dpi::{LogicalSize, Size},
	event::*,
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
};

fn main() {
	pollster::block_on(run());
}

async fn run() {
	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_title("WGPU Compute Raytracer")
		.with_inner_size(Size::Logical(LogicalSize::new(100.0, 100.0)))
		.build(&event_loop)
		.unwrap();

	let mut camera = Camera::new(Vec3::new(0.0, 0.0, 0.0));

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

	// different metalic spheres
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

	let mut renderer = Renderer::new(&window, &spheres, camera).await;

	let mut last_redraw = Instant::now();

	let mut pressed_key_codes = HashSet::<VirtualKeyCode>::new();

	let shader_code_changed_flag = Arc::new(AtomicBool::new(false));
	let shader_code_changed_flag_clone = shader_code_changed_flag.clone();
	let mut shader_code_file_watcher = notify::recommended_watcher(move |result| match result {
		Ok(notify::Event { kind, .. }) => {
			if matches!(kind, notify::EventKind::Modify(_)) {
				shader_code_changed_flag_clone.store(true, Ordering::Relaxed)
			}
		}
		Err(e) => eprintln!("failed to listen to shader code file changes: {e}"),
	})
	.expect("failed to create shader code file watcher");
	shader_code_file_watcher
		.watch(&PathBuf::from("src/shaders"), RecursiveMode::Recursive)
		.expect("failed to watch shader code files");

	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Poll;

		match event {
			Event::RedrawRequested(_) => {
				let delta_time = Instant::now() - last_redraw;
				println!(
					"FPS = {:.0} ({delta_time:?}), FRAME = {}",
					1.0 / delta_time.as_secs_f64(),
					renderer.frame_counter
				);
				last_redraw = Instant::now();

				if shader_code_changed_flag.load(Ordering::Relaxed) {
					println!("HOT RELOADING SHADERS...");
					if let Err(e) = renderer.hot_reload_shaders_from_files(
						"src/shaders/compute.wgsl",
						"src/shaders/render.wgsl",
					) {
						eprintln!("COULD NOT HOT RELOAD SHADERS: {e}");
					}
					println!("HOT RELOADING SHADERS FINISHED!");

					shader_code_changed_flag.store(false, Ordering::Relaxed);
				}

				camera_input_controller(&mut camera, &pressed_key_codes, delta_time.as_secs_f32());

				renderer.update_camera(camera);

				renderer.update();
			}
			Event::MainEventsCleared => {
				window.request_redraw();
			}
			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				..
			} => *control_flow = ControlFlow::Exit,
			Event::WindowEvent {
				event: WindowEvent::Resized(new_size),
				..
			} => {
				renderer.resize(new_size);

				window.request_redraw();
			}
			Event::WindowEvent {
				event: WindowEvent::KeyboardInput { input, .. },
				..
			} => match input.state {
				ElementState::Pressed => {
					if let Some(virtual_key_codes) = input.virtual_keycode {
						if matches!(virtual_key_codes, VirtualKeyCode::Escape) {
							*control_flow = ControlFlow::ExitWithCode(0);
						}

						pressed_key_codes.insert(virtual_key_codes);
					}
				}
				ElementState::Released => {
					if let Some(virtual_key_codes) = input.virtual_keycode {
						pressed_key_codes.remove(&virtual_key_codes);
					}
				}
			},
			_ => {}
		}
	});
}

fn camera_input_controller(
	camera: &mut Camera,
	pressed_key_scancodes: &HashSet<VirtualKeyCode>,
	dt: f32,
) {
	const SPEED: f32 = 4.0;

	let input_is_key_down = |key_code| -> bool { pressed_key_scancodes.contains(&key_code) };

	let speed = SPEED;
	/*if input_is_key_down(VirtualKeyCode::ShiftLeft) {
		speed *= 2.0;
	}
	if input_is_key_down(KeyCode::ControlLeft) {
		speed *= 0.5;
	}*/

	let mut amount_forward = if input_is_key_down(VirtualKeyCode::W) {
		1.0
	} else {
		0.0
	};
	if input_is_key_down(VirtualKeyCode::S) {
		amount_forward -= 1.0;
	}
	camera.position += camera.get_forward() * amount_forward * speed * dt;

	let mut amount_right = if input_is_key_down(VirtualKeyCode::A) {
		-1.0
	} else {
		0.0
	};
	if input_is_key_down(VirtualKeyCode::D) {
		amount_right += 1.0;
	}
	camera.position += camera.get_right() * amount_right * speed * dt;

	let mut amount_up = if input_is_key_down(VirtualKeyCode::E) {
		1.0
	} else {
		0.0
	};
	if input_is_key_down(VirtualKeyCode::Q) {
		amount_up -= 1.0;
	}
	camera.position += camera.get_up() * amount_up * speed * dt;
}
