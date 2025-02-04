use std::f32;

mod uniform_buffer;

pub use uniform_buffer::UniformBuffer;

mod camera;
pub use camera::{Camera, CameraUniformBuffer};

mod renderer;
pub use renderer::Renderer;

mod sphere;
pub use sphere::{
	create_10_metallic_scene, create_sample_scene, create_simple_scene, Material, Sphere,
	SPHERE_BUFFER_BIND_GROUP,
};

#[inline(always)]
pub const fn to_degrees(radians: f32) -> f32 {
	radians * (180.0 / f32::consts::PI)
}

#[inline(always)]
pub const fn to_radians(degrees: f32) -> f32 {
	degrees * (f32::consts::PI / 180.0)
}
