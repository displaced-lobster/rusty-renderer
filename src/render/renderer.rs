use cgmath::{
    Deg,
    Quaternion,
    Rotation3,
    Vector3,
};

use crate::{
  camera::{Camera, CameraUniform},
  color::ColorUniform,
  instance::InstanceRaw,
  light::LightUniform,
  model::{ModelVertex, Vertex},
  projection::Projection,
  render::{LightRenderer, ModelRenderer},
  texture::Texture,
  uniform::Uniform,
};

pub struct Renderer {
  ambient_uniform: Uniform<ColorUniform>,
  camera_uniform: Uniform<CameraUniform>,
  depth_texture: Texture,
  light_renderer: Option<LightRenderer>,
  light_uniform: Uniform<LightUniform>,
  model_renderer: Option<ModelRenderer>,
  projection: Projection,
}

impl Renderer {
  pub fn new(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
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
    let projection = Projection::new(config.width, config.height, Deg(45.0), 0.1, 100.0);

    Self {
      ambient_uniform,
      camera_uniform,
      depth_texture,
      light_renderer: None,
      light_uniform,
      model_renderer: None,
      projection,
    }
  }

  pub fn add_model(
    &mut self,
    device: &wgpu::Device,
    color_format: wgpu::TextureFormat,
  ) {
    self.model_renderer = Some(ModelRenderer::new(
      device,
      &[
        &self.ambient_uniform.bind_group_layout,
        &self.camera_uniform.bind_group_layout,
        &self.light_uniform.bind_group_layout,
      ],
      color_format,
      Some(Texture::DEPTH_FORMAT),
      &[ModelVertex::desc(), InstanceRaw::desc()],
    ));
  }

  pub fn disable_light_render(&mut self) {
    self.light_renderer = None;
  }

  pub fn enable_light_render(
    &mut self,
    device: &wgpu::Device,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
  ) {
    self.light_renderer = Some(LightRenderer::new(
      device,
      &[&self.camera_uniform.bind_group_layout, &self.light_uniform.bind_group_layout],
      color_format,
      depth_format,
      vertex_layouts,
    ));
  }

  pub fn light_render_enabled(&self) -> bool {
    match self.light_renderer {
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

      if let Some(renderer) = &self.light_renderer {
        renderer.render(
          &mut render_pass,
          model,
          &self.camera_uniform.bind_group,
          &self.light_uniform.bind_group,
        )
      }

      if let Some(renderer) = &self.model_renderer {
        renderer.render(
          &mut render_pass,
          model,
          &self.ambient_uniform.bind_group,
          &self.camera_uniform.bind_group,
          &self.light_uniform.bind_group,
        )
      }
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
