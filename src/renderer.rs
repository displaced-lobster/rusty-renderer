use crate::texture::Texture;

pub struct Renderer {
  pub depth_texture: Texture,
  pub light_render_pipeline: Option<wgpu::RenderPipeline>,
  pub render_pipeline: wgpu::RenderPipeline,
}

impl Renderer {
  pub fn new(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    config: &wgpu::SurfaceConfiguration,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
  ) -> Self {
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
      depth_texture,
      light_render_pipeline: None,
      render_pipeline,
    }
  }

  pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
    self.depth_texture = Texture::create_depth_texture(device, config, "depth_texture");
  }

  // pub fn render(&self, surface: &wgpu::Surface, device: &wgpu::Device) -> Result<(), wgpu::SurfaceError> {
  //   let output = surface.get_current_frame()?.output;
  //   let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
  //   let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
  //       label: Some("Render Encoder"),
  //   });
  //   {
  //       let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
  //           label: Some("Render Pass"),
  //           color_attachments: &[
  //               wgpu::RenderPassColorAttachment {
  //                   view: &view,
  //                   resolve_target: None,
  //                   ops: wgpu::Operations {
  //                       load: wgpu::LoadOp::Clear(
  //                           wgpu::Color {
  //                               r: 0.1,
  //                               g: 0.2,
  //                               b: 0.3,
  //                               a: 1.0,
  //                           }
  //                       ),
  //                       store: true,
  //                   },
  //               }
  //           ],
  //           depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
  //               view: &self.depth_texture.view,
  //               depth_ops: Some(wgpu::Operations {
  //                   load: wgpu::LoadOp::Clear(1.0),
  //                   store: true,
  //               }),
  //               stencil_ops: None,
  //           }),
  //       });
  //       render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

  //       if let Some(render_pipeline) = &self.renderer.light_render_pipeline {
  //           render_pass.set_pipeline(&render_pipeline);
  //           render_pass.draw_light_model(
  //               &self.obj_model,
  //               &self.camera_bind_group,
  //               &self.light_bind_group,
  //           );
  //       }

  //       render_pass.set_pipeline(&self.renderer.render_pipeline);
  //       render_pass.draw_model_instanced(
  //           &self.obj_model,
  //           0..self.instances.len() as u32,
  //           &self.camera_bind_group,
  //           &self.light_bind_group,
  //       );
  //   }
  // }

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
