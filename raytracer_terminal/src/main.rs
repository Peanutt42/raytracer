use crossterm::{
	cursor,
	event::{
		self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers,
		KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
	},
	execute,
	terminal::{self, ClearType},
};
use rayon::prelude::*;
use raytracer::{
	BVH, Camera, Scalar, Vec3, combine_spheres_and_cubes, create_simple_scene, get_camera_rotation,
	render,
};
use raytracer_terminal::{CameraController, Cell, FrameBuffer, draw_to_terminal};
use std::{io, time::Instant};

fn main() -> io::Result<()> {
	let (spheres, cubes) = create_simple_scene();
	let bvh = BVH::new(combine_spheres_and_cubes(spheres, cubes)).unwrap();

	let mut stdout = io::stdout();

	terminal::enable_raw_mode()?;
	execute!(
		stdout,
		terminal::EnterAlternateScreen,
		cursor::Hide,
		EnableMouseCapture,
		PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES),
		terminal::Clear(ClearType::All)
	)?;

	let (mut width, mut height) = terminal::size()?;

	let mut yaw = -90.0;
	let mut pitch = 0.0;
	let mut camera = Camera::new(
		Vec3::new(0.0, 0.0, 0.0),
		get_camera_rotation(yaw, pitch),
		Camera::DEFAULT_FOV,
		Camera::DEFAULT_FOCUS_DIST,
		Camera::DEFAULT_DEFOCUS_ANGLE,
		width as usize,
		height as usize,
	);
	let mut old_camera = camera.clone();

	let mut fb = FrameBuffer::new(width as usize, height as usize);
	let mut accum_image = vec![Vec3::zero(); fb.width * fb.height];
	let mut frame_counter = 1;
	let mut last_update = Instant::now();
	let mut camera_controller = CameraController::default();

	loop {
		let now = Instant::now();
		let delta_time = now - last_update;
		last_update = now;

		let (new_width, new_height) = terminal::size()?;

		camera_controller.update(
			&mut camera,
			&mut yaw,
			&mut pitch,
			new_width as usize,
			new_height as usize,
			delta_time,
		);

		if width != new_width || height != new_height || camera != old_camera {
			old_camera = camera.clone();

			reset(
				new_width,
				new_height,
				&mut width,
				&mut height,
				&mut fb,
				&mut accum_image,
				&mut frame_counter,
			);
		}

		render_scene(&mut accum_image, fb.width, &bvh, &camera);
		draw_to_terminal(&mut fb, &accum_image, frame_counter, delta_time);
		frame_counter += 1;

		fb.flush(&mut stdout)?;

		if event::poll(std::time::Duration::from_millis(0))?
			&& let Event::Key(key) = event::read()?
		{
			let ctrl_c = (matches!(key.code, KeyCode::Char('c'))
				&& key.modifiers.contains(KeyModifiers::CONTROL));
			let should_quit = (matches!(key.code, KeyCode::Esc) || ctrl_c);

			if should_quit {
				break;
			}

			camera_controller.on_key_event(key);
		}
	}

	execute!(
		stdout,
		PopKeyboardEnhancementFlags,
		DisableMouseCapture,
		terminal::LeaveAlternateScreen,
		cursor::Show
	)?;
	terminal::disable_raw_mode()
}

fn reset(
	new_width: u16,
	new_height: u16,
	width: &mut u16,
	height: &mut u16,
	framebuffer: &mut FrameBuffer,
	accum_image: &mut Vec<Vec3>,
	frame_counter: &mut usize,
) {
	*width = new_width;
	*height = new_height;
	framebuffer.width = new_width as usize;
	framebuffer.height = new_height as usize;
	framebuffer
		.cells
		.resize(framebuffer.width * framebuffer.height, Cell::BLANK);
	*accum_image = vec![Vec3::zero(); framebuffer.width * framebuffer.height];
	*frame_counter = 1;
}

fn render_scene(accum_image: &mut [Vec3], width: usize, bvh: &BVH, camera: &Camera) {
	let max_depth = 4;

	accum_image
		.par_chunks_exact_mut(width)
		.enumerate()
		.for_each(|(y, row)| {
			let mut rand = rand::rng();
			for (x, output_color) in row.iter_mut().enumerate() {
				*output_color = *output_color
					+ render(x as Scalar, y as Scalar, camera, bvh, max_depth, &mut rand)
						.linear_to_gamma();
			}
		});
}
