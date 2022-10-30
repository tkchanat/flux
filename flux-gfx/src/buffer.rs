use bytemuck::{
  bytes_of, cast_slice, try_cast_slice, try_cast_slice_mut, try_from_bytes, try_from_bytes_mut,
  Pod, PodCastError,
};
use std::ops::Deref;

pub unsafe trait BufferContents: Send + Sync + 'static {
  /// Converts an immutable reference to `Self` to an immutable byte slice.
  fn as_bytes(&self) -> &[u8];

  /// Converts an immutable byte slice into an immutable reference to `Self`.
  fn from_bytes(bytes: &[u8]) -> Result<&Self, PodCastError>;

  /// Converts a mutable byte slice into a mutable reference to `Self`.
  fn from_bytes_mut(bytes: &mut [u8]) -> Result<&mut Self, PodCastError>;

  /// Returns the size of an element of the type.
  fn size_of_element() -> u64;
}

unsafe impl<T> BufferContents for T
where
  T: Pod + Send + Sync,
{
  #[inline]
  fn as_bytes(&self) -> &[u8] {
    bytes_of(self)
  }

  #[inline]
  fn from_bytes(bytes: &[u8]) -> Result<&T, PodCastError> {
    try_from_bytes(bytes)
  }

  #[inline]
  fn from_bytes_mut(bytes: &mut [u8]) -> Result<&mut T, PodCastError> {
    try_from_bytes_mut(bytes)
  }

  #[inline]
  fn size_of_element() -> u64 {
    1
  }
}

unsafe impl<T> BufferContents for [T]
where
  T: Pod + Send + Sync,
{
  #[inline]
  fn as_bytes(&self) -> &[u8] {
    cast_slice(self)
  }

  #[inline]
  fn from_bytes(bytes: &[u8]) -> Result<&[T], PodCastError> {
    try_cast_slice(bytes)
  }

  #[inline]
  fn from_bytes_mut(bytes: &mut [u8]) -> Result<&mut [T], PodCastError> {
    try_cast_slice_mut(bytes)
  }

  #[inline]
  fn size_of_element() -> u64 {
    std::mem::size_of::<T>() as u64
  }
}

bitflags::bitflags! {
  pub struct BufferUsage: u32 {
    const TRANSFER_SRC = 0b1;
    const TRANSFER_DST = 0b10;
    const UNIFORM_TEXEL_BUFFER = 0b100;
    const STORAGE_TEXEL_BUFFER = 0b1000;
    const UNIFORM_BUFFER = 0b1_0000;
    const STORAGE_BUFFER = 0b10_0000;
    const INDEX_BUFFER = 0b100_0000;
    const VERTEX_BUFFER = 0b1000_0000;
    const INDIRECT_BUFFER = 0b1_0000_0000;
  }
}

#[derive(Clone, Debug, Default)]
pub struct Buffer {
  pub(super) handle: Option<usize>,
}
impl Buffer {
  pub fn new(usage: BufferUsage, size: usize) -> Self {
    unsafe {
      if let Some(device) = crate::device::RENDER_DEVICE.as_ref() {
        device.create_buffer_raw(usage, size)
      } else {
        Self { handle: None }
      }
    }
  }

  pub fn map<T: BufferContents + Pod, F: Fn(&mut T)>(&self, f: F) {
    unsafe {
      if let Some(device) = crate::device::RENDER_DEVICE.as_ref() {
        device.map_buffer(&self, f);
      }
    }
  }
}

pub struct VertexBuffer {
  pub(super) buffer: Buffer,
}
impl VertexBuffer {
  pub fn new<B: BufferContents + Pod>(data: B) -> Self {
    unsafe {
      if let Some(device) = crate::device::RENDER_DEVICE.as_ref() {
        Self {
          buffer: device.create_buffer(BufferUsage::VERTEX_BUFFER, data),
        }
      } else {
        Self {
          buffer: Buffer::default(),
        }
      }
    }
  }
}
impl Deref for VertexBuffer {
  type Target = Buffer;
  fn deref(&self) -> &Self::Target {
    &self.buffer
  }
}

pub enum IndexFormat {
  U16,
  U32,
}

pub struct IndexBuffer {
  pub(super) buffer: Buffer,
  pub(super) format: IndexFormat,
}
impl IndexBuffer {
  pub fn new<B: BufferContents + Pod>(data: B) -> Self {
    unsafe {
      if let Some(device) = crate::device::RENDER_DEVICE.as_ref() {
        Self {
          buffer: device.create_buffer(BufferUsage::INDEX_BUFFER, data),
          format: IndexFormat::U32,
        }
      } else {
        Self {
          buffer: Buffer::default(),
          format: IndexFormat::U32,
        }
      }
    }
  }
}
impl Deref for IndexBuffer {
  type Target = Buffer;
  fn deref(&self) -> &Self::Target {
    &self.buffer
  }
}

pub struct UniformBuffer<T> {
  pub(crate) buffer: Buffer,
  pub data: T,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable> UniformBuffer<T> {
  pub fn new() -> Self {
    let data = T::zeroed();
    Self {
      buffer: Buffer::default(),
      data,
    }
  }
}
