use glam::Vec3;
use notify::{RecursiveMode, Watcher};
use raytracer_gpu::{create_wallpaper_scene, Camera, Renderer};
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

	let mut camera = Camera::new(Vec3::new(0.0, 0.0, 0.0), -90.0, 0.0, 90.0, 0.1, 1000.0);

	// also see: create_simple_scene, create_sample_scene, create_10_metallic_scene, create_glass_scene, create_wallpaper_scene
	let (spheres, cubes) = create_wallpaper_scene();

	let mut renderer = Renderer::new(&window, &spheres, &cubes, camera).await;

	let mut last_redraw = Instant::now();

	let mut pressed_key_codes = HashSet::<VirtualKeyCode>::new();
	let mut pressed_mouse_buttons = HashSet::<MouseButton>::new();
	let mut shift_down = false;
	let mut ctrl_down = false;
	let mut mouse_delta = (0.0, 0.0);

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
				window.set_title(
					format!(
						"WGPU Compute Raytracer: {:.0} fps ({delta_time:.2?}) frame: {}",
						1.0 / delta_time.as_secs_f64(),
						renderer.frame_counter
					)
					.as_str(),
				);
				last_redraw = Instant::now();

				if shader_code_changed_flag.load(Ordering::Relaxed) {
					println!("HOT RELOADING SHADERS...");
					match renderer.hot_reload_shaders_from_files(
						"src/shaders/compute.wgsl",
						"src/shaders/render.wgsl",
					) {
						Ok(_) => println!("HOT RELOADING SHADERS FINISHED!"),
						Err(e) => eprintln!("COULD NOT HOT RELOAD SHADERS:\n{e}"),
					}

					shader_code_changed_flag.store(false, Ordering::Relaxed);
				}

				camera_input_controller(
					&mut camera,
					&pressed_key_codes,
					&pressed_mouse_buttons,
					&mut mouse_delta,
					ctrl_down,
					shift_down,
					delta_time.as_secs_f32(),
				);

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
			Event::WindowEvent {
				event: WindowEvent::ModifiersChanged(modifiers),
				..
			} => {
				ctrl_down = modifiers.contains(ModifiersState::CTRL);
				shift_down = modifiers.contains(ModifiersState::SHIFT);
			}
			Event::WindowEvent {
				event: WindowEvent::MouseInput { state, button, .. },
				..
			} => match state {
				ElementState::Pressed => {
					pressed_mouse_buttons.insert(button);
				}
				ElementState::Released => {
					pressed_mouse_buttons.remove(&button);
				}
			},
			Event::DeviceEvent {
				event: DeviceEvent::MouseMotion { delta },
				..
			} => {
				mouse_delta.0 += delta.0;
				mouse_delta.1 += delta.1;
			}
			_ => {}
		}
	});
}

fn camera_input_controller(
	camera: &mut Camera,
	pressed_key_codes: &HashSet<VirtualKeyCode>,
	pressed_mouse_buttons: &HashSet<MouseButton>,
	mouse_delta: &mut (f64, f64),
	ctrl_down: bool,
	shift_down: bool,
	dt: f32,
) {
	if !pressed_mouse_buttons.contains(&MouseButton::Right) {
		*mouse_delta = (0.0, 0.0);
		return;
	}

	const SENSITIVITY: f32 = 0.25;
	let delta_yaw = mouse_delta.0 as f32 * SENSITIVITY;
	let delta_pitch = -mouse_delta.1 as f32 * SENSITIVITY;
	camera.yaw += delta_yaw;
	camera.pitch = (camera.pitch + delta_pitch).clamp(-89.9, 89.9);
	*mouse_delta = (0.0, 0.0);

	const SPEED: f32 = 4.0;

	let input_is_key_down = |key_code| -> bool { pressed_key_codes.contains(&key_code) };

	let mut speed = SPEED;
	if shift_down {
		speed *= 2.0;
	}
	if ctrl_down {
		speed *= 0.5;
	}

	let mut move_dir = Vec3::ZERO;

	let mut amount_forward = if input_is_key_down(VirtualKeyCode::W) {
		1.0
	} else {
		0.0
	};
	if input_is_key_down(VirtualKeyCode::S) {
		amount_forward -= 1.0;
	}
	move_dir += camera.get_forward() * amount_forward;

	let mut amount_right = if input_is_key_down(VirtualKeyCode::A) {
		-1.0
	} else {
		0.0
	};
	if input_is_key_down(VirtualKeyCode::D) {
		amount_right += 1.0;
	}
	move_dir += camera.get_right() * amount_right;

	let mut amount_up = if input_is_key_down(VirtualKeyCode::E) {
		1.0
	} else {
		0.0
	};
	if input_is_key_down(VirtualKeyCode::Q) {
		amount_up -= 1.0;
	}
	move_dir += Camera::WORLD_UP * amount_up;

	let move_dir = if move_dir == Vec3::ZERO {
		Vec3::ZERO
	} else {
		move_dir.normalize()
	};
	camera.position += move_dir * speed * dt;
}
