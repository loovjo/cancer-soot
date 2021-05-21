#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use anyhow::Result;

pub(super) struct MenuRender {
    render_pipeline: wgpu::RenderPipeline,
}

impl MenuRender {
    pub(super) async fn new(device: &wgpu::Device, screen_layout_bind_group_layout: &wgpu::BindGroupLayout, format: wgpu::TextureFormat) -> Result<Self> {
        let vs_desc: wgpu::ShaderModuleDescriptor =
            wgpu::include_spirv!("shaders/compiled/menu.vert.spv");

        let fs_desc: wgpu::ShaderModuleDescriptor =
            wgpu::include_spirv!("shaders/compiled/menu.frag.spv");

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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Menu render pipeline layout"),
                bind_group_layouts: &[screen_layout_bind_group_layout],
                push_constant_ranges: &[],
            }
        );

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Menu render pipeline"),
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
                    format: format,
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

        Ok(MenuRender {
            render_pipeline,
        })
    }

    pub(super) fn render(&mut self, encoder: &mut wgpu::CommandEncoder, frame: &wgpu::SwapChainTexture, screen_layout_bind_group: &wgpu::BindGroup) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, screen_layout_bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}
