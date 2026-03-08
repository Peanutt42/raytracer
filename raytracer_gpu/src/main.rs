use glam::Vec3;
use notify::{RecursiveMode, Watcher};
use raytracer::{
	create_10_metallic_scene, create_glass_scene, create_sample_scene, create_simple_scene,
	create_wallpaper_scene,
};
use raytracer_gpu::{Camera, Cube, Material, Renderer, Sphere};
use std::{
	collections::HashSet,
	path::PathBuf,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	time::Instant,
};
use winit::{
	application::ApplicationHandler,
	dpi::{LogicalSize, Size},
	event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent},
	event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
	keyboard::{KeyCode, ModifiersState, PhysicalKey},
	window::{Window, WindowId},
};

fn main() {
	pollster::block_on(run());
}

async fn run() {
	#[allow(clippy::type_complexity)]
	let (create_scene_fn, normal_sky_color): (
		fn() -> (Vec<raytracer::Sphere>, Vec<raytracer::Cube>),
		bool,
	) = match std::env::args().nth(1) {
		Some(ref arg) if arg == "simple" => (create_simple_scene, true),
		Some(ref arg) if arg == "glass" => (create_glass_scene, true),
		Some(ref arg) if arg == "metal" => (create_10_metallic_scene, true),
		Some(ref arg) if arg == "sample" => (create_sample_scene, true),
		Some(ref arg) if arg == "wallpaper" => (create_wallpaper_scene, false),
		_ => (create_simple_scene, true),
	};

	let shader_code_changed_flag = Arc::new(AtomicBool::new(false));
	let shader_code_directory = PathBuf::from("src/shaders");
	let _shader_code_file_watcher = if shader_code_directory.exists() {
		let shader_code_changed_flag_clone = shader_code_changed_flag.clone();
		let mut shader_code_file_watcher =
			notify::recommended_watcher(move |result| match result {
				Ok(notify::Event { kind, .. }) => {
					if matches!(kind, notify::EventKind::Modify(_)) {
						shader_code_changed_flag_clone.store(true, Ordering::Relaxed)
					}
				}
				Err(e) => eprintln!("failed to listen to shader code file changes: {e}"),
			})
			.expect("failed to create shader code file watcher");
		shader_code_file_watcher
			.watch(&shader_code_directory, RecursiveMode::Recursive)
			.expect("failed to watch shader code files");
		Some(shader_code_file_watcher)
	} else {
		println!("Warning: Shader code will not be hot reloaded");
		None
	};

	let event_loop = EventLoop::new().expect("failed to create event loop");
	event_loop.set_control_flow(ControlFlow::Poll);

	let camera = Camera::new(Vec3::new(0.0, 0.0, 0.0), -90.0, 0.0, 90.0, 0.1, 1000.0);

	let mut app = App {
		create_scene_fn,
		normal_sky_color,
		camera,
		renderer: None,
		window: None,
		last_redraw: Instant::now(),
		pressed_key_codes: HashSet::new(),
		pressed_mouse_buttons: HashSet::new(),
		shift_down: false,
		ctrl_down: false,
		mouse_delta: (0.0, 0.0),
		shader_code_changed_flag,
	};

	event_loop.run_app(&mut app).expect("event loop failed");
}

struct App {
	#[allow(clippy::type_complexity)]
	create_scene_fn: fn() -> (Vec<raytracer::Sphere>, Vec<raytracer::Cube>),
	normal_sky_color: bool,
	camera: Camera,
	renderer: Option<Renderer>,
	window: Option<Arc<Window>>,
	last_redraw: Instant,
	pressed_key_codes: HashSet<KeyCode>,
	pressed_mouse_buttons: HashSet<MouseButton>,
	shift_down: bool,
	ctrl_down: bool,
	mouse_delta: (f64, f64),
	shader_code_changed_flag: Arc<AtomicBool>,
}

impl ApplicationHandler for App {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.window.is_some() {
			return;
		}

		let window = Arc::new(
			event_loop
				.create_window(
					Window::default_attributes()
						.with_title("WGPU Compute Raytracer")
						.with_inner_size(Size::Logical(LogicalSize::new(100.0, 100.0))),
				)
				.expect("failed to create window"),
		);

		let (spheres, cubes) = (self.create_scene_fn)();
		let spheres = convert_spheres(spheres);
		let cubes = convert_cubes(cubes);
		let renderer = pollster::block_on(Renderer::new(
			window.clone(),
			&spheres,
			&cubes,
			self.camera,
			self.normal_sky_color,
		));

