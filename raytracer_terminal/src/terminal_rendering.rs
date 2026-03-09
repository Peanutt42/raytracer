use crate::{Cell, FrameBuffer};
use raytracer::{Scalar, Vec3};
use std::time::Duration;

pub fn draw_to_terminal(
	framebuffer: &mut FrameBuffer,
	accum_image: &[Vec3],
	frame_counter: usize,
	delta_time: Duration,
) {
	for (i, cell) in framebuffer.cells.iter_mut().enumerate() {
		let final_color = accum_image[i] / frame_counter as Scalar;
		*cell = color_to_cell(final_color);
	}

	let fps = 1.0 / delta_time.as_secs_f32();
	let info = format!(
		"FPS: {} (Controls: WASD to move, ↑↓←→ arrows to look, Esc or Ctrl+c to quit)",
		fps.round()
	);
	for (i, c) in info.chars().enumerate() {
		let cell = Cell {
			ch: c,
			..Cell::BLANK
		};
		framebuffer.set(i.min(framebuffer.width), 0, cell);
	}
}

fn vec3_color_to_crossterm_color(color: Vec3) -> crossterm::style::Color {
	crossterm::style::Color::Rgb {
		r: (color.x * 255.0) as u8,
		g: (color.y * 255.0) as u8,
		b: (color.z * 255.0) as u8,
	}
}

// TODO: add edge detection pass to have bg be the color and char + fg be the edge
fn color_to_cell(color: Vec3) -> Cell {
	const RED_LUMINANCE: Scalar = 0.2126;
	const GREEN_LUMINANCE: Scalar = 0.7152;
	const BLUE_LUMINANCE: Scalar = 0.0722;

	// let chars = " `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@".chars().collect::<Vec<char>>();
	let chars = [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
	let luminance =
		(color.x * RED_LUMINANCE + color.y * GREEN_LUMINANCE + color.z * BLUE_LUMINANCE)
			.clamp(0.0, 1.0);
	let darkness = 1.0 - luminance;
	let ch = chars[(darkness * (chars.len() as f64 - 1.0)).round() as usize];

	let bg_color = color - Vec3::one() * 0.25;

	Cell {
		fg: vec3_color_to_crossterm_color(bg_color),
		bg: vec3_color_to_crossterm_color(color),
		ch,
	}
}
