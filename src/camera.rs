use crate::math::{Vec3, Ray};

pub struct Camera {
	pub origin: Vec3,
	width: usize,
	height: usize,
	pixel_delta_x: f64,
	pixel_delta_y: f64,
}

fn pixel_sample_square() -> (f64, f64) {
	(-0.5 + rand::random::<f64>(), -0.5 + rand::random::<f64>())
}

impl Camera {
	pub fn new(origin: Vec3, width: usize, height: usize) -> Self {
		let viewport_height = 2.0;
		let viewport_width = viewport_height * (width as f64 / height as f64);
		Camera {
			origin: origin,
			width: width,
			height: height,
			pixel_delta_x: viewport_width / width as f64,
			pixel_delta_y: viewport_height / height as f64,
		}
	}

	pub fn get_ray(&self, x: usize, y: usize) -> Ray {
		let pixel_dir = Vec3::new(
			(2.0 * x as f64 / self.width as f64) - 1.0,
			1.0 - (2.0 * y as f64 / self.height as f64),
			-1.0,
		).normalize();
		let pixel_center = self.origin + pixel_dir;
		let sample_offset = pixel_sample_square();
		let offset = Vec3::new(sample_offset.0 * self.pixel_delta_x, sample_offset.1 * self.pixel_delta_y, 0.0);
		let pixel_sample = pixel_center + offset;
		Ray::new(self.origin, pixel_sample - self.origin)
	}
}