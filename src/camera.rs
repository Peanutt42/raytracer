use crate::math::*;


pub struct Camera {
	pub origin: Vec3,
	pixel00_loc: Vec3,
	pixel_delta_x: Vec3,
	pixel_delta_y: Vec3,
	defocus_disk_x: Vec3,
	defocus_disk_y: Vec3,
	defocus_angle: f64,
}

impl Camera {
	pub fn new(origin: Vec3, fov: f64, look_at: &Vec3, vup: &Vec3, focus_dist: f64, defocus_angle: f64, width: usize, height: usize) -> Self {
		// viewport
		let theta = radians(fov);
		let h = f64::tan(theta / 2.0);
		let viewport_height = 2.0 * h * focus_dist;
		let viewport_width = viewport_height * (width as f64 / height as f64);

		let w = (origin - *look_at).normalize();
		let u = vup.cross(&w).normalize();
		let v = w.cross(&u);

		// Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = u * viewport_width;
        let viewport_v = (-v) * viewport_height;

		let pixel_delta_x = viewport_u / width as f64;
		let pixel_delta_y = viewport_v / height as f64;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left = origin - (w * focus_dist) - viewport_u/2.0 - viewport_v/2.0;
        let pixel00_loc = viewport_upper_left + (pixel_delta_x + pixel_delta_y) * 0.5;

        // Calculate the camera defocus disk basis vectors.
        let defocus_radius = focus_dist * f64::tan(radians(defocus_angle / 2.0));
        let defocus_disk_x = u * defocus_radius;
        let defocus_disk_y = v * defocus_radius;

		Camera {
			origin: origin,
			pixel00_loc: pixel00_loc,
			pixel_delta_x: pixel_delta_x,
			pixel_delta_y: pixel_delta_y,
			defocus_disk_x: defocus_disk_x,
			defocus_disk_y: defocus_disk_y,
			defocus_angle: defocus_angle
		}
	}

	fn pixel_sample_square(&self) -> Vec3 {
		((-0.5 * rand::random::<f64>()) * self.pixel_delta_x) + ((-0.5 * rand::random::<f64>()) * self.pixel_delta_y)
	}

	fn defocus_disk_sample(&self) -> Vec3 {
		let p = Vec3::random_in_unit_disk();
		self.origin + (p.x * self.defocus_disk_x) * (p.y * self.defocus_disk_y)
	}

	pub fn get_ray(&self, x: f64, y: f64) -> Ray {
		let pixel_center = self.pixel00_loc + (self.pixel_delta_x * x) + (self.pixel_delta_y * y);
		let pixel_sample = pixel_center + self.pixel_sample_square();
		let ray_origin: Vec3;
		if self.defocus_angle <= 0.0 {
			ray_origin = self.origin;
		} else {
			ray_origin = self.defocus_disk_sample();
		}
		Ray::new(ray_origin, (pixel_sample - ray_origin).normalize())
	}
}