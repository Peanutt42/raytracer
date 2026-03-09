use crossterm::{
	cursor, queue,
	style::{Color, Print, SetBackgroundColor, SetForegroundColor},
	terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate},
};
use std::io::{self, Stdout, Write};

#[derive(Clone, Copy)]
pub struct Cell {
	pub ch: char,
	pub fg: Color,
	pub bg: Color,
}

impl Cell {
	pub const BLANK: Self = Self {
		ch: ' ',
		fg: Color::White,
		bg: Color::Black,
	};
}

pub struct FrameBuffer {
	pub width: usize,
	pub height: usize,
	pub cells: Vec<Cell>,
}

impl FrameBuffer {
	pub fn new(width: usize, height: usize) -> Self {
		Self {
			width,
			height,
			cells: vec![Cell::BLANK; width * height],
		}
	}

	pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
		if x < self.width && y < self.height {
			self.cells[y * self.width + x] = cell;
		}
	}

	pub fn clear(&mut self, bg: Color) {
		for cell in &mut self.cells {
			*cell = Cell {
				ch: ' ',
				fg: Color::White,
				bg,
			};
		}
	}

	pub fn flush(&self, stdout: &mut Stdout) -> io::Result<()> {
		queue!(stdout, BeginSynchronizedUpdate, cursor::MoveTo(0, 0))?;

		for row in self.cells.chunks(self.width) {
			for cell in row {
				queue!(
					stdout,
					SetBackgroundColor(cell.bg),
					SetForegroundColor(cell.fg),
					Print(cell.ch)
				)?;
			}
		}

		queue!(stdout, cursor::MoveTo(0, 0), EndSynchronizedUpdate)?;

		stdout.flush()
	}
}
