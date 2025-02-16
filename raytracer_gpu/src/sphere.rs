use glam::Vec3;

use crate::{Material, MaterialType};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
	position: Vec3,
	radius: f32,
	albedo: Vec3,
	material_type: MaterialType,
	material_param1: f32,
	_padding: Vec3,
}

impl Sphere {
	pub fn new(position: Vec3, radius: f32, albedo: Vec3, material: Material) -> Self {
		Self {
			position,
			radius,
			albedo,
			material_type: material.get_type(),
			material_param1: material.get_param1(),
			_padding: Vec3::ZERO,
		}
	}
}
