use cgmath::{InnerSpace, Vector3};
use wgpu::util::DeviceExt;

const COLOR: [f32;4] = [1.0, 0.1, 0.1, 1.0];

pub trait Vertex {
  fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

pub struct Mesh {
  pub name: String,
  pub vertex_buffer: wgpu::Buffer,
  pub index_buffer: wgpu::Buffer,
  pub num_elements: u32,
  pub material: usize,
}

pub struct MeshBuilder {
  indices: Vec<u32>,
  label: String,
  vertices: Vec<MeshVertex>,
}

impl MeshBuilder {
  pub fn new(label: &str) -> Self {
    Self {
      indices: Vec::new(),
      label: String::from(label),
      vertices: Vec::new(),
    }
  }

  pub fn add_face(&mut self, indices: (u32, u32, u32)) {
    let (i1, i2, i3) = indices;
    self.indices.push(i1);
    self.indices.push(i2);
    self.indices.push(i3);
  }

  pub fn add_linked_quad(&mut self, position: Vector3<f32>, link: bool, index_offset: u32) {
    self.add_vertex(position, Vector3::unit_y());

    if link {
      let i0 = self.vertices.len() as u32 - 1;
      let i1 = i0 - 1;
      let i2 = i0 - index_offset;
      let i3 = i0 - index_offset - 1;

      self.add_face((i0, i2, i1));
      self.add_face((i2, i3, i1));
    }
  }

  pub fn add_quad(&mut self, position: Vector3<f32>, width: Vector3<f32>, length: Vector3<f32>) {
    let normal = length.cross(width).normalize();

    self.add_vertex(position, normal);
    self.add_vertex(position + length, normal);
    self.add_vertex(position + width + length, normal);
    self.add_vertex(position + width, normal);

    let base_index = self.vertices.len() as u32 - 4;

    self.add_face((base_index, base_index + 1, base_index + 2));
    self.add_face((base_index, base_index + 2, base_index + 3));
  }

  pub fn add_triangle(&mut self, v0: Vector3<f32>, v1: Vector3<f32>, v2: Vector3<f32>) {
    let normal = (v1 - v0).cross(v2 - v0).normalize();

    self.add_vertex(v0, normal);
    self.add_vertex(v1, normal);
    self.add_vertex(v2, normal);

    let base_index = self.vertices.len() as u32 - 3;

    self.add_face((base_index, base_index + 1, base_index + 2));
  }

  pub fn add_vertex<P, N>(&mut self, position: P, normal: N)
  where
    P: Into<[f32;3]>,
    N: Into<[f32;3]>,
  {
    self.vertices.push(MeshVertex {
      position: position.into(),
      normal: normal.into(),
      color: COLOR,
    });
  }

  pub fn build(&self, device: &wgpu::Device) -> Mesh {
    let vertex_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{} Vertex Buffer", self.label)),
        contents: bytemuck::cast_slice(&self.vertices),
        usage: wgpu::BufferUsages::VERTEX,
      }
    );
    let index_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{} Index Buffer", self.label)),
        contents: bytemuck::cast_slice(&self.indices),
        usage: wgpu::BufferUsages::INDEX,
      }
    );

    Mesh {
      name: String::from(&self.label),
      vertex_buffer,
      index_buffer,
      num_elements: self.indices.len() as u32,
      material: 0,
    }
  }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
  pub position: [f32; 3],
  pub normal: [f32; 3],
  pub color: [f32; 4],
}

impl Vertex for MeshVertex {
  fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    use std::mem;

    wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 0,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
          shader_location: 2,
          format: wgpu::VertexFormat::Float32x4,
        },
      ],
    }
  }
}
