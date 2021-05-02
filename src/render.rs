use anyhow::anyhow;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use anyhow::Result;

use winit::dpi::PhysicalSize;
use winit::window::Window;

use wgpu::util::DeviceExt;

pub struct Render {
    pub size: PhysicalSize<u32>,

    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    render_pipeline: wgpu::RenderPipeline,

    render_state_buffer: wgpu::Buffer,
    render_state_bind_group: wgpu::BindGroup,
}

impl Render {
    pub async fn new(win: &Window) -> Result<Self> {
        let inst = wgpu::Instance::new(wgpu::BackendBit::all());

        let surface = unsafe { inst.create_surface(win) };

        let adapter = inst
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .ok_or(anyhow!("Could not find adapter!"))?;

        debug!("Got adapter: {:?}", adapter);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Main device"),
                    features: wgpu::Features::PUSH_CONSTANTS,
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        debug!("Got device + queue: {:?} + {:?}", device, queue);

        let size = win.inner_size();
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter
                .get_swap_chain_preferred_format(&surface)
                .unwrap_or_else(|| {
                    warn!("Swapchain has no preferred formats");
                    wgpu::TextureFormat::Rgba32Uint
                }),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        debug!("Created swap chain {:?}", swap_chain);

        let vs_desc: wgpu::ShaderModuleDescriptor =
            wgpu::include_spirv!("shaders/compiled/section.vert.spv");

        let fs_desc: wgpu::ShaderModuleDescriptor =
            wgpu::include_spirv!("shaders/compiled/section.frag.spv");

        let vs_mod = device.create_shader_module(&vs_desc);
        let fs_mod = device.create_shader_module(&fs_desc);

        let blending = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
        };

        let render_state_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Render state buffer"),
            contents: &[0u8; std::mem::size_of::<crate::state::RenderState>()],
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let render_state_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Render state bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let render_state_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render state bind group"),
            layout: &render_state_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: render_state_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render pipeline layout"),
                bind_group_layouts: &[&render_state_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_mod,
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_mod,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    blend: Some(blending),
                    format: sc_desc.format,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Front),
                polygon_mode: wgpu::PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !1,
                alpha_to_coverage_enabled: false,
            },
        });

        Ok(Render {
            surface,
            device,
            queue,

            sc_desc,
            swap_chain,
            size,

            render_pipeline,

            render_state_bind_group,
            render_state_buffer,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    pub fn render(&mut self, render_state: &crate::state::RenderState) -> std::result::Result<(), wgpu::SwapChainError> {
        self.queue.write_buffer(&self.render_state_buffer, 0, bytemuck::cast_slice::<_, u8>(&[*render_state]));

        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.,
                            g: 0.,
                            b: 0.,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.render_state_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
        self.queue.submit(vec![encoder.finish()]);

        Ok(())
    }
}