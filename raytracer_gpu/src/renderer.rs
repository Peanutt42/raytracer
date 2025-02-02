use std::borrow::Cow;

use wgpu::{
	util::DeviceExt, Adapter, BindGroup, BindGroupLayout, Buffer, CommandEncoder, ComputePipeline,
	Device, Instance, Queue, RenderPipeline, Sampler, ShaderStages, Surface, SurfaceConfiguration,
	Texture, TextureView, TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{Camera, CameraUniformBuffer, Sphere, SPHERE_BUFFER_BIND_GROUP};

pub struct Renderer {
	#[allow(unused)]
	instance: Instance,
	surface: Surface,
	#[allow(unused)]
	adapter: Adapter,
	device: Device,
	queue: Queue,
	config: SurfaceConfiguration,
	sampler: Sampler,

	camera: Camera,
	camera_uniform_buffer: CameraUniformBuffer,

	compute_pipeline: ComputePipeline,
	storage_texture: wgpu::Texture,
	#[allow(unused)]
	storage_view: wgpu::TextureView,
	#[allow(unused)]
	storage_buffer: Buffer,
	storage_bind_group: wgpu::BindGroup,
	#[allow(unused)]
	storage_bind_group_layout: BindGroupLayout,

	render_pipeline: RenderPipeline,
	render_texture: wgpu::Texture,
	#[allow(unused)]
	render_view: wgpu::TextureView,
	render_bind_group: wgpu::BindGroup,
	#[allow(unused)]
	render_bind_group_layout: BindGroupLayout,
}

impl Renderer {
	pub async fn new(window: &Window, spheres: &[Sphere], camera: Camera) -> Self {
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
		let surface = unsafe { instance.create_surface(&window) }.unwrap();

		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			})
			.await
			.unwrap();

		let (device, queue) = adapter
			.request_device(&wgpu::DeviceDescriptor::default(), None)
			.await
			.unwrap();

		let mut config = surface
			.get_default_config(
				&adapter,
				window.inner_size().width,
				window.inner_size().height,
			)
			.unwrap();
		config.format = wgpu::TextureFormat::Rgba8Unorm;
		surface.configure(&device, &config);

		let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

		let camera_uniform_buffer = CameraUniformBuffer::new(
			Some("Camera Uniform"),
			camera,
			ShaderStages::COMPUTE,
			&device,
		);

		let storage_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("Storage Bind Group Layout"),
				entries: &[
					// Sphere buffer
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::COMPUTE,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Storage { read_only: true },
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						count: None,
					},
					// Storage texture (compute write)
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::COMPUTE,
						ty: wgpu::BindingType::StorageTexture {
							access: wgpu::StorageTextureAccess::WriteOnly,
							format: wgpu::TextureFormat::Rgba8Unorm,
							view_dimension: wgpu::TextureViewDimension::D2,
						},
						count: None,
					},
				],
			});

		let render_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("Render Bind Group Layout"),
				entries: &[
					// Sampled texture (render read)
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
							view_dimension: wgpu::TextureViewDimension::D2,
							multisampled: false,
						},
						count: None,
					},
					// Sampler (render)
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None,
					},
				],
			});

		let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Sphere Buffer"),
			contents: bytemuck::cast_slice(spheres),
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
		});

		// START: window size dependant
		let (storage_texture, storage_view) = Self::create_storage_texture(&device, &config);

		let (render_texture, render_view) = Self::create_render_texture(&device, &config);

		let storage_bind_group = Self::create_storage_bind_group(
			&device,
			&storage_bind_group_layout,
			&storage_buffer,
			&storage_view,
		);

		let render_bind_group = Self::create_render_bind_group(
			&device,
			&render_bind_group_layout,
			&render_view,
			&sampler,
		);
		// END: window size dependant

		let compute_pipeline = {
			let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
				label: Some("Raytrace Compute Shader Module"),
				source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("compute.wgsl"))),
			});

			let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Raytrace Compute Pipeline Layout"),
				bind_group_layouts: &[
					&storage_bind_group_layout,
					camera_uniform_buffer.get_binding(),
				],
				push_constant_ranges: &[],
			});

			device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
				label: Some("Raytrace Compute Pipeline"),
				layout: Some(&pipeline_layout),
				module: &shader,
				entry_point: "main",
			})
		};

		let render_pipeline = {
			let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
				label: None,
				source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("render.wgsl"))),
			});

			let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: None,
				bind_group_layouts: &[&render_bind_group_layout],
				push_constant_ranges: &[],
			});

			device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: None,
				layout: Some(&pipeline_layout),
				vertex: wgpu::VertexState {
					module: &shader,
					entry_point: "vs_main",
					buffers: &[],
				},
				fragment: Some(wgpu::FragmentState {
					module: &shader,
					entry_point: "fs_main",
					targets: &[Some(config.format.into())],
				}),
				primitive: wgpu::PrimitiveState::default(),
				depth_stencil: None,
				multisample: wgpu::MultisampleState::default(),
				multiview: None,
			})
		};

		Self {
			instance,
			surface,
			adapter,
			device,
			queue,
			config,
			sampler,
			camera,
			camera_uniform_buffer,
			compute_pipeline,
			storage_texture,
			storage_view,
			storage_buffer,
			storage_bind_group,
			storage_bind_group_layout,
			render_pipeline,
			render_texture,
			render_view,
			render_bind_group,
			render_bind_group_layout,
		}
	}

	fn create_storage_texture(
		device: &Device,
		config: &SurfaceConfiguration,
	) -> (Texture, TextureView) {
		let texture = device.create_texture(&wgpu::TextureDescriptor {
			label: Some("Storage Texture"),
			size: wgpu::Extent3d {
				width: config.width,
				height: config.height,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8Unorm,
			usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
			view_formats: &[],
		});
		let texture_view = texture.create_view(&TextureViewDescriptor::default());
		(texture, texture_view)
	}

	fn create_storage_bind_group(
		device: &Device,
		storage_bind_group_layout: &BindGroupLayout,
		storage_buffer: &Buffer,
		storage_view: &TextureView,
	) -> BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: storage_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: storage_buffer.as_entire_binding(),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::TextureView(storage_view),
				},
			],
			label: Some("Storage Bind Group"),
		})
	}

	fn create_render_bind_group(
		device: &Device,
		render_bind_group_layout: &BindGroupLayout,
		render_view: &TextureView,
		sampler: &Sampler,
	) -> BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: render_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(render_view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(sampler),
				},
			],
			label: Some("Render Bind Group"),
		})
	}

	fn create_render_texture(
		device: &Device,
		config: &SurfaceConfiguration,
	) -> (Texture, TextureView) {
		let texture = device.create_texture(&wgpu::TextureDescriptor {
			label: Some("Render Texture"),
			size: wgpu::Extent3d {
				width: config.width,
				height: config.height,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8Unorm,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			view_formats: &[],
		});
		let texture_view = texture.create_view(&TextureViewDescriptor::default());
		(texture, texture_view)
	}

	pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
		self.config.width = new_size.width.max(1);
		self.config.height = new_size.height.max(1);
		self.surface.configure(&self.device, &self.config);

		let (storage_texture, storage_view) =
			Self::create_storage_texture(&self.device, &self.config);
		self.storage_texture = storage_texture;
		self.storage_view = storage_view;

		let (render_texture, render_view) = Self::create_render_texture(&self.device, &self.config);
		self.render_texture = render_texture;
		self.render_view = render_view;

		self.storage_bind_group = Self::create_storage_bind_group(
			&self.device,
			&self.storage_bind_group_layout,
			&self.storage_buffer,
			&self.storage_view,
		);

		self.render_bind_group = Self::create_render_bind_group(
			&self.device,
			&self.render_bind_group_layout,
			&self.render_view,
			&self.sampler,
		);
	}

	pub fn update(&self) {
		let output = self.surface.get_current_texture().unwrap();
		let view = output
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		self.camera_uniform_buffer.update(self.camera, &self.queue);

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

		self.compute_pass(&mut encoder);

		self.render_pass(&mut encoder, &view);

		self.queue.submit(Some(encoder.finish()));
		output.present();
	}

	pub fn update_camera(&mut self, new_camera: Camera) {
		self.camera = new_camera;
	}

	fn compute_pass(&self, encoder: &mut CommandEncoder) {
		{
			let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
			pass.set_pipeline(&self.compute_pipeline);
			pass.set_bind_group(SPHERE_BUFFER_BIND_GROUP, &self.storage_bind_group, &[]);
			self.camera_uniform_buffer.bind_compute(&mut pass);
			let workgroups_x = (self.config.width + 15) / 16;
			let workgroups_y = (self.config.height + 15) / 16;
			pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
		}

		// Copy from storage texture to render texture
		encoder.copy_texture_to_texture(
			wgpu::ImageCopyTexture {
				texture: &self.storage_texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			wgpu::ImageCopyTexture {
				texture: &self.render_texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			wgpu::Extent3d {
				width: self.config.width,
				height: self.config.height,
				depth_or_array_layers: 1,
			},
		);
	}

	fn render_pass(&self, encoder: &mut CommandEncoder, view: &TextureView) {
		let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: None,
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
					store: true,
				},
			})],
			depth_stencil_attachment: None,
		});
		pass.set_pipeline(&self.render_pipeline);
		pass.set_bind_group(0, &self.render_bind_group, &[]);
		pass.draw(0..3, 0..1);
	}
}
