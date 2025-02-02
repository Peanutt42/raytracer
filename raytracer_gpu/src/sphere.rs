pub const SPHERE_BUFFER_BIND_GROUP: u32 = 0;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
	pub position: [f32; 3],
	pub emission: f32,
	pub color: [f32; 3],
	pub radius: f32,
}
