use glam::Vec3;

pub const SPHERE_BUFFER_BIND_GROUP: u32 = 0;

pub enum Material {
	Lambertain { emission: f32 },
	Metalic { fuzz: f32 },
}

impl Material {
	fn get_type(&self) -> MaterialType {
		match self {
			Self::Lambertain { .. } => MaterialType::LAMBERTAIN,
			Self::Metalic { .. } => MaterialType::METALIC,
		}
	}

	fn get_param1(self) -> f32 {
		match self {
			Self::Lambertain { emission } => emission,
			Self::Metalic { fuzz } => fuzz,
		}
	}
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MaterialType {
	material_type: u32,
}
impl MaterialType {
	const LAMBERTAIN: Self = Self { material_type: 0 };
	const METALIC: Self = Self { material_type: 1 };
}

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
