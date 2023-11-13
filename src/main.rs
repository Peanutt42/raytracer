use indicatif::{ProgressStyle, ProgressIterator};
use image::{RgbImage, Rgb};

mod math;
use math::{Vec3, Ray};

mod scene;
use scene::{Sphere, Scene};

mod camera;
use camera::Camera;


fn vec3_to_rgb(v: &Vec3) -> Rgb<u8> {
	Rgb([(v.x * 256.0) as u8, (v.y * 256.0) as u8, (v.z * 256.0) as u8])
}

fn linear_to_gamma(linear: f64) -> f64 {
	linear.sqrt()
}

fn ray_color(ray: &Ray, scene: &Scene, depth: usize) -> Vec3 {
	if depth <= 0 {
		return Vec3::new(0.0, 0.0, 0.0);
	}
	
	if let Some(hit) = scene.trace(&ray) {
		let new_dir = hit.normal + Vec3::random_hemisphere(&hit.normal);
		return ray_color(&Ray::new(hit.point, new_dir), &scene, depth - 1) * 0.5;
	}
	let unit_dir = ray.dir.normalize();
	let a = 0.5 * (unit_dir.y + 1.0);
	Vec3::new(1.0, 1.0, 1.0) * (1.0-a) + Vec3::new(0.5, 0.7, 1.0) * a
}

fn main() {
	let width = 200;
	let height = 200;
	let samples_per_pixel = 1500;
	let max_depth = 100;

	let camera = Camera::new(Vec3::new(0.0, 0.0, 0.0), width, height);
	
	let mut scene = Scene::new();
	scene.spheres.push(Sphere::new(Vec3::new(0.0,0.0,-1.0), 0.5));
	scene.spheres.push(Sphere::new(Vec3::new(0.0,-100.5,-1.0), 100.0));


	let progress_bar_style = ProgressStyle::with_template("{elapsed} {percent}% {wide_bar:.green/white}").unwrap();

	let mut image = RgbImage::new(width as u32, height as u32);
	let inv_samples_per_pixel = 1.0 / samples_per_pixel as f64;
	for y in (0..height).progress_with_style(progress_bar_style).with_finish(indicatif::ProgressFinish::Abandon) {
		for x in 0..width {
			let mut final_color = Vec3::new(0.0, 0.0, 0.0);
			for _ in 0..samples_per_pixel {
				let ray = camera.get_ray(x as usize, y as usize);
				final_color = final_color + ray_color(&ray, &scene, max_depth);
			}
			final_color = final_color * inv_samples_per_pixel as f64;
			final_color = Vec3::new(linear_to_gamma(final_color.x), linear_to_gamma(final_color.y), linear_to_gamma(final_color.z));
			image.put_pixel(x as u32, y as u32, vec3_to_rgb(&final_color));
		}
	}

	if let Err(error) = image.save("output.png") {
		println!("\nFailed to save to output.png: {}", error.to_string());
	}
	else {
		println!("\nFinished");
	}

}
