#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorUniform {
    pub color: [f32; 4],
}

impl From<ColorUniform> for wgpu::Color {
  fn from (uniform: ColorUniform) -> Self {
    wgpu::Color {
      r: uniform.color[0] as f64,
      g: uniform.color[1] as f64,
      b: uniform.color[2] as f64,
      a: uniform.color[3] as f64,
    }
  }
}
