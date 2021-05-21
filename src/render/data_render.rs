use anyhow::anyhow;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use anyhow::Result;

pub(super) struct DataRender {
    texture_size: wgpu::Extent3d,
    data_texture: wgpu::Texture,
    data_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl DataRender {
    pub(super) async fn new(device: &wgpu::Device, screen_layout_bind_group_layout: &wgpu::BindGroupLayout, format: wgpu::TextureFormat) -> Result<Self> {
        let texture_size = wgpu::Extent3d {
            width: 256,
            height: 256,
            depth_or_array_layers: 1,
        };

        let data_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some("2d data texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R32Float,
                usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED,
            }
        );

        let data_texture_view = data_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let data_sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                label: Some("Data sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,

                ..wgpu::SamplerDescriptor::default()
            }
        );

        let data_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Data bind group"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: false,
                        },
                        count: None,
                    },
                ],
            }
        );

        let data_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Data bind group"),
                layout: &data_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&data_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&data_sampler),
                    },
                ],
            }
        );

        let vs_desc: wgpu::ShaderModuleDescriptor =
            wgpu::include_spirv!("shaders/compiled/data.vert.spv");

        let fs_desc: wgpu::ShaderModuleDescriptor =
            wgpu::include_spirv!("shaders/compiled/data.frag.spv");

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
                label: Some("Data render pipeline layout"),
                bind_group_layouts: &[
                    screen_layout_bind_group_layout,
                    &data_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Data render pipeline"),
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

        Ok(DataRender {
            texture_size,
            data_texture,
            data_bind_group,
            render_pipeline,
        })
    }

    pub(super) fn render(&mut self, encoder: &mut wgpu::CommandEncoder, queue: &mut wgpu::Queue, frame: &wgpu::SwapChainTexture, screen_layout_bind_group: &wgpu::BindGroup, data: &[[f32; 256]; 256]) {
        unsafe {
            use std::num::NonZeroU32;
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &self.data_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::default(),
                },
                bytemuck::cast_slice(data), // TODO: fucking don't
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(NonZeroU32::new_unchecked(256 * 4)),
                    rows_per_image: Some(NonZeroU32::new_unchecked(256)),
                },
                self.texture_size,
            );
        }

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
        render_pass.set_bind_group(0, screen_layout_bind_group, &[]);
        render_pass.set_bind_group(1, &self.data_bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}
