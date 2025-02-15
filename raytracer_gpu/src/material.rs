pub const COMPUTE_BIND_GROUP: u32 = 0;

#[derive(Debug, Clone, Copy)]
pub enum Material {
	Lambertain { emission: f32 },
	Metalic { fuzz: f32 },
	Dielectric { ir: f32 },
}

impl Material {
	pub fn get_type(&self) -> MaterialType {
		match self {
			Self::Lambertain { .. } => MaterialType::LAMBERTAIN,
			Self::Metalic { .. } => MaterialType::METALIC,
			Self::Dielectric { .. } => MaterialType::DIELECTRIC,
		}
	}

	pub fn get_param1(self) -> f32 {
		match self {
			Self::Lambertain { emission } => emission,
			Self::Metalic { fuzz } => fuzz,
			Self::Dielectric { ir } => ir,
		}
	}
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialType {
	material_type: u32,
}
impl MaterialType {
	pub const LAMBERTAIN: Self = Self { material_type: 0 };
	pub const METALIC: Self = Self { material_type: 1 };
	pub const DIELECTRIC: Self = Self { material_type: 2 };
}
