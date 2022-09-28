use super::context;
use std::ops::RangeBounds;

pub struct VertexBuffer {
  pub(crate) buffer: wgpu::Buffer,
}

impl VertexBuffer {
  pub fn new(data: &[u8]) -> Self {
    Self {
      buffer: context().create_buffer(None, data, wgpu::BufferUsages::VERTEX),
    }
  }
}

pub struct IndexBuffer {
  pub(crate) buffer: wgpu::Buffer,
  index_count: u64,
}

impl IndexBuffer {
  pub fn new(data: &[u8]) -> Self {
    Self {
      buffer: context().create_buffer(None, data, wgpu::BufferUsages::INDEX),
      index_count: data.len() as u64,
    }
  }
}

pub struct UniformBuffer<T> {
  pub(crate) buffer: wgpu::Buffer,
  pub data: T,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable> UniformBuffer<T> {
  pub fn new() -> Self {
    let data = T::zeroed();
    Self {
      buffer: context().create_buffer(
        None,
        bytemuck::cast_slice(&[data]),
        wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      ),
      data,
    }
  }
  pub fn update(&self) {
    context().update_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.data]));
  }
  pub fn binding(&self) -> wgpu::BindingResource {
    self.buffer.as_entire_binding()
  }
}
