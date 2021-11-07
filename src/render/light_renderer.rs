use crate::{
  draw::DrawLight,
  render::create_render_pipeline,
};

pub struct LightRenderer {
  render_pipeline: wgpu::RenderPipeline,
}

impl LightRenderer {
  pub fn new(
    device: &wgpu::Device,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
  ) -> Self {
    let light_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Light Pipeline Layout"),
      bind_group_layouts,
      push_constant_ranges: &[],
    });

    let render_pipeline = {
      let shader = wgpu::ShaderModuleDescriptor {
        label: Some("Light Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/light.wgsl").into()),
      };

      create_render_pipeline(
        device,
        &light_pipeline_layout,
        format,
        depth_format,
        vertex_layouts,
        shader,
        "Light Render Pipeline",
      )
    };

    Self { render_pipeline }
  }

  pub fn render<'a>(
    &'a self,
    render_pass: &mut wgpu::RenderPass<'a>,
    model: &'a crate::model::Model,
    camera_bind_group: &'a wgpu::BindGroup,
    light_bind_group: &'a wgpu::BindGroup,
  ) {
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.draw_light_model(
      model,
      camera_bind_group,
      light_bind_group,
    );
  }
}
