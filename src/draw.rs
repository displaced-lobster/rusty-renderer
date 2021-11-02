use std::ops::Range;

use crate::{
  material::Material,
  mesh::Mesh,
  model::Model,
};

pub trait DrawModel<'a> {
  fn draw_mesh(
    &mut self,
    mesh: &'a Mesh,
    material: &'a Material,
    camera: &'a wgpu::BindGroup,
    light: &'a wgpu::BindGroup,
  );
  fn draw_mesh_instanced(
    &mut self,
    mesh: &'a Mesh,
    material: &'a Material,
    instances: Range<u32>,
    camera: &'a wgpu::BindGroup,
    light: &'a wgpu::BindGroup,
  );
  fn draw_model(
    &mut self,
    model: &'a Model,
    camera: &'a wgpu::BindGroup,
    light: &'a wgpu::BindGroup,
  );
  fn draw_model_instanced(
    &mut self,
    model: &'a Model,
    instances: Range<u32>,
    camera: &'a wgpu::BindGroup,
    light: &'a wgpu::BindGroup,
  );
  fn draw_model_instanced_with_material(
    &mut self,
    model: &'a Model,
    material: &'a Material,
    instances: Range<u32>,
    camera: &'a wgpu::BindGroup,
    light: &'a wgpu::BindGroup,
  );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
  'b: 'a,
{
  fn draw_mesh(
    &mut self,
    mesh: &'b Mesh,
    material: &'b Material,
    camera: &'b wgpu::BindGroup,
    light: &'a wgpu::BindGroup,
  ) {
    self.draw_mesh_instanced(mesh, material, 0..1, camera, light);
  }

  fn draw_mesh_instanced(
    &mut self,
    mesh: &'b Mesh,
    material: &'b Material,
    instances: Range<u32>,
    camera: &'b wgpu::BindGroup,
    light: &'a wgpu::BindGroup,
  ){
    self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
    self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    self.set_bind_group(0, &material.bind_group, &[]);
    self.set_bind_group(1, camera, &[]);
    self.set_bind_group(2, light, &[]);
    self.draw_indexed(0..mesh.num_elements, 0, instances);
  }

  fn draw_model(
    &mut self,
    model: &'b Model,
    camera: &'b wgpu::BindGroup,
    light: &'a wgpu::BindGroup,
  ) {
    self.draw_model_instanced(model, 0..1, camera, light);
  }

  fn draw_model_instanced(
    &mut self,
    model: &'b Model,
    instances: Range<u32>,
    camera: &'b wgpu::BindGroup,
    light: &'a wgpu::BindGroup,
  ) {
    for mesh in &model.meshes {
      let material = &model.materials[mesh.material];

      self.draw_mesh_instanced(mesh, material, instances.clone(), camera, light);
    }
  }

  fn draw_model_instanced_with_material(
    &mut self,
    model: &'a Model,
    material: &'b Material,
    instances: Range<u32>,
    camera: &'b wgpu::BindGroup,
    light: &'b wgpu::BindGroup,
  ) {
    for mesh in &model.meshes {
      self.draw_mesh_instanced(mesh, material, instances.clone(), camera, light);
    }
  }
}

pub trait DrawLight<'a> {
    fn draw_light_mesh(
        &mut self,
        mesh: &'a Mesh,
        camera: &'a wgpu::BindGroup,
        light: &'a wgpu::BindGroup,
    );
    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
        camera: &'a wgpu::BindGroup,
        light: &'a wgpu::BindGroup,
    );

    fn draw_light_model(
        &mut self,
        model: &'a Model,
        camera: &'a wgpu::BindGroup,
        light: &'a wgpu::BindGroup,
    );
    fn draw_light_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera: &'a wgpu::BindGroup,
        light: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawLight<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_light_mesh(
        &mut self,
        mesh: &'b Mesh,
        camera: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.draw_light_mesh_instanced(mesh, 0..1, camera, light);
    }

    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        camera: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, camera, &[]);
        self.set_bind_group(1, light, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_light_model(
        &mut self,
        model: &'b Model,
        camera: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        self.draw_light_model_instanced(model, 0..1, camera, light);
    }
    fn draw_light_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        camera: &'b wgpu::BindGroup,
        light: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            self.draw_light_mesh_instanced(mesh, instances.clone(), camera, light);
        }
    }
}
