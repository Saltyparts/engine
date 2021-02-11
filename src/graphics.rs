use anyhow::Result;

use bytemuck::{
    Pod,
    Zeroable,
};

use std::{
    borrow::Cow,
    path::Path,
};

use wgpu::{ *, util::* };

const VERTEX_BUFFER_SIZE: u64 = 32000000;
const INDEX_BUFFER_SIZE: u64 = 32000000;
const TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;
const DEPTH_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub struct Instance {
    instance: wgpu::Instance,
    compiler: shaderc::Compiler,

    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    swap_chain_descriptor: SwapChainDescriptor,
    swap_chain: SwapChain,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    texture: Texture,
    texture_view: TextureView,
    sampler: Sampler,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,

    depth_texture: Texture,
    depth_texture_view: TextureView,
    vertex_shader_module: ShaderModule,
    fragment_shader_module: ShaderModule,
    pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,
}

impl Instance {
    pub async fn new<W: raw_window_handle::HasRawWindowHandle>(window: &W, window_size: [u32; 2]) -> Result<Instance> {
        let instance = wgpu::Instance::new(BackendBit::PRIMARY);

        let mut compiler = shaderc::Compiler::new().unwrap();

        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
            &DeviceDescriptor {
                label: Some("device"),
                features: adapter.features(),
                limits: Limits::default(),
            },
            None,
        ).await?;

        let (swap_chain_descriptor, swap_chain) = create_swap_chain(&device, &surface, window_size);
        let (depth_texture, depth_texture_view) = create_depth_texture(&device, window_size);

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("vertex_buffer"),
            size: VERTEX_BUFFER_SIZE,
            usage: BufferUsage::VERTEX | BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("index_buffer"),
            size: INDEX_BUFFER_SIZE,
            usage: BufferUsage::INDEX | BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::cast_slice(&[
                1, 0, 0, 0,
                0, 1, 0, 0,
                0, 0, 1, 0,
                0, 0, 0, 1,
            ]),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        });

        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d { width: 1, height: 1, depth: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        });

        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(64)
                    },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                }
            ]
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: BindingResource::Buffer { buffer: &uniform_buffer, offset: 0, size: None }},
                BindGroupEntry { binding: 1, resource: BindingResource::TextureView(&texture_view) },
                BindGroupEntry { binding: 2, resource: BindingResource::Sampler(&sampler) },
            ],
            label: None,
        });

        let vertex_shader = compiler.compile_into_spirv(include_str!("shader.vert"), shaderc::ShaderKind::Vertex, "shader.vert", "main", None).unwrap();
        let fragment_shader = compiler.compile_into_spirv(include_str!("shader.frag"), shaderc::ShaderKind::Fragment, "shader.frag", "main", None).unwrap();

        let vertex_shader_module = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("vertex_shader"),
            source: ShaderSource::SpirV(Cow::from(vertex_shader.as_binary())),
            flags: ShaderFlags::VALIDATION,
        });

        let fragment_shader_module = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("fragment_shader"),
            source: ShaderSource::SpirV(Cow::from(fragment_shader.as_binary())),
            flags: ShaderFlags::VALIDATION,
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[]
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &vertex_shader_module,
                entry_point: "main",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: InputStepMode::Vertex,
                    attributes: &vertex_attr_array![
                        0 => Float3,
                        1 => Float2,
                        2 => Float3
                    ],
                }]
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::Back,
                polygon_mode: PolygonMode::Fill,
            },
            depth_stencil: Some(DepthStencilState {
                format: DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
                clamp_depth: false,
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &fragment_shader_module,
                entry_point: "main",
                targets: &[ColorTargetState {
                    format: TEXTURE_FORMAT,
                    alpha_blend: BlendState::REPLACE,
                    color_blend: BlendState::REPLACE,
                    write_mask: ColorWrite::ALL,
                }]
            }),
        });

        Ok(Instance{
            instance,
            compiler,

            surface,
            adapter,
            device,
            queue,
            swap_chain_descriptor,
            swap_chain,

            vertex_buffer,
            index_buffer,
            uniform_buffer,
            texture,
            texture_view,
            sampler,
            bind_group_layout,
            bind_group,

            depth_texture,
            depth_texture_view,
            vertex_shader_module,
            fragment_shader_module,
            pipeline_layout,
            pipeline,
        })
    }

    pub fn render(&mut self, view_matrix: &[f32; 16], models: &[&Model]) -> Result<()> {
        let mut meshes = vec![];
        let mut vertices: Vec<Vertex> = vec![];
        let mut indices: Vec<u32> = vec![];
        for model in models {
            for mesh in &model.meshes {
                meshes.push((
                    vertices.len() as i32,
                    indices.len() as u32..(indices.len() + mesh.indices.len()) as u32,
                ));
                vertices.extend(&mesh.vertices);
                indices.extend(&mesh.indices);
            }
        }

        let vertex_data = bytemuck::cast_slice(&vertices);
        let index_data = bytemuck::cast_slice(&indices);

        self.queue.write_buffer(&self.vertex_buffer, 0, vertex_data);
        self.queue.write_buffer(&self.index_buffer, 0, index_data);
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(view_matrix));

        let frame = self.swap_chain.get_current_frame()?;
        let render_texture = frame.output;

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &render_texture.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..vertex_data.len() as u64));
            render_pass.set_index_buffer(self.index_buffer.slice(0..index_data.len() as u64), IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_pipeline(&self.pipeline);
            for mesh in meshes {
                render_pass.draw_indexed(mesh.1, mesh.0, 0..1);
            }
        }

        self.queue.submit(Some(encoder.finish()));

        Ok(())
    }

    pub fn resize(&mut self, size: [u32; 2]) {
        if size[0] == 0 || size[1] == 0 { return; }

        let (swap_chain_descriptor, swap_chain) = create_swap_chain(&self.device, &self.surface, size);
        self.swap_chain_descriptor = swap_chain_descriptor;
        self.swap_chain = swap_chain;

        let (depth_texture, depth_texture_view) = create_depth_texture(&self.device, size);
        self.depth_texture = depth_texture;
        self.depth_texture_view = depth_texture_view;
    }
}

