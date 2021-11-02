use anyhow::{Context, Result};
use cgmath::InnerSpace;
use rayon::iter::{IntoParallelRefIterator, IntoParallelIterator, ParallelIterator};
use std::path::Path;
use tobj::LoadOptions;
use wgpu::util::DeviceExt;

use crate::{
  material::Material,
  mesh::Mesh,
  texture::Texture,
};

pub trait Vertex {
  fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
  position: [f32; 3],
  tex_coords: [f32; 2],
  normal: [f32; 3],
  tangent: [f32; 3],
  bitangent: [f32; 3],
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
          offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
          shader_location: 2,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
          shader_location: 3,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
          shader_location: 4,
          format: wgpu::VertexFormat::Float32x3,
        },
      ],
    }
  }
}

pub struct Model {
  pub meshes: Vec<Mesh>,
  pub materials: Vec<Material>,
}

impl Model {
  pub fn load<P: AsRef<Path>>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    path: P,
  ) -> Result<Self> {
    let (obj_models, obj_materials) = tobj::load_obj(path.as_ref(), &LoadOptions {
      triangulate: true,
      single_index: true,
      ..Default::default()
    })?;
    let obj_materials = obj_materials?;
    let containing_folder = path.as_ref().parent().context("Directory has no parten")?;
    let materials = obj_materials.par_iter().map(|mat| {
      let mut textures = [
        (containing_folder.join(&mat.diffuse_texture), false),
        (containing_folder.join(&mat.normal_texture), true),
      ].par_iter().map(|(texture_path, is_normal_map)| {
        Texture::load(device, queue, texture_path, *is_normal_map)
      }).collect::<Result<Vec<_>>>()?;

      let normal_texture = textures.pop().unwrap();
      let diffuse_texture = textures.pop().unwrap();

      Ok(Material::new(
        device,
        &mat.name,
        diffuse_texture,
        normal_texture,
        layout,
      ))
    }).collect::<Result<Vec<Material>>>()?;
    let meshes = obj_models.iter().map(|m| {
      let mut vertices = (0..m.mesh.positions.len() / 3).into_par_iter().map(|i| {
        ModelVertex {
          position: [
            m.mesh.positions[i * 3],
            m.mesh.positions[i * 3 + 1],
            m.mesh.positions[i * 3 + 2],
          ].into(),
          tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]].into(),
          normal: [
            m.mesh.normals[i * 3],
            m.mesh.normals[i * 3 + 1],
            m.mesh.normals[i * 3 + 2],
          ].into(),
          tangent: [0.0; 3].into(),
          bitangent: [0.0; 3].into(),
        }
      }).collect::<Vec<_>>();

      let indices = &m.mesh.indices;
      let mut triangles_included = (0..vertices.len()).collect::<Vec<_>>();

      for c in indices.chunks(3) {
        let v0 = vertices[c[0] as usize];
        let v1 = vertices[c[1] as usize];
        let v2 = vertices[c[2] as usize];

        let pos0: cgmath::Vector3<_> = v0.position.into();
        let pos1: cgmath::Vector3<_> = v1.position.into();
        let pos2: cgmath::Vector3<_> = v2.position.into();

        let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
        let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
        let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

        let delta_pos1 = pos1 - pos0;
        let delta_pos2 = pos2 - pos0;

        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
        let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

        vertices[c[0] as usize].tangent = (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
        vertices[c[1] as usize].tangent = (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
        vertices[c[2] as usize].tangent = (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();
        vertices[c[0] as usize].bitangent = (bitangent + cgmath::Vector3::from(vertices[c[0] as usize].bitangent)).into();
        vertices[c[1] as usize].bitangent = (bitangent + cgmath::Vector3::from(vertices[c[1] as usize].bitangent)).into();
        vertices[c[2] as usize].bitangent = (bitangent + cgmath::Vector3::from(vertices[c[2] as usize].bitangent)).into();

        triangles_included[c[0] as usize] += 1;
        triangles_included[c[1] as usize] += 1;
        triangles_included[c[2] as usize] += 1;
      }

      for (i, n) in triangles_included.into_iter().enumerate() {
        let denom = 1.0 / n as f32;
        let mut v = &mut vertices[i];

        v.tangent = (cgmath::Vector3::from(v.tangent) * denom)
          .normalize()
          .into();
        v.bitangent = (cgmath::Vector3::from(v.bitangent) * denom)
          .normalize()
          .into();
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

    Ok(Self { meshes, materials })
  }
}