		self.window = Some(window);
		self.renderer = Some(renderer);
	}

	fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
		match event {
			WindowEvent::CloseRequested => {
				event_loop.exit();
			}
			WindowEvent::Resized(new_size) => {
				if let Some(renderer) = &mut self.renderer {
					renderer.resize(new_size);
				}
				if let Some(window) = &self.window {
					window.request_redraw();
				}
			}
			WindowEvent::KeyboardInput { event, .. } => {
				let PhysicalKey::Code(key_code) = event.physical_key else {
					return;
				};
				match event.state {
					ElementState::Pressed => {
						if key_code == KeyCode::Escape {
							event_loop.exit();
						}
						self.pressed_key_codes.insert(key_code);
					}
					ElementState::Released => {
						self.pressed_key_codes.remove(&key_code);
					}
				}
			}
			WindowEvent::ModifiersChanged(modifiers) => {
				self.ctrl_down = modifiers.state().contains(ModifiersState::CONTROL);
				self.shift_down = modifiers.state().contains(ModifiersState::SHIFT);
			}
			WindowEvent::MouseInput { state, button, .. } => match state {
				ElementState::Pressed => {
					self.pressed_mouse_buttons.insert(button);
				}
				ElementState::Released => {
					self.pressed_mouse_buttons.remove(&button);
				}
			},
			WindowEvent::RedrawRequested => {
				let Some(renderer) = &mut self.renderer else {
					return;
				};
				let Some(window) = &self.window else {
					return;
				};

				let delta_time = Instant::now() - self.last_redraw;
				window.set_title(
					format!(
						"WGPU Compute Raytracer: {:.0} fps ({delta_time:.2?}) frame: {}",
						1.0 / delta_time.as_secs_f64(),
						renderer.render_info.frame_counter
					)
					.as_str(),
				);
				self.last_redraw = Instant::now();

				if self.shader_code_changed_flag.load(Ordering::Relaxed) {
					println!("HOT RELOADING SHADERS...");
					match renderer.hot_reload_shaders_from_files(
						"src/shaders/compute.wgsl",
						"src/shaders/render.wgsl",
					) {
						Ok(_) => println!("HOT RELOADING SHADERS FINISHED!"),
						Err(e) => eprintln!("COULD NOT HOT RELOAD SHADERS:\n{e}"),
					}
					self.shader_code_changed_flag
						.store(false, Ordering::Relaxed);
				}

				camera_input_controller(
					&mut self.camera,
					&self.pressed_key_codes,
					&self.pressed_mouse_buttons,
					&mut self.mouse_delta,
					self.ctrl_down,
					self.shift_down,
					delta_time.as_secs_f32(),
				);

				renderer.update_camera(self.camera);
				renderer.update();
			}
			_ => {}
		}
	}

	fn device_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		_device_id: DeviceId,
		event: DeviceEvent,
	) {
		if let DeviceEvent::MouseMotion { delta } = event {
			self.mouse_delta.0 += delta.0;
			self.mouse_delta.1 += delta.1;
		}
	}

	fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
		if let Some(window) = &self.window {
			window.request_redraw();
		}
	}
}

fn camera_input_controller(
	camera: &mut Camera,
	pressed_key_codes: &HashSet<KeyCode>,
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

	let mut amount_forward = if input_is_key_down(KeyCode::KeyW) {
		1.0
	} else {
		0.0
	};
	if input_is_key_down(KeyCode::KeyS) {
		amount_forward -= 1.0;
	}
	move_dir += camera.get_forward() * amount_forward;

	let mut amount_right = if input_is_key_down(KeyCode::KeyA) {
		-1.0
	} else {
		0.0
	};
	if input_is_key_down(KeyCode::KeyD) {
		amount_right += 1.0;
	}
	move_dir += camera.get_right() * amount_right;

	let mut amount_up = if input_is_key_down(KeyCode::KeyE) {
		1.0
	} else {
		0.0
	};
	if input_is_key_down(KeyCode::KeyQ) {
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

fn convert_raytracer_vec3(vec3: raytracer::Vec3) -> Vec3 {
	Vec3 {
		x: vec3.x as f32,
		y: vec3.y as f32,
		z: vec3.z as f32,
	}
}

fn get_albedo(material: &raytracer::Material) -> Vec3 {
	let albedo = match material {
		raytracer::Material::Metal { albedo, .. } => *albedo,
		raytracer::Material::Lambertain { albedo, .. } => *albedo,
		raytracer::Material::Dielectric { .. } => raytracer::Vec3::one(),
	};
	convert_raytracer_vec3(albedo)
}

fn extract_material(material: &raytracer::Material) -> Material {
	match material {
		raytracer::Material::Dielectric { ir } => Material::Dielectric { ir: *ir as f32 },
		raytracer::Material::Lambertain { emission, .. } => Material::Lambertain {
			emission: *emission as f32,
		},
		raytracer::Material::Metal { fuzz, .. } => Material::Metalic { fuzz: *fuzz as f32 },
	}
}

fn convert_cubes(cubes: Vec<raytracer::Cube>) -> Vec<Cube> {
	cubes
		.into_iter()
		.map(|cube| {
			Cube::new(
				convert_raytracer_vec3(cube.center),
				convert_raytracer_vec3(cube.half_extend),
				get_albedo(&cube.material),
				extract_material(&cube.material),
			)
		})
		.collect()
}

fn convert_spheres(spheres: Vec<raytracer::Sphere>) -> Vec<Sphere> {
	spheres
		.into_iter()
		.map(|sphere| {
			Sphere::new(
				convert_raytracer_vec3(sphere.center),
				sphere.radius as f32,
				get_albedo(&sphere.material),
				extract_material(&sphere.material),
			)
		})
		.collect()
}
