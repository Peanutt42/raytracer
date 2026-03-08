use crossterm::{
	cursor,
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
	execute,
	terminal::{self, ClearType},
};
use rayon::prelude::*;
use raytracer::{
	BVH, Camera, Scalar, Vec3, combine_spheres_and_cubes, create_simple_scene, get_camera_rotation,
	render,
};
use raytracer_terminal::{Cell, FrameBuffer};
use std::{
	io,
	time::{Duration, Instant},
};

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
		terminal::Clear(ClearType::All)
	)?;

	let (mut width, mut height) = terminal::size()?;

	let yaw = -90.0;
	let pitch = 0.0;
	let camera = Camera::new(
		Vec3::new(0.0, 0.0, 0.0),
		get_camera_rotation(yaw, pitch),
		90.0,
		10.0,
		0.6,
		width as usize,
		height as usize,
	);

	let mut fb = FrameBuffer::new(width as usize, height as usize);
	let mut accum_image = vec![Vec3::zero(); fb.width * fb.height];
	let mut frame_counter = 1;
	let mut last_update = Instant::now();

	loop {
		let now = Instant::now();
		let delta_time = now - last_update;
		last_update = now;

		let (new_width, new_height) = terminal::size()?;

		if width != new_width || height != new_height {
			width = new_width;
			height = new_height;
			fb.width = new_width as usize;
			fb.height = new_height as usize;
			fb.cells.resize(fb.width * fb.height, Cell::BLANK);
			accum_image = vec![Vec3::zero(); fb.width * fb.height];
			frame_counter = 1;
		}

		render_scene(&mut accum_image, fb.width, &bvh, &camera);
		draw(&mut fb, &accum_image, frame_counter, delta_time);
		frame_counter += 1;

		fb.flush(&mut stdout)?;

		if event::poll(std::time::Duration::from_millis(0))?
			&& let Event::Key(key) = event::read()?
		{
			let ctrl_c = (matches!(key.code, KeyCode::Char('c'))
				&& key.modifiers.contains(KeyModifiers::CONTROL));
			let should_quit = (matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) || ctrl_c);

			if should_quit {
				break;
			}
		}
	}

	execute!(
		stdout,
		DisableMouseCapture,
		terminal::LeaveAlternateScreen,
		cursor::Show
	)?;
	terminal::disable_raw_mode()
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

fn draw(
	framebuffer: &mut FrameBuffer,
	accum_image: &[Vec3],
	frame_counter: usize,
	delta_time: Duration,
) {
	for (i, cell) in framebuffer.cells.iter_mut().enumerate() {
		let final_color_vec3 = accum_image[i] / frame_counter as Scalar;
		cell.bg = crossterm::style::Color::Rgb {
			r: (final_color_vec3.x * 255.0) as u8,
			g: (final_color_vec3.y * 255.0) as u8,
			b: (final_color_vec3.z * 255.0) as u8,
		};
		cell.ch = ' ';
	}

	let fps = 1.0 / delta_time.as_secs_f32();
	let info = format!("FPS: {}", fps.round());
	for (i, c) in info.chars().enumerate() {
		let cell = Cell {
			ch: c,
			..Cell::BLANK
		};
		framebuffer.set(i.min(framebuffer.width), 0, cell);
	}
}
