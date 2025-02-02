use glam::Vec3;

use crate::{to_radians, UniformBuffer};

const CAMERA_UNIFORM_BIND_GROUP: u32 = 1;

pub type CameraUniformBuffer = UniformBuffer<Camera, CAMERA_UNIFORM_BIND_GROUP>;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct Camera {
	pub position: Vec3,
	_padding: f32,
}

impl Camera {
	const WORLD_UP: Vec3 = Vec3 {
		x: 0.0,
		y: 1.0,
		z: 0.0,
	};

	pub fn new(position: Vec3) -> Self {
		Self {
			position,
			_padding: 0.0,
		}
	}

	pub fn get_forward(&self) -> Vec3 {
		let yaw_rad = to_radians(-90.0);
		let pitch_rad: f32 = 0.0;
		let yaw_sin = yaw_rad.sin();
		let yaw_cos = yaw_rad.cos();
		let pitch_sin = pitch_rad.sin();
		let pitch_cos = pitch_rad.cos();

		Vec3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos)
	}

	pub fn get_right(&self) -> Vec3 {
		self.get_forward().cross(Self::WORLD_UP).normalize()
	}

	pub fn get_up(&self) -> Vec3 {
		self.get_right().cross(self.get_forward()).normalize()
	}
}
