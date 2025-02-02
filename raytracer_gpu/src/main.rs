use glam::Vec3;
use raytracer_gpu::{Camera, Renderer, Sphere};
use std::{collections::HashSet, time::Instant};
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

	let spheres = vec![
		Sphere {
			position: Vec3::new(0.0, 1.0, -2.0),
			emission: 0.5,
			color: Vec3::new(1.0, 0.0, 0.0),
			radius: 0.5,
		},
		Sphere {
			position: Vec3::new(1.0, 0.5, -3.0),
			emission: 0.0,
			color: Vec3::new(0.75, 0.75, 0.75),
			radius: 0.8,
		},
		Sphere {
			position: Vec3::new(-1.0, -0.5, -4.0),
			emission: 0.0,
			color: Vec3::new(0.75, 0.75, 0.75),
			radius: 1.0,
		},
		Sphere {
			position: Vec3::new(-1.0, 1.0, -4.0),
			emission: 0.0,
			color: Vec3::new(0.75, 0.75, 0.75),
			radius: 0.4,
		},
		// sun
		Sphere {
			position: Vec3::new(10000.0, 5000.0, 10000.0),
			emission: 60.0,
			color: Vec3::new(0.8, 0.4, 0.2),
			radius: 5000.0,
		},
		// ground
		Sphere {
			position: Vec3::new(0.0, -100002.0, 0.0),
			emission: 0.0,
			color: Vec3::new(0.5, 0.5, 0.5),
			radius: 100000.0,
		},
	];

	let mut renderer = Renderer::new(&window, &spheres, camera).await;

	let mut last_redraw = Instant::now();

	let mut pressed_key_codes = HashSet::<VirtualKeyCode>::new();

	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Poll;

		match event {
			Event::RedrawRequested(_) => {
				let delta_time = Instant::now() - last_redraw;
				println!(
					"FPS = {:.0}, {delta_time:?}",
					1.0 / delta_time.as_secs_f64()
				);
				last_redraw = Instant::now();

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
