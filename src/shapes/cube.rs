use crate::{Hittable, Bounded, Renderable, Vec3, Material, AABB, Ray};

#[derive(Clone, Copy, Debug)]
pub struct Cube {
	pub center: Vec3,
	pub material: Material,
	half_extend: Vec3,
	aabb: AABB,
}

impl Cube {
	pub fn new(center: Vec3, half_extend: Vec3, material: Material) -> Self {
		Cube {
			center,
			half_extend,
			material,
			aabb: AABB::new(
				center - half_extend,
				center + half_extend
			)
		}
	}
}

impl Hittable for Cube {
	fn hit(&self, ray: &Ray) -> Option<f64> {
		let origin = ray.origin - self.center;

		let m = 1.0 / ray.dir;
		let n: Vec3 = m * origin;
		let k: Vec3 = m.abs() * self.half_extend;

		let t1: Vec3 = -n - k;
		let t2: Vec3 = -n + k;

		let t_n: f64 = t1.x.max(t1.y).max(t1.z);
		let t_f: f64 = t2.x.min(t2.y).min(t2.z);

		if t_n > t_f || t_f <= 0. || t_n <= 0. {
			None
		} else {
			Some(t_n)
		}
	}
}

impl Bounded for Cube {
	fn get_aabb(&self) -> AABB {
		self.aabb
	}
}

impl Renderable for Cube {
	fn get_normal(&self, p: &Vec3, _ray: &Ray) -> Vec3 {
		let rel_p = *p - self.center;
		let maxc = rel_p.x.abs().max(rel_p.y.abs()).max(rel_p.z.abs());
		if maxc == rel_p.x.abs() {
			return Vec3::new(rel_p.x.signum(), 0.0, 0.0);
		}
		if maxc == rel_p.y.abs() {
			return Vec3::new(0.0, rel_p.y.signum(), 0.0);
		}
		Vec3::new(0.0, 0.0, rel_p.z.signum())
	}

	fn get_material(&self) -> Option<&Material> {
		Some(&self.material)
	}
}