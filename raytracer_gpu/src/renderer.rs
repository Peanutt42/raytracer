use std::{borrow::Cow, path::Path};
use wgpu::{
	util::DeviceExt, Adapter, BindGroup, BindGroupLayout, Buffer, CommandEncoder, ComputePipeline,
	Device, ErrorFilter, Features, Instance, Limits, Queue, RenderPipeline, Sampler, ShaderModule,
	ShaderStages, Surface, SurfaceConfiguration, Texture, TextureView, TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{Camera, CameraUniformBuffer, Cube, Sphere, UniformBuffer, COMPUTE_BIND_GROUP};

const FRAME_COUNTER_UNIFORM_BUFFER_BIND_GROUP: u32 = 1;

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

	pub frame_counter: u32,
	frame_counter_uniform_buffer: UniformBuffer<u32, FRAME_COUNTER_UNIFORM_BUFFER_BIND_GROUP>,

	compute_pipeline: ComputePipeline,
	compute_texture: wgpu::Texture,
	#[allow(unused)]
	compute_view: wgpu::TextureView,
	#[allow(unused)]
	sphere_buffer: Buffer,
	#[allow(unused)]
	cube_buffer: Buffer,
	compute_bind_group: wgpu::BindGroup,
	#[allow(unused)]
	compute_bind_group_layout: BindGroupLayout,

	render_pipeline: RenderPipeline,
	render_texture: wgpu::Texture,
	#[allow(unused)]
	render_view: wgpu::TextureView,
	render_bind_group: wgpu::BindGroup,
	#[allow(unused)]
	render_bind_group_layout: BindGroupLayout,
}

impl Renderer {
	pub async fn new(window: &Window, spheres: &[Sphere], cubes: &[Cube], camera: Camera) -> Self {
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
			.request_device(
				&wgpu::DeviceDescriptor {
					label: Some("Device"),
					features: Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
					limits: Limits::default(),
				},
				None,
			)
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
			camera.get_uniform(config.width as f32, config.height as f32),
			ShaderStages::COMPUTE,
			&device,
		);

		let frame_counter = 1;
		let frame_counter_uniform_buffer =
			UniformBuffer::<u32, FRAME_COUNTER_UNIFORM_BUFFER_BIND_GROUP>::new(
				Some("Render Uniform Buffer"),
				frame_counter,
				ShaderStages::FRAGMENT | ShaderStages::COMPUTE,
				&device,
			);

		let compute_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("Compute Bind Group Layout"),
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
					// Cube buffer
					wgpu::BindGroupLayoutEntry {
						binding: 1,
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
						binding: 2,
						visibility: wgpu::ShaderStages::COMPUTE,
						ty: wgpu::BindingType::StorageTexture {
							access: wgpu::StorageTextureAccess::ReadWrite,
							format: wgpu::TextureFormat::Rgba32Float,
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

		let sphere_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Compute Sphere Buffer"),
			contents: bytemuck::cast_slice(spheres),
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
		});

