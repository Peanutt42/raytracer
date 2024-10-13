use std::cmp::Ordering;

use crate::{Bounded, Object, Ray, RayHit, Renderable, Scene, AABB};

#[derive(Debug)]
pub enum BVHNode {
	Branch {
		left: Box<BVH>,
		right: Box<BVH>,
	},
	FewObjects(Scene),
}

#[derive(Debug)]
pub struct BVH {
	tree: BVHNode,
	aabb: AABB,
}

impl BVH {
	pub fn new(mut scene: Scene) -> Option<Self> {
		fn box_compare(axis: usize) -> impl FnMut(&Object, &Object) -> Ordering {
			move |a, b| {
				let a_aabb = a.get_aabb();
				let b_aabb = b.get_aabb();
				let ac = a_aabb.min[axis] + a_aabb.max[axis];
				let bc = b_aabb.min[axis] + b_aabb.max[axis];
				ac.partial_cmp(&bc).unwrap()
			}
		}

		fn axis_range(objects: &[Object], axis: usize) -> f64 {
			let (min, max) = objects.iter().fold((f64::MAX, f64::MIN), |(bmin, bmax), hit| {
				let aabb = hit.get_aabb();
				(bmin.min(aabb.min[axis]), bmax.max(aabb.max[axis]))
			});
			max - min
		}

		let mut axis_ranges: [(usize, f64); 3] = [
			(0, axis_range(&scene.objects, 0)),
			(1, axis_range(&scene.objects, 1)),
			(2, axis_range(&scene.objects, 2))
		];

		axis_ranges.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

		let axis = axis_ranges[0].0;

		scene.objects.sort_unstable_by(box_compare(axis));
		match scene.objects.len() {
			0 => None,
			1..6 => {
				let mut aabb = scene.objects[0].get_aabb();
				for i in 1..scene.objects.len() {
					aabb = AABB::surrounding(aabb, scene.objects[i].get_aabb());
				}
				Some(BVH {
					tree: BVHNode::FewObjects(scene),
					aabb,
				})
			},
			_ => {
				let right = BVH::new(Scene::new(
					scene.objects.drain(scene.objects.len() / 2..).collect()
				));
				let left = BVH::new(scene);
				if let (Some(left), Some(right)) = (left, right) {
					let aabb = AABB::surrounding(left.aabb, right.aabb);
					Some(BVH {
						tree: BVHNode::Branch {
							left: Box::new(left),
							right: Box::new(right)
						},
						aabb
					})
				}
				else {
					None
				}
			},
		}
	}

	fn hit(&self, ray: &Ray) -> Option<(f64, &Object)> {
		if self.aabb.hit(ray) {
			let is_distance_valid = |distance: f64| distance > 0.001;

			match &self.tree {
				BVHNode::FewObjects(scene) => /*object.hit(ray).and_then(|distance| {
					if is_distance_valid(distance) {
						Some((distance, object))
					}
					else {
						None
					}
				})*/
				scene.hit(ray),
				BVHNode::Branch { left, right } => {
					let left = left.hit(ray);
					let right = right.hit(ray);
					match (left, right) {
						(
							Some((left_distance, left_object)),
							Some((right_distance, right_object))
						) => {
							if is_distance_valid(left_distance) && left_distance < right_distance {
								Some((left_distance, left_object))
							}
							else if is_distance_valid(right_distance) {
								Some((right_distance, right_object))
							}
							else {
								None
							}
						},
						(Some((distance, object)), None) => if is_distance_valid(distance) {
							Some((distance, object))
						}
						else {
							None
						},
						(None, Some((distance, object))) => if is_distance_valid(distance) {
							Some((distance, object))
						}
						else {
							None
						},
						(None, None) => None,
					}
				}
			}
		}
		else {
			None
		}
	}

	pub fn trace(&self, ray: &Ray) -> Option<RayHit> {
		self.hit(ray)
			.and_then(|(distance, object)| {
				object.get_material().map(|material| {
					let p = ray.at(distance);
					let mut normal = object.get_normal(&p, ray);
					let front_face = ray.dir.dot(normal) < 0.0;
					if !front_face {
						normal = -normal;
					}
					RayHit::new(p, normal, material, front_face)
				})
			})
	}
}