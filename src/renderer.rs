use cgmath::{
    Deg,
    Quaternion,
    Rotation3,
    Vector3,
};
use std::ops::Range;
use wgpu::util::DeviceExt;

use crate::{
  camera::{Camera, CameraUniform},
  draw::{DrawLight, DrawModel},
  light::LightUniform,
  projection::Projection,
  texture::Texture,
};

pub struct Renderer {
  camera_bind_group: Option<wgpu::BindGroup>,
  camera_buffer: wgpu::Buffer,
  camera_uniform: CameraUniform,
  depth_texture: Texture,
  light_bind_group: Option<wgpu::BindGroup>,
  light_buffer: wgpu::Buffer,
  light_uniform: LightUniform,
  light_render_pipeline: Option<wgpu::RenderPipeline>,
  render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
  pub fn new(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    config: &wgpu::SurfaceConfiguration,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
  ) -> Self {
    let camera_uniform = CameraUniform::new();
    let camera_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),

            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    );
    let light_uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
        };
    let light_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    );
    let depth_texture = Texture::create_depth_texture(device, config, "depth_texture");
    let render_pipeline = {
      let shader = wgpu::ShaderModuleDescriptor {
        label: Some("Normal Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
      };

      create_render_pipeline(
        device,
        layout,
        config.format,
        depth_format,
        vertex_layouts,
        shader,
      )
    };

    Self {
      camera_bind_group: None,
      camera_buffer,
      camera_uniform,
      depth_texture,
      light_bind_group: None,
      light_buffer,
      light_render_pipeline: None,
      light_uniform,
      render_pipeline,
    }
  }

  pub fn update_camera_uniform<C: Camera>(&mut self, camera: &C, projection: &Projection) {
    self.camera_uniform.update_view_proj(camera, projection);
  }

  pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
    self.depth_texture = Texture::create_depth_texture(device, config, "depth_texture");
  }

  pub fn render(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    view: &wgpu::TextureView,
    model: &crate::model::Model,
    instance_buffer: &wgpu::Buffer,
    num_instances: Range<u32>,
  ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }
                            ),
                            store: true,
                        },
                    }
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

            match (
                &self.light_render_pipeline,
                &self.camera_bind_group,
                &self.light_bind_group,
            ) {
                (
                    Some(render_pipeline),
                    Some(camera_bind_group),
                    Some(light_bind_group),
                ) => {
                    render_pass.set_pipeline(&render_pipeline);
                    render_pass.draw_light_model(
                        model,
                        &camera_bind_group,
                        &light_bind_group,
                    );
                }
                _ => ()
            }

            match (
                &self.camera_bind_group,
                &self.light_bind_group,
            ) {
                (
                    Some(camera_bind_group),
                    Some(light_bind_group),
                ) => {
                    render_pass.set_pipeline(&self.render_pipeline);
                    render_pass.draw_model_instanced(
                        model,
                        num_instances,
                        &camera_bind_group,
                        &light_bind_group,
                    );
                }
                _ => ()
            }
        }
        queue.submit(std::iter::once(encoder.finish()));
    }

  pub fn set_camera_bind_group(&mut self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) {
    self.camera_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: self.camera_buffer.as_entire_binding(),
            }
        ],
        label: Some("camera_bind_group"),
    }));
  }

  pub fn set_light_bind_group(&mut self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) {
    self.light_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: self.light_buffer.as_entire_binding(),
        }],
        label: None,
    }));
  }

  pub fn set_light_render_pipeline(
    &mut self,
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
  ) {
    self.light_render_pipeline = {
      let shader = wgpu::ShaderModuleDescriptor {
        label: Some("Light Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/light.wgsl").into()),
      };
      Some(create_render_pipeline(
        device,
        layout,
        color_format,
        depth_format,
        vertex_layouts,
        shader,
      ))
    };
  }

  pub fn update(&mut self, queue: &wgpu::Queue, dt: std::time::Duration) {
    queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

    let old_position: Vector3<_> = self.light_uniform.position.into();

    self.light_uniform.position = (
        Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), Deg(60.0 * dt.as_secs_f32()))* old_position
    ).into();

    queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));
  }
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(&shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "main",
            targets: &[wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState{
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            clamp_depth: false,
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
    })
}
