use std::num::NonZeroU32;

pub struct StafraInitialStateBindings
{
    initial_state_width:  u32,
    initial_state_height: u32,

    initial_state_tex: wgpu::Texture
}

impl StafraInitialStateBindings
{
    pub fn new(device: &wgpu::Device, board_width: u32, board_height: u32) -> Self
    {
        let initial_state_texture_descriptor = wgpu::TextureDescriptor
        {
            label: None,
            size:  wgpu::Extent3d
            {
                width:                 board_width,
                height:                board_height,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count:    1,
            dimension:       wgpu::TextureDimension::D2,
            format:          wgpu::TextureFormat::Rgba8Unorm,
            usage:           wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
        };

        let initial_state_tex = device.create_texture(&initial_state_texture_descriptor);
        Self
        {
            initial_state_width:  board_width,
            initial_state_height: board_height,

            initial_state_tex
        }
    }

    pub fn create_initial_state_view(&self) -> wgpu::TextureView
    {
        self.initial_state_tex.create_view(&wgpu::TextureViewDescriptor
        {
            label:             None,
            format:            Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension:         Some(wgpu::TextureViewDimension::D2),
            aspect:            wgpu::TextureAspect::All,
            base_mip_level:    0,
            mip_level_count:   None,
            base_array_layer:  0,
            array_layer_count: None
        })
    }

    pub fn upload_texture(&self, queue: &wgpu::Queue, image_array: Vec<u8>, width: u32, height: u32)
    {
        queue.write_texture(wgpu::ImageCopyTexture
        {
            texture:   &self.initial_state_tex,
            mip_level: 0,
            origin:    wgpu::Origin3d::ZERO,
            aspect:    wgpu::TextureAspect::All
        },
        image_array.as_slice(),
        wgpu::ImageDataLayout
        {
            offset:         0,
            bytes_per_row:  NonZeroU32::new(width * 4),
            rows_per_image: NonZeroU32::new(height)
        },
        wgpu::Extent3d
        {
            width:                 self.initial_state_width,
            height:                self.initial_state_height,
            depth_or_array_layers: 1
        });
    }
}