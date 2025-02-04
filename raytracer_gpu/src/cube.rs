use glam::Vec3;

use crate::{Material, MaterialType};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Cube {
	position: Vec3,
	_position_padding: f32,
	half_extend: Vec3,
	_size_padding: f32,
	albedo: Vec3,
	material_type: MaterialType,
	material_param1: f32,
	_material_param1_padding: Vec3,
}

impl Cube {
	pub fn new(position: Vec3, half_extend: Vec3, albedo: Vec3, material: Material) -> Self {
		Self {
			position,
			_position_padding: 0.0,
			half_extend,
			_size_padding: 0.0,
			albedo,
			material_type: material.get_type(),
			material_param1: material.get_param1(),
			_material_param1_padding: Vec3::ZERO,
		}
	}
}
