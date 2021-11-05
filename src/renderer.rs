use cgmath::{
    Deg,
    Quaternion,
    Rotation3,
    Vector3,
};
use std::ops::Range;

use crate::{
  camera::{Camera, CameraUniform},
  color::ColorUniform,
  draw::{DrawLight, DrawModel},
  light::LightUniform,
  projection::Projection,
  texture::Texture,
  uniform::Uniform,
};

pub struct Renderer {
  ambient_uniform: Uniform<ColorUniform>,
  camera_uniform: Uniform<CameraUniform>,
  depth_texture: Texture,
  light_render_pipeline: Option<wgpu::RenderPipeline>,
  light_uniform: Uniform<LightUniform>,
  projection: Projection,
  render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
  pub fn new(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
  ) -> Self {
    let camera_uniform = Uniform::new(device, CameraUniform::new(), "camera");
    let ambient_uniform = Uniform::new(device, ColorUniform { color: [0.5, 0.0, 0.0, 1.0] }, "ambient");
    let light_uniform = Uniform::new(
      device,
      LightUniform {
        position: [2.0, 2.0, 2.0],
        _padding: 0,
        color: [1.0, 1.0, 1.0],
      },
      "light",
    );

    let depth_texture = Texture::create_depth_texture(device, config, "depth_texture");

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("render Pipeline Layout"),
      bind_group_layouts: &[
        &ambient_uniform.bind_group_layout,
        &camera_uniform.bind_group_layout,
        &light_uniform.bind_group_layout,
      ],
      push_constant_ranges: &[],
    });
    let render_pipeline = {
      let shader = wgpu::ShaderModuleDescriptor {
        label: Some("Normal Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
      };

      create_render_pipeline(
        device,
        &render_pipeline_layout,
        config.format,
        depth_format,
        vertex_layouts,
        shader,
        "Render Pipeline",
      )
    };

    let projection = Projection::new(config.width, config.height, Deg(45.0), 0.1, 100.0);

    Self {
      ambient_uniform,
      camera_uniform,
      depth_texture,
      light_render_pipeline: None,
      light_uniform,
      projection,
      render_pipeline,
    }
  }

  pub fn disable_light_render_pipeline(&mut self) {
    self.light_render_pipeline = None;
  }

  pub fn enable_light_render_pipeline(
    &mut self,
    device: &wgpu::Device,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
  ) {
    let light_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Light Pipeline Layout"),
      bind_group_layouts: &[&self.camera_uniform.bind_group_layout, &self.light_uniform.bind_group_layout],
      push_constant_ranges: &[],
    });

    self.light_render_pipeline = {
      let shader = wgpu::ShaderModuleDescriptor {
        label: Some("Light Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/light.wgsl").into()),
      };

      Some(create_render_pipeline(
        device,
        &light_pipeline_layout,
        color_format,
        depth_format,
        vertex_layouts,
        shader,
        "Light Render Pipeline",
      ))
    };
  }

  pub fn light_render_pipeline_enabled(&self) -> bool {
    match self.light_render_pipeline {
      Some(_) => true,
      _ => false
    }
  }

  pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
    self.depth_texture = Texture::create_depth_texture(device, config, "depth_texture");
    self.projection.resize(config.width, config.height);
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
                load: wgpu::LoadOp::Clear(self.ambient_uniform.uniform.into()),
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

      match &self.light_render_pipeline {
        Some(render_pipeline) => {
          render_pass.set_pipeline(&render_pipeline);
          render_pass.draw_light_model(
            model,
            &self.camera_uniform.bind_group,
            &self.light_uniform.bind_group,
          );
        }
        _ => ()
      }

      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.draw_model_instanced(
        model,
        num_instances,
        &self.ambient_uniform.bind_group,
        &self.camera_uniform.bind_group,
        &self.light_uniform.bind_group,
      );
    }
    queue.submit(std::iter::once(encoder.finish()));
  }

  pub fn update(&mut self, queue: &wgpu::Queue, dt: std::time::Duration) {
    queue.write_buffer(&self.camera_uniform.buffer, 0, bytemuck::cast_slice(&[self.camera_uniform.uniform]));

    let old_position: Vector3<_> = self.light_uniform.uniform.position.into();

    self.light_uniform.uniform.position = (
        Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), Deg(60.0 * dt.as_secs_f32()))* old_position
    ).into();

    queue.write_buffer(&self.light_uniform.buffer, 0, bytemuck::cast_slice(&[self.light_uniform.uniform]));
  }

  pub fn update_camera_uniform<C: Camera>(&mut self, camera: &C) {
    self.camera_uniform.uniform.update_view_proj(camera, &self.projection);
  }
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
    label: &str,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(&shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
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
