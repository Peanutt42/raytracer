use std::marker::PhantomData;

use bytemuck::{Pod, Zeroable};
use wgpu::{
	util::{BufferInitDescriptor, DeviceExt},
	BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
	BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ComputePass,
	Device, Queue, RenderPass, ShaderStages,
};

// 'T': buffer content structure
// 'BG': Bind Group index
pub struct UniformBuffer<T: Pod + Zeroable, const BG: u32> {
	buffer: Buffer,
	bind_group: BindGroup,
	bind_group_layout: BindGroupLayout,
	phantom: PhantomData<T>,
}

impl<T: Pod + Zeroable, const BG: u32> UniformBuffer<T, BG> {
	pub fn new(label: Option<&str>, data: T, visibility: ShaderStages, device: &Device) -> Self {
		let buffer = device.create_buffer_init(&BufferInitDescriptor {
			label,
			contents: bytemuck::cast_slice(&[data]),
			usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
		});

		let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
			label: None,
			entries: &[BindGroupLayoutEntry {
				binding: 0,
				visibility,
				ty: BindingType::Buffer {
					ty: BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
		});

		let bind_group = device.create_bind_group(&BindGroupDescriptor {
			label: None,
			layout: &bind_group_layout,
			entries: &[BindGroupEntry {
				binding: 0,
				resource: buffer.as_entire_binding(),
			}],
		});

		Self {
			buffer,
			bind_group,
			bind_group_layout,
			phantom: PhantomData,
		}
	}

	pub fn get_binding(&self) -> &BindGroupLayout {
		&self.bind_group_layout
	}

	pub fn update(&self, data: T, queue: &Queue) {
		queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
	}

	pub fn bind_compute<'a>(&'a self, computepass: &'_ mut ComputePass<'a>) {
		computepass.set_bind_group(BG, &self.bind_group, &[]);
	}

	pub fn bind_render<'a>(&'a self, renderpass: &'_ mut RenderPass<'a>) {
		renderpass.set_bind_group(BG, &self.bind_group, &[]);
	}
}