fn create_swap_chain(device: &Device, surface: &Surface, size: [u32; 2]) -> (SwapChainDescriptor, SwapChain) {
    let swap_chain_descriptor = SwapChainDescriptor {
        usage: TextureUsage::RENDER_ATTACHMENT,
        format: TEXTURE_FORMAT,
        width: size[0],
        height: size[1],
        present_mode: PresentMode::Mailbox,
    };

    let swap_chain = device.create_swap_chain(surface, &swap_chain_descriptor);

    (swap_chain_descriptor, swap_chain)
}

fn create_depth_texture(device: &Device, size: [u32; 2]) -> (Texture, TextureView) {
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("depth texture"),
        size: Extent3d { width: size[0], height: size[1], depth: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: DEPTH_TEXTURE_FORMAT,
        usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    });

    let view = texture.create_view(&TextureViewDescriptor::default());

    (texture, view)
}

pub struct Model {
    pub meshes: Vec<Mesh>,
}

impl Model {
    pub fn new<T: AsRef<Path>>(path: T) -> Result<Model> {
        let (models, _materials) = tobj::load_obj(path.as_ref(), true)?;
        let mut meshes = vec![];
        for model in models {
            let mut vertices = vec![];
            for i in 0..model.mesh.positions.len() / 3 {
                vertices.push(Vertex {
                    position: [
                        model.mesh.positions[i * 3],
                        model.mesh.positions[i * 3 + 1],
                        model.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [
                        model.mesh.texcoords[i * 2],
                        model.mesh.texcoords[i * 2 + 1]
                    ],
                    normal: [
                        model.mesh.normals[i * 3],
                        model.mesh.normals[i * 3 + 1],
                        model.mesh.normals[i * 3 + 2],
                    ],
                });
            }

            meshes.push(Mesh {
                vertices,
                indices: model.mesh.indices,
            });
        }

        Ok(Model {
            meshes,
        })
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}
