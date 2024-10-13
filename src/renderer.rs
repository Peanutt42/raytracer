use crate::{Camera, Ray, Scene, Vec3, BVH};

fn ray_color(ray: &Ray, bvh: &BVH, contribution: &mut Vec3, depth: i32, rand: &mut rand::prelude::ThreadRng) -> Vec3 {
	if depth <= 0 {
		return Vec3::zero();
	}

	if let Some(hit) = bvh.trace(ray) {
		if let Some(scattered) = hit.material.scatter(ray, &hit, rand) {
			*contribution = (*contribution) * scattered.attenuation;
			scattered.attenuation * ray_color(&scattered.scattered, bvh, contribution, depth - 1, rand) + hit.material.emission_color()
		}
		else {
			Vec3::zero()
		}
	}
	else {
		Scene::get_sky_color(ray.dir) * (*contribution)
	}
}

// output color is in linear color space -> convert to gamma with Vec3::linear_to_gamma
pub fn render(x: f64, y: f64, camera: &Camera, bvh: &BVH, max_depth: i32, rand: &mut rand::prelude::ThreadRng) -> Vec3 {
	let ray = camera.get_ray(x, y, rand);
	let mut contribution = Vec3::one();
	ray_color(&ray, bvh, &mut contribution, max_depth, rand)
}