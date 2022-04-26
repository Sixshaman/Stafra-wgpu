use std::num::NonZeroU32;
use super::stafra_static_state::StafraStaticState;

//All bindings for the main stafra state that only need to be initialized once: click rule info, spawn buffer data
pub struct StafraStaticBindings
{
    render_click_rule_bind_group: wgpu::BindGroup,
    bake_click_rule_bind_group:   wgpu::BindGroup,

    click_rule_texture: wgpu::Texture,
    click_rule_buffer:  wgpu::Buffer,

    click_rule_render_flags:        u32,
    click_rule_render_flags_buffer: wgpu::Buffer,

    spawn_period:      u32,
    spawn_data_flags:  u32,
    spawn_data_buffer: wgpu::Buffer
}

impl StafraStaticBindings
{
    pub fn new(device: &wgpu::Device, static_state: &StafraStaticState) -> Self
    {
        let click_rule_size = 32;

        let click_rule_texture_descriptor = wgpu::TextureDescriptor
        {
            label: Some("Click rule texture"),
            size:  wgpu::Extent3d
            {
                width:                 click_rule_size,
                height:                click_rule_size,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count:    1,
            dimension:       wgpu::TextureDimension::D2,
            format:          wgpu::TextureFormat::R32Uint,
            usage:           wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST
        };

        let click_rule_buffer_descriptor = wgpu::BufferDescriptor
        {
            label:              Some("Click rule buffer"),
            size:               4 * std::mem::size_of::<i32>() as u64 + ((click_rule_size * click_rule_size * 2) as u64) * std::mem::size_of::<i32>() as u64,
            usage:              wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        };

        let click_rule_render_flags_buffer_descriptor = wgpu::BufferDescriptor
        {
            label:              Some("Click rule flags buffer"),
            size:               std::mem::size_of::<u32>() as u64,
            usage:              wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        };

        let spawn_data_buffer_descriptor = wgpu::BufferDescriptor
        {
            label:              Some("Spawn data buffer"),
            size:               2 * std::mem::size_of::<u32>() as u64,
            usage:              wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        };

        let click_rule_texture             = device.create_texture(&click_rule_texture_descriptor);
        let click_rule_buffer              = device.create_buffer(&click_rule_buffer_descriptor);
        let click_rule_render_flags_buffer = device.create_buffer(&click_rule_render_flags_buffer_descriptor);
        let spawn_data_buffer              = device.create_buffer(&spawn_data_buffer_descriptor);

        let click_rule_texture_view_descriptor = wgpu::TextureViewDescriptor
        {
            label:             Some("Click rule view"),
            format:            Some(wgpu::TextureFormat::R32Uint),
            dimension:         Some(wgpu::TextureViewDimension::D2),
            aspect:            wgpu::TextureAspect::All,
            base_mip_level:    0,
            mip_level_count:   None,
            base_array_layer:  0,
            array_layer_count: None
        };

        let click_rule_texture_view = click_rule_texture.create_view(&click_rule_texture_view_descriptor);

        let render_click_rule_bind_group = static_state.create_render_click_rule_bind_group(device, &click_rule_texture_view, &click_rule_render_flags_buffer);
        let bake_click_rule_bind_group   = static_state.create_bake_click_rule_bind_group(device,   &click_rule_texture_view, &click_rule_buffer);

        Self
        {
            render_click_rule_bind_group,
            bake_click_rule_bind_group,

            click_rule_texture,
            click_rule_buffer,

            click_rule_render_flags: 0,
            click_rule_render_flags_buffer,

            spawn_period:     u32::MAX,
            spawn_data_flags: 0,
            spawn_data_buffer
        }
    }

    pub fn set_click_rule_grid_enabled(&mut self, enable: bool)
    {
        let render_grid_flag = 0x01;
        if enable
        {
            self.click_rule_render_flags |= render_grid_flag;
        }
        else
        {
            self.click_rule_render_flags &= !render_grid_flag;
        }

        let dirty_flag = 0x80000000;
        self.click_rule_render_flags |= dirty_flag;
    }

    pub fn set_click_rule_read_only(&mut self, is_read_only: bool)
    {
        let click_rule_read_only_flag = 0x02;
        if is_read_only
        {
            self.click_rule_render_flags |= click_rule_read_only_flag;
        }
        else
        {
            self.click_rule_render_flags &= !click_rule_read_only_flag;
        }

        let dirty_flag = 0x80000000;
        self.click_rule_render_flags |= dirty_flag;
    }

    pub fn set_spawn_period(&mut self, spawn_period: u32)
    {
        self.spawn_period = spawn_period;

        let dirty_flag = 0x80000000;
        self.spawn_data_flags |= dirty_flag;
    }

    pub fn set_smooth_transform_enabled(&mut self, enable: bool)
    {
        let smooth_transform_enable_flag = 0x01;
        if enable
        {
            self.spawn_data_flags |= smooth_transform_enable_flag;
        }
        else
        {
            self.spawn_data_flags &= !smooth_transform_enable_flag;
        }

        let dirty_flag = 0x80000000;
        self.spawn_data_flags |= dirty_flag;
    }

    pub fn update_draw_state(&mut self, queue: &wgpu::Queue)
    {
        let dirty_flag = 0x80000000;

        if self.click_rule_render_flags & dirty_flag != 0
        {
            let buffer_data = self.click_rule_render_flags.to_le_bytes();
            queue.write_buffer(&self.click_rule_render_flags_buffer, 0, &buffer_data);

            self.click_rule_render_flags &= !dirty_flag;
        }

        if self.spawn_data_flags & dirty_flag != 0
        {
            let elem_size = std::mem::size_of::<u32>();

            let mut buffer_data = [0u8; std::mem::size_of::<u32>() * 2];
            buffer_data[elem_size * 0..elem_size * 1].copy_from_slice(&self.spawn_period.to_le_bytes());
            buffer_data[elem_size * 1..elem_size * 2].copy_from_slice(&self.spawn_data_flags.to_le_bytes());

            queue.write_buffer(&self.spawn_data_buffer, 0, &buffer_data);

            self.spawn_data_flags &= !dirty_flag;
        }
    }

    pub fn click_rule_buffer_binding(&self) -> wgpu::BufferBinding
    {
        self.click_rule_buffer.as_entire_buffer_binding()
    }

    pub fn spawn_buffer_binding(&self) -> wgpu::BufferBinding
    {
        self.spawn_data_buffer.as_entire_buffer_binding()
    }

    pub fn reset_click_rule(&mut self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, static_state: &StafraStaticState, click_rule_data: &[u8; 32 * 32])
    {
        let click_rule_size = 32;

        let mut click_rule_byte_data = vec![0u8; click_rule_data.len() * std::mem::size_of::<u32>()];
        for (index, click_rule_cell) in click_rule_data.into_iter().enumerate()
        {
            let click_rule_byte_data_index = index * std::mem::size_of::<u32>();

            let click_rule_cell_bytes = click_rule_cell.to_le_bytes();
            for (byte_index, byte_value) in click_rule_cell_bytes.into_iter().enumerate()
            {
                click_rule_byte_data[click_rule_byte_data_index + byte_index] = byte_value;
            }
        }

        queue.write_texture(wgpu::ImageCopyTexture
        {
            texture:   &self.click_rule_texture,
            mip_level: 0,
            origin:    wgpu::Origin3d::ZERO,
            aspect:    wgpu::TextureAspect::All
        },
        click_rule_byte_data.as_slice(),
        wgpu::ImageDataLayout
        {
            offset:         0,
            bytes_per_row:  NonZeroU32::new(click_rule_size as u32 * std::mem::size_of::<u32>() as u32),
            rows_per_image: NonZeroU32::new(click_rule_size as u32)
        },
        wgpu::Extent3d
        {
            width:                 click_rule_size,
            height:                click_rule_size,
            depth_or_array_layers: 1
        });

        self.bake_click_rule(queue, encoder, static_state);
    }

    pub fn draw_click_rule(&self, encoder: &mut wgpu::CommandEncoder, click_rule_frame_view: &wgpu::TextureView, static_state: &StafraStaticState)
    {
        let mut click_rule_render_pass = static_state.create_click_rule_draw_pass(encoder, &click_rule_frame_view);
        click_rule_render_pass.set_bind_group(0, &self.render_click_rule_bind_group, &[]);
        click_rule_render_pass.draw(0..3, 0..1);
    }

    fn bake_click_rule(&self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, static_state: &StafraStaticState)
    {
        let click_rule_size = 32;
        let workgroup_size  = 8;

        let thread_group_size = (click_rule_size / workgroup_size) as u32;

        let click_rule_buffer_size = 4 * std::mem::size_of::<u32>() + click_rule_size * click_rule_size * 2 * std::mem::size_of::<i32>();
        let click_rule_buffer_data = vec![0u8; click_rule_buffer_size];

        queue.write_buffer(&self.click_rule_buffer, 0, click_rule_buffer_data.as_slice());

        {
            let mut bake_click_rule_pass = static_state.create_bake_click_rule_pass(encoder);
            bake_click_rule_pass.set_bind_group(0, &self.bake_click_rule_bind_group, &[]);
            bake_click_rule_pass.dispatch(thread_group_size, thread_group_size, 1);
        }
    }
}