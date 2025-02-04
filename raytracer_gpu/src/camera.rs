use glam::{Mat4, Vec3};

use crate::{to_radians, UniformBuffer};

const CAMERA_UNIFORM_BIND_GROUP: u32 = 1;

pub type CameraUniformBuffer = UniformBuffer<CameraUniform, CAMERA_UNIFORM_BIND_GROUP>;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct CameraUniform {
	pub inverse_projection: Mat4,
	pub inverse_view: Mat4,
	pub position: Vec3,
	_position_padding: f32,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Camera {
	pub position: Vec3,
	pub yaw: f32,
	pub pitch: f32,
	/// degrees
	pub fov: f32,
	pub z_near: f32,
	pub z_far: f32,
}

impl Camera {
	pub const WORLD_UP: Vec3 = Vec3 {
		x: 0.0,
		y: 1.0,
		z: 0.0,
	};

	pub fn new(position: Vec3, yaw: f32, pitch: f32, fov: f32, z_near: f32, z_far: f32) -> Self {
		Self {
			position,
			yaw,
			pitch,
			fov,
			z_near,
			z_far,
		}
	}

	pub fn get_forward(&self) -> Vec3 {
		let pitch_radians = to_radians(self.pitch);
		let yaw_radians = to_radians(self.yaw);
		Vec3::new(
			yaw_radians.cos() * pitch_radians.cos(),
			pitch_radians.sin(),
			yaw_radians.sin() * pitch_radians.cos(),
		)
		.normalize()
	}

	pub fn get_right(&self) -> Vec3 {
		self.get_forward().cross(Self::WORLD_UP).normalize()
	}

	pub fn get_up(&self) -> Vec3 {
		self.get_right().cross(self.get_forward()).normalize()
	}

	pub fn get_uniform(&self, width: f32, height: f32) -> CameraUniform {
		let mut aspect_ratio = width / height;
		if !aspect_ratio.is_normal() {
			aspect_ratio = 16.0 / 9.0;
		}

		CameraUniform {
			inverse_projection: Mat4::perspective_rh(
				to_radians(self.fov),
				aspect_ratio,
				self.z_near,
				self.z_far,
			)
			.inverse(),
			inverse_view: Mat4::look_to_rh(self.position, self.get_forward(), self.get_up())
				.inverse(),
			position: self.position,
			_position_padding: 0.0,
		}
	}
}