		let cube_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Compute Cube Buffer"),
			contents: bytemuck::cast_slice(cubes),
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
		});

		// START: window size dependant
		let (compute_texture, compute_view) = Self::create_compute_texture(&device, &config);

		let (render_texture, render_view) = Self::create_render_texture(&device, &config);

		let compute_bind_group = Self::create_compute_bind_group(
			&device,
			&compute_bind_group_layout,
			&sphere_buffer,
			&cube_buffer,
			&compute_view,
		);

		let render_bind_group = Self::create_render_bind_group(
			&device,
			&render_bind_group_layout,
			&render_view,
			&sampler,
		);
		// END: window size dependant

		let compute_pipeline = Self::create_compute_pipeline(
			Cow::Borrowed(include_str!("shaders/compute.wgsl")),
			&device,
			&compute_bind_group_layout,
			&camera_uniform_buffer,
			&frame_counter_uniform_buffer,
		)
		.expect("failed to compile compute shader");

		let render_pipeline = Self::create_render_pipeline(
			Cow::Borrowed(include_str!("shaders/render.wgsl")),
			&device,
			&render_bind_group_layout,
			&frame_counter_uniform_buffer,
			&config,
		)
		.expect("failed to compile render shader");

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
			frame_counter,
			frame_counter_uniform_buffer,
			compute_pipeline,
			compute_texture,
			compute_view,
			sphere_buffer,
			cube_buffer,
			compute_bind_group,
			compute_bind_group_layout,
			render_pipeline,
			render_texture,
			render_view,
			render_bind_group,
			render_bind_group_layout,
		}
	}

	fn create_shader_module(
		device: &Device,
		shader_code: Cow<'static, str>,
		label: Option<&'static str>,
	) -> Result<ShaderModule, String> {
		device.push_error_scope(ErrorFilter::Validation);

		let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label,
			source: wgpu::ShaderSource::Wgsl(shader_code),
		});

		if let Some(e) = pollster::block_on(device.pop_error_scope()) {
			return Err(match e {
				wgpu::Error::OutOfMemory { source } => format!("Out of Memory: {source}"),
				wgpu::Error::Validation { description, .. } => {
					format!("Failed to compile shader:\n{description}")
				}
			});
		}

		Ok(shader)
	}

	fn create_compute_pipeline(
		shader_code: Cow<'static, str>,
		device: &Device,
		compute_bind_group_layout: &BindGroupLayout,
		camera_uniform_buffer: &CameraUniformBuffer,
		frame_counter_uniform_buffer: &UniformBuffer<u32, FRAME_COUNTER_UNIFORM_BUFFER_BIND_GROUP>,
	) -> Result<ComputePipeline, String> {
		let shader = Self::create_shader_module(
			device,
			shader_code,
			Some("Raytrace Compute Shader Module"),
		)?;

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Raytrace Compute Pipeline Layout"),
			bind_group_layouts: &[
				compute_bind_group_layout,
				camera_uniform_buffer.get_binding(),
				frame_counter_uniform_buffer.get_binding(),
			],
			push_constant_ranges: &[],
		});

		let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
			label: Some("Raytrace Compute Pipeline"),
			layout: Some(&pipeline_layout),
			module: &shader,
			entry_point: "main",
		});
		Ok(pipeline)
	}

	fn create_render_pipeline(
		shader_code: Cow<'static, str>,
		device: &Device,
		render_bind_group_layout: &BindGroupLayout,
		frame_counter_uniform_buffer: &UniformBuffer<u32, FRAME_COUNTER_UNIFORM_BUFFER_BIND_GROUP>,
		config: &SurfaceConfiguration,
	) -> Result<RenderPipeline, String> {
		let shader = Self::create_shader_module(device, shader_code, None)?;

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[
				render_bind_group_layout,
				frame_counter_uniform_buffer.get_binding(),
			],
			push_constant_ranges: &[],
		});

		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
		});
		Ok(pipeline)
	}

	fn create_compute_texture(
		device: &Device,
		config: &SurfaceConfiguration,
	) -> (Texture, TextureView) {
		let texture = device.create_texture(&wgpu::TextureDescriptor {
			label: Some("Compute Texture Storage"),
			size: wgpu::Extent3d {
				width: config.width,
				height: config.height,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba32Float,
			usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
			view_formats: &[],
		});
		let texture_view = texture.create_view(&TextureViewDescriptor::default());
		(texture, texture_view)
	}

	fn create_compute_bind_group(
		device: &Device,
		compute_bind_group_layout: &BindGroupLayout,
		sphere_buffer: &Buffer,
		cube_buffer: &Buffer,
		compute_view: &TextureView,
	) -> BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Compute Bind Group"),
			layout: compute_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: sphere_buffer.as_entire_binding(),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: cube_buffer.as_entire_binding(),
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: wgpu::BindingResource::TextureView(compute_view),
				},
			],
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
			format: wgpu::TextureFormat::Rgba32Float,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			view_formats: &[],
		});
		let texture_view = texture.create_view(&TextureViewDescriptor::default());
		(texture, texture_view)
	}

	/// resizing also resets accumulation!
	pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
		self.config.width = new_size.width.max(1);
		self.config.height = new_size.height.max(1);
		self.surface.configure(&self.device, &self.config);

		let (compute_texture, compute_view) =
			Self::create_compute_texture(&self.device, &self.config);
		self.compute_texture = compute_texture;
		self.compute_view = compute_view;

		let (render_texture, render_view) = Self::create_render_texture(&self.device, &self.config);
		self.render_texture = render_texture;
		self.render_view = render_view;

		self.compute_bind_group = Self::create_compute_bind_group(
			&self.device,
			&self.compute_bind_group_layout,
			&self.sphere_buffer,
			&self.cube_buffer,
			&self.compute_view,
		);

		self.render_bind_group = Self::create_render_bind_group(
			&self.device,
			&self.render_bind_group_layout,
			&self.render_view,
			&self.sampler,
		);

		self.camera_uniform_buffer.update(
			self.camera
				.get_uniform(self.config.width as f32, self.config.height as f32),
			&self.queue,
		);

		self.reset_accumulation();
	}

	pub fn update(&mut self) {
		let output = self.surface.get_current_texture().unwrap();
		let view = output
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		self.frame_counter_uniform_buffer
			.update(self.frame_counter, &self.queue);

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

		self.compute_pass(&mut encoder);

		self.render_pass(&mut encoder, &view);

		self.queue.submit(Some(encoder.finish()));
		output.present();

		self.frame_counter += 1;
	}

	/// hot reloading also resets accumulation!
	pub fn hot_reload_shaders_from_files(
		&mut self,
		compute_shader_filepath: impl AsRef<Path>,
		render_shader_filepath: impl AsRef<Path>,
	) -> Result<(), String> {
		let compute_shader_code = std::fs::read_to_string(compute_shader_filepath)
			.map_err(|e| format!("failed to read compute.wgsl shader file: {e}"))?;
		let render_shader_code = std::fs::read_to_string(render_shader_filepath)
			.map_err(|e| format!("failed to read render.wgsl shader file: {e}"))?;
		self.hot_reload_shaders(
			Cow::Owned(compute_shader_code),
			Cow::Owned(render_shader_code),
		)
	}

	/// hot reloading also resets accumulation!
	pub fn hot_reload_shaders(
		&mut self,
		compute_shader_code: Cow<'static, str>,
		render_shader_code: Cow<'static, str>,
	) -> Result<(), String> {
		let compute_pipeline = Self::create_compute_pipeline(
			compute_shader_code,
			&self.device,
			&self.compute_bind_group_layout,
			&self.camera_uniform_buffer,
			&self.frame_counter_uniform_buffer,
		)?;

		let render_pipeline = Self::create_render_pipeline(
			render_shader_code,
			&self.device,
			&self.render_bind_group_layout,
			&self.frame_counter_uniform_buffer,
			&self.config,
		)?;

		self.compute_pipeline = compute_pipeline;
		self.render_pipeline = render_pipeline;
		self.reset_accumulation();
		Ok(())
	}

	/// should get called when camera or any objects move
	/// -> resets frame counter and clears accumulation image
	fn reset_accumulation(&mut self) {
		self.frame_counter = 1;
		let (storage_texture, storage_view) =
			Self::create_compute_texture(&self.device, &self.config);
		self.compute_texture = storage_texture;
		self.compute_view = storage_view;
		self.compute_bind_group = Self::create_compute_bind_group(
			&self.device,
			&self.compute_bind_group_layout,
			&self.sphere_buffer,
			&self.cube_buffer,
			&self.compute_view,
		);
	}

	pub fn update_camera(&mut self, new_camera: Camera) {
		if self.camera != new_camera {
			self.camera = new_camera;
			self.camera_uniform_buffer.update(
				self.camera
					.get_uniform(self.config.width as f32, self.config.height as f32),
				&self.queue,
			);
			self.reset_accumulation();
		}
	}

	fn compute_pass(&self, encoder: &mut CommandEncoder) {
		{
			let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
			pass.set_pipeline(&self.compute_pipeline);
			pass.set_bind_group(COMPUTE_BIND_GROUP, &self.compute_bind_group, &[]);
			self.camera_uniform_buffer.bind_compute(&mut pass);
			self.frame_counter_uniform_buffer
				.custom_bind_compute(&mut pass, 2);
			let workgroups_x = (self.config.width + 15) / 16;
			let workgroups_y = (self.config.height + 15) / 16;
			pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
		}

		// Copy from storage texture to render texture
		encoder.copy_texture_to_texture(
			wgpu::ImageCopyTexture {
				texture: &self.compute_texture,
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
		self.frame_counter_uniform_buffer.bind_render(&mut pass);
		pass.draw(0..3, 0..1);
	}
}
