use crate::{
  draw::DrawModel,
  render::create_render_pipeline,
};

pub struct ModelRenderer {
  num_instances: u32,
  render_pipeline: wgpu::RenderPipeline,
}

impl ModelRenderer {
  pub fn new(
    device: &wgpu::Device,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
  ) -> Self {
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("render Pipeline Layout"),
      bind_group_layouts,
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
        format,
        depth_format,
        vertex_layouts,
        shader,
        "Render Pipeline",
      )
    };

    Self {
      num_instances: 1,
      render_pipeline,
    }
  }

  pub fn render<'a>(
    &'a self,
    render_pass: &mut wgpu::RenderPass<'a>,
    model: &'a crate::model::Model,
    ambient_bind_group: &'a wgpu::BindGroup,
    camera_bind_group: &'a wgpu::BindGroup,
    light_bind_group: &'a wgpu::BindGroup,
  ) {
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.draw_model_instanced(
      model,
      0..self.num_instances,
      ambient_bind_group,
      camera_bind_group,
      light_bind_group,
    );
  }
}
