use anyhow::Result;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::path::Path;
use tobj::LoadOptions;
use wgpu::util::DeviceExt;

use crate::mesh::Mesh;

pub trait Vertex {
  fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
  position: [f32; 3],
  normal: [f32; 3],
  color: [f32; 4],
}

impl Vertex for ModelVertex {
  fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    use std::mem;

    wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
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

pub struct Model {
  pub color: [f32; 4],
  pub meshes: Vec<Mesh>,

}

impl Model {
  pub fn load<P: AsRef<Path>>(
    device: &wgpu::Device,
    path: P,
  ) -> Result<Self> {
    let (obj_models, _) = tobj::load_obj(path.as_ref(), &LoadOptions {
      triangulate: true,
      single_index: true,
      ..Default::default()
    })?;
    let meshes = obj_models.iter().map(|m| {
      let vertices = (0..m.mesh.positions.len() / 3).into_par_iter().map(|i| {
        ModelVertex {
          position: [
            m.mesh.positions[i * 3],
            m.mesh.positions[i * 3 + 1],
            m.mesh.positions[i * 3 + 2],
          ].into(),
          normal: [
            m.mesh.normals[i * 3],
            m.mesh.normals[i * 3 + 1],
            m.mesh.normals[i * 3 + 2],
          ].into(),
          color: [0.0, 1.0, 0.0, 1.0],
        }
      }).collect::<Vec<_>>();

      let indices = &m.mesh.indices;
      let mut triangles_included = (0..vertices.len()).collect::<Vec<_>>();

      for c in indices.chunks(3) {

        for i in 0..3 {
          let index = c[i] as usize;

          triangles_included[index] += 1;
        }
      }

      let vertex_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
          label: Some(&format!("{:?} Vertex Buffer", path.as_ref())),
          contents: bytemuck::cast_slice(&vertices),
          usage: wgpu::BufferUsages::VERTEX,
        }
      );
      let index_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
          label: Some(&format!("{:?} Index Buffer", path.as_ref())),
          contents: bytemuck::cast_slice(&m.mesh.indices),
          usage: wgpu::BufferUsages::INDEX,
        }
      );

      Ok(Mesh {
        name: String::from(&m.name),
        vertex_buffer,
        index_buffer,
        num_elements: m.mesh.indices.len() as u32,
        material: m.mesh.material_id.unwrap_or(0),
      })
    }).collect::<Result<Vec<_>>>()?;

    Ok(Self { color: [0.3, 0.3, 0.3, 1.0], meshes })
  }
}


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniform {
    pub color: [f32; 4],
}
