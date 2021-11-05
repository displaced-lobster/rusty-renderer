use wgpu::util::DeviceExt;

pub struct Uniform<U> {
  pub bind_group: wgpu::BindGroup,
  pub bind_group_layout: wgpu::BindGroupLayout,
  pub buffer: wgpu::Buffer,
  pub uniform: U,
}

impl<U> Uniform<U> {
  pub fn new(device: &wgpu::Device, uniform: U, label: &str) -> Self
  where
    U: bytemuck::Pod
  {
    let buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&[uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      }
    );
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        }
      ],
      label: Some(&format!("{}_binding_group_layout", label)),
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: &bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: buffer.as_entire_binding(),
        }
      ],
      label: Some(&format!("{}_bind_group", label)),
    });

    Uniform {
      bind_group,
      bind_group_layout,
      buffer,
      uniform,
    }
  }
}
