use super::device::*;

pub struct Texture2D {
  handle: wgpu::Texture,
  view: wgpu::TextureView,
  size: wgpu::Extent3d,
  format: wgpu::TextureFormat,
}

impl Texture2D {
  pub fn new(
    label: Option<&str>,
    size: wgpu::Extent3d,
    mip_level_count: u32,
    sample_count: u32,
    dimension: wgpu::TextureDimension,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
  ) -> Self {
    let handle = context().create_texture(
      label,
      size,
      mip_level_count,
      sample_count,
      dimension,
      format,
      usage,
    );
    let view = handle.create_view(&wgpu::TextureViewDescriptor::default());
    Self {
      handle,
      view,
      size,
      format,
    }
  }

  pub fn update(&self, data: &[u8]) {
    let x_stride = self.format.describe().block_size as u32;
    let y_stride = self.size.width * x_stride;
    context().update_texture(
      // Tells wgpu where to copy the pixel data
      wgpu::ImageCopyTexture {
        texture: &self.handle,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      // The actual pixel data
      data,
      // The layout of the texture
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: std::num::NonZeroU32::new(y_stride),
        rows_per_image: std::num::NonZeroU32::new(self.size.height),
      },
      self.size,
    );
  }

  pub fn view(&self) -> &wgpu::TextureView {
    &self.view
  }
}
