use std::num::NonZeroU32;
use super::stafra_static_state::StafraStaticState;
use super::stafra_static_state_bindings::StafraStaticBindings;
use crate::stafra_initial_state_bindings::StafraInitialStateBindings;

//Board bindings for the main stafra state. Re-initialized every time after resizing the board
pub struct StafraBoardBindings
{
    board_width:  u32,
    board_height: u32,

    main_render_state_bind_group:  wgpu::BindGroup,
    clear_default_bind_group:      wgpu::BindGroup,
    initial_transform_bind_group:  wgpu::BindGroup,
    filter_restriction_bind_group: wgpu::BindGroup,
    next_step_bind_group_a:        wgpu::BindGroup,
    next_step_bind_group_b:        wgpu::BindGroup,
    final_transform_bind_group_a:  wgpu::BindGroup,
    final_transform_bind_group_b:  wgpu::BindGroup,
    clear_stability_bind_group_a:  wgpu::BindGroup,
    clear_stability_bind_group_b:  wgpu::BindGroup,
    clear_restriction_bind_group:  wgpu::BindGroup,
    generate_mip_bind_groups:      Vec<wgpu::BindGroup>,

    #[allow(dead_code)]
    current_board:     wgpu::Texture,
    #[allow(dead_code)]
    next_board:        wgpu::Texture,
    #[allow(dead_code)]
    current_stability: wgpu::Texture,
    #[allow(dead_code)]
    next_stability:    wgpu::Texture,

    restriction:       wgpu::Texture,
    final_state:       wgpu::Texture,
    video_frame:       wgpu::Texture,
}

pub struct ImageBuffer
{
    pub image_buffer: wgpu::Buffer,
    pub raw_width:    u32,
    pub raw_height:   u32,
    pub row_pitch:    usize
}

pub struct ImageData
{
    pub pixel_data:   Vec<u8>,
    pub image_width:  u32,
    pub image_height: u32
}

impl StafraBoardBindings
{
    pub fn new(device: &wgpu::Device, static_state: &StafraStaticState, static_bindings: &StafraStaticBindings, initial_state_bindings: &StafraInitialStateBindings, width: u32, height: u32) -> Self
    {
        assert!((width  + 1).is_power_of_two());
        assert!((height + 1).is_power_of_two());

        let board_width  = width;
        let board_height = height;

        let board_texture_descriptor = wgpu::TextureDescriptor
        {
            label: Some("Board texture"),
            size:  wgpu::Extent3d
            {
                width:                 (board_width  + 1) / 2,
                height:                (board_height + 1) / 2,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count:    1,
            dimension:       wgpu::TextureDimension::D2,
            format:          wgpu::TextureFormat::R32Uint,
            usage:           wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING
        };

        let final_state_mips = (std::cmp::max(board_width, board_height) as f32).log2().ceil() as u32;
        let final_state_texture_descriptor = wgpu::TextureDescriptor
        {
            label: Some("Final state texture"),
            size:  wgpu::Extent3d
            {
                width:                 (board_width  + 1) / 2,
                height:                (board_height + 1) / 2,
                depth_or_array_layers: 1
            },
            mip_level_count: final_state_mips,
            sample_count:    1,
            dimension:       wgpu::TextureDimension::D2,
            format:          wgpu::TextureFormat::Rgba8Unorm,
            usage:           wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC
        };

        let video_frame_texture_descriptor = wgpu::TextureDescriptor
        {
            label: Some("Video frame texture"),
            size:  wgpu::Extent3d
            {
                width:                 1024,
                height:                1024,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count:    1,
            dimension:       wgpu::TextureDimension::D2,
            format:          wgpu::TextureFormat::Bgra8Unorm,
            usage:           wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC
        };

        let current_board      = device.create_texture(&board_texture_descriptor);
        let next_board         = device.create_texture(&board_texture_descriptor);
        let current_stability  = device.create_texture(&board_texture_descriptor);
        let next_stability     = device.create_texture(&board_texture_descriptor);
        let restriction        = device.create_texture(&board_texture_descriptor);
        let final_state        = device.create_texture(&final_state_texture_descriptor);
        let video_frame        = device.create_texture(&video_frame_texture_descriptor);

        let board_view_descriptor = wgpu::TextureViewDescriptor
        {
            label:             Some("Board view"),
            format:            Some(wgpu::TextureFormat::R32Uint),
            dimension:         Some(wgpu::TextureViewDimension::D2),
            aspect:            wgpu::TextureAspect::All,
            base_mip_level:    0,
            mip_level_count:   None,
            base_array_layer:  0,
            array_layer_count: None
        };

        let final_state_view_descriptor = wgpu::TextureViewDescriptor
        {
            label:             Some("Final state view"),
            format:            Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension:         Some(wgpu::TextureViewDimension::D2),
            aspect:            wgpu::TextureAspect::All,
            base_mip_level:    0,
            mip_level_count:   NonZeroU32::new(final_state_mips),
            base_array_layer:  0,
            array_layer_count: None
        };

        let initial_state_view     = initial_state_bindings.create_initial_state_view();
        let current_board_view     = current_board.create_view(&board_view_descriptor);
        let next_board_view        = next_board.create_view(&board_view_descriptor);
        let current_stability_view = current_stability.create_view(&board_view_descriptor);
        let next_stability_view    = next_stability.create_view(&board_view_descriptor);
        let restriction_view       = restriction.create_view(&board_view_descriptor);
        let final_state_view       = final_state.create_view(&final_state_view_descriptor);

        let mut final_state_mip_views = Vec::with_capacity(final_state_mips as usize);
        for i in 0..final_state_mips
        {
            final_state_mip_views.push(final_state.create_view(&wgpu::TextureViewDescriptor
            {
                label:             Some(&format!("Final state mip {} view", i)),
                format:            Some(wgpu::TextureFormat::Rgba8Unorm),
                dimension:         Some(wgpu::TextureViewDimension::D2),
                aspect:            wgpu::TextureAspect::All,
                base_mip_level:    i,
                mip_level_count:   NonZeroU32::new(1),
                base_array_layer:  0,
                array_layer_count: None
            }));
        }

        let main_render_state_bind_group = static_state.create_render_main_bind_group(device, &final_state_view);

        let clear_default_bind_group     = static_state.create_clear_default_bind_group(device,     &next_board_view);
        let clear_stability_bind_group_a = static_state.create_clear_stability_bind_group(device,   &current_stability_view);
        let clear_stability_bind_group_b = static_state.create_clear_stability_bind_group(device,   &next_stability_view);
        let clear_restriction_bind_group = static_state.create_clear_restriction_bind_group(device, &restriction_view);

        let initial_transform_bind_group  = static_state.create_initial_transform_bind_group(device, &initial_state_view, &next_board_view);
        let filter_restriction_bind_group = static_state.create_filter_restriction_bind_group(device, &next_board_view, &restriction_view, &current_board_view);

        let next_step_bind_group_a = static_state.create_next_step_bind_group(device, &current_board_view, &current_stability_view, &next_board_view, &next_stability_view, &restriction_view, static_bindings.click_rule_buffer_binding());
        let next_step_bind_group_b = static_state.create_next_step_bind_group(device, &next_board_view, &next_stability_view, &current_board_view, &current_stability_view, &restriction_view, static_bindings.click_rule_buffer_binding());

        let final_transform_bind_group_a = static_state.create_final_transform_bind_group(device, &current_stability_view, &final_state_mip_views[0], static_bindings.spawn_buffer_binding());
        let final_transform_bind_group_b = static_state.create_final_transform_bind_group(device, &next_stability_view,    &final_state_mip_views[0], static_bindings.spawn_buffer_binding());

        let mut generate_mip_bind_groups = Vec::with_capacity(final_state_mips as usize - 1);
        for i in 0..(final_state_mips - 1)
        {
            generate_mip_bind_groups.push(static_state.create_generate_mip_bind_group(device, &final_state_mip_views[i as usize], &final_state_mip_views[i as usize + 1]));
        }

        Self
        {
            board_width,
            board_height,

            main_render_state_bind_group,
            clear_default_bind_group,
            initial_transform_bind_group,
            filter_restriction_bind_group,
            next_step_bind_group_a,
            next_step_bind_group_b,
            final_transform_bind_group_a,
            final_transform_bind_group_b,
            clear_stability_bind_group_a,
            clear_stability_bind_group_b,
            clear_restriction_bind_group,
            generate_mip_bind_groups,

            current_board,
            next_board,
            current_stability,
            next_stability,
            restriction,
            final_state,
            video_frame
        }
    }

    pub fn filter_restriction(&self, encoder: &mut wgpu::CommandEncoder, static_state: &StafraStaticState)
    {
        let thread_groups_x = std::cmp::max((self.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_height + 1) / (2 * 16), 1u32);

        {
            let mut filter_restriction_pass = static_state.create_filter_restriction_pass(encoder);
            filter_restriction_pass.set_bind_group(0, &self.filter_restriction_bind_group, &[]);
            filter_restriction_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }
    }

    pub fn clear_stability(&self, encoder: &mut wgpu::CommandEncoder, static_state: &StafraStaticState)
    {
        let thread_groups_x = std::cmp::max((self.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_height + 1) / (2 * 16), 1u32);

        {
            let mut clear_stability_pass_a = static_state.create_clear_stability_pass(encoder);
            clear_stability_pass_a.set_bind_group(0, &self.clear_stability_bind_group_a, &[]);
            clear_stability_pass_a.dispatch(thread_groups_x, thread_groups_y, 1);
        }

        {
            let mut clear_stability_pass_b = static_state.create_clear_stability_pass(encoder);
            clear_stability_pass_b.set_bind_group(0, &self.clear_stability_bind_group_b, &[]);
            clear_stability_pass_b.dispatch(thread_groups_x, thread_groups_y, 1);
        }
    }

    pub fn clear_restriction(&mut self, encoder: &mut wgpu::CommandEncoder, static_state: &StafraStaticState)
    {
        let thread_groups_x = std::cmp::max((self.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_height + 1) / (2 * 16), 1u32);

        {
            let mut clear_restriction_pass = static_state.create_clear_restriction_pass(encoder);
            clear_restriction_pass.set_bind_group(0, &self.clear_restriction_bind_group, &[]);
            clear_restriction_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }
    }

    pub fn reset_board_standard_corners(&mut self, encoder: &mut wgpu::CommandEncoder, static_state: &StafraStaticState)
    {
        let thread_groups_x = std::cmp::max((self.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_height + 1) / (2 * 16), 1u32);

        {
            let mut reset_pass = static_state.create_clear_4_corners_pass(encoder);
            reset_pass.set_bind_group(0, &self.clear_default_bind_group, &[]);
            reset_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }
    }

    pub fn reset_board_standard_edges(&mut self, encoder: &mut wgpu::CommandEncoder, static_state: &StafraStaticState)
    {
        let thread_groups_x = std::cmp::max((self.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_height + 1) / (2 * 16), 1u32);

        {
            let mut reset_pass = static_state.create_clear_4_sides_pass(encoder);
            reset_pass.set_bind_group(0, &self.clear_default_bind_group, &[]);
            reset_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }
    }

    pub fn reset_board_standard_center(&mut self, encoder: &mut wgpu::CommandEncoder, static_state: &StafraStaticState)
    {
        let thread_groups_x = std::cmp::max((self.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_height + 1) / (2 * 16), 1u32);

        {
            let mut reset_pass = static_state.create_clear_center_pass(encoder);
            reset_pass.set_bind_group(0, &self.clear_default_bind_group, &[]);
            reset_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }
    }

    pub fn reset_board_custom(&mut self, encoder: &mut wgpu::CommandEncoder, static_state: &StafraStaticState)
    {
        let thread_groups_x = std::cmp::max((self.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_height + 1) / (2 * 16), 1u32);

        {
            let mut initial_transform_pass = static_state.create_initial_transform_pass(encoder);
            initial_transform_pass.set_bind_group(0, &self.initial_transform_bind_group, &[]);
            initial_transform_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }
    }

    pub fn draw_main_state(&self, encoder: &mut wgpu::CommandEncoder, main_frame_view: &wgpu::TextureView, static_state: &StafraStaticState)
    {
        let mut main_render_pass = static_state.create_main_draw_pass(encoder, main_frame_view);
        main_render_pass.set_bind_group(0, &self.main_render_state_bind_group, &[]);
        main_render_pass.draw(0..3, 0..1);
    }

    pub fn render_video_frame(&mut self, encoder: &mut wgpu::CommandEncoder, static_state: &StafraStaticState)
    {
        let video_frame_view = self.video_frame.create_view(&wgpu::TextureViewDescriptor::default());

        let mut main_render_pass = static_state.create_main_draw_pass(encoder, &video_frame_view);
        main_render_pass.set_bind_group(0, &self.main_render_state_bind_group, &[]);
        main_render_pass.draw(0..3, 0..1);
    }

    pub fn calc_next_frame(&self, encoder: &mut wgpu::CommandEncoder, static_state: &StafraStaticState, frame_number: u32)
    {
        let thread_groups_x = std::cmp::max((self.board_width  + 1) / (2 * 8), 1u32);
        let thread_groups_y = std::cmp::max((self.board_height + 1) / (2 * 8), 1u32);

        {
            let mut next_step_pass = static_state.create_next_step_pass(encoder);

            let bind_group = if frame_number % 2 == 0 {&self.next_step_bind_group_a} else {&self.next_step_bind_group_b};
            next_step_pass.set_bind_group(0, bind_group, &[]);
            next_step_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }
    }

    pub fn generate_final_image(&self, encoder: &mut wgpu::CommandEncoder, static_state: &StafraStaticState, frame_number: u32)
    {
        let thread_groups_x = std::cmp::max((self.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_height + 1) / (2 * 16), 1u32);

        {
            let mut final_transform_pass = static_state.create_generate_final_image_pass(encoder);

            let bind_group = if frame_number % 2 == 0 {&self.final_transform_bind_group_a} else {&self.final_transform_bind_group_b};
            final_transform_pass.set_bind_group(0, bind_group, &[]);
            final_transform_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }

        let mut thread_groups_mip_x = std::cmp::max(thread_groups_x / 2, 1u32);
        let mut thread_groups_mip_y = std::cmp::max(thread_groups_y / 2, 1u32);
        for gen_mip_bind_group in &self.generate_mip_bind_groups
        {
            let mut generate_mip_pass = static_state.create_generate_mip_pass(encoder);
            generate_mip_pass.set_bind_group(0, &gen_mip_bind_group, &[]);
            generate_mip_pass.dispatch(thread_groups_mip_x, thread_groups_mip_y, 1);

            thread_groups_mip_x = std::cmp::max(thread_groups_mip_x / 2, 1u32);
            thread_groups_mip_y = std::cmp::max(thread_groups_mip_y / 2, 1u32);
        }
    }

    pub fn initial_transform_restriction(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, initial_restriction_view: &wgpu::TextureView, static_state: &StafraStaticState)
    {
        let restriction_view = self.restriction.create_view(&wgpu::TextureViewDescriptor
        {
            label:             Some("Restriction view"),
            format:            Some(wgpu::TextureFormat::R32Uint),
            dimension:         Some(wgpu::TextureViewDimension::D2),
            aspect:            wgpu::TextureAspect::All,
            base_mip_level:    0,
            mip_level_count:   None,
            base_array_layer:  0,
            array_layer_count: None
        });

        let initial_restriction_transform_bind_group = static_state.create_initial_restriction_transform_bind_group(device, &initial_restriction_view, &restriction_view);

        let thread_groups_x = std::cmp::max((self.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_height + 1) / (2 * 16), 1u32);

        {
            let mut initial_restriction_transform_pass = static_state.create_initial_restriction_transform_pass(encoder);
            initial_restriction_transform_pass.set_bind_group(0, &initial_restriction_transform_bind_group, &[]);
            initial_restriction_transform_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }
    }

    pub fn create_image_data_buffer(&self, device: &wgpu::Device, buffer_copy_encoder: &mut wgpu::CommandEncoder) -> ImageBuffer
    {
        let data_width  = (self.board_width  + 1) / 2;
        let data_height = (self.board_height + 1) / 2;

        let row_alignment = 256 as usize;
        let row_pitch     = ((data_width as usize * std::mem::size_of::<f32>()) + (row_alignment - 1)) & (!(row_alignment - 1));

        let image_buffer = device.create_buffer(&wgpu::BufferDescriptor
        {
            label: Some("PNG image staging buffer"),
            size: (row_pitch * data_height as usize) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        buffer_copy_encoder.copy_texture_to_buffer(wgpu::ImageCopyTexture
        {
            texture:   &self.final_state,
            mip_level: 0,
            origin:    wgpu::Origin3d
            {
               x: 0,
               y: 0,
               z: 0
            },
            aspect: wgpu::TextureAspect::All
        },
        wgpu::ImageCopyBuffer
        {
            buffer: &image_buffer,
            layout: wgpu::ImageDataLayout
            {
               offset:         0,
               bytes_per_row:  std::num::NonZeroU32::new(row_pitch as u32),
               rows_per_image: std::num::NonZeroU32::new(data_height)
            }
        },
        wgpu::Extent3d
        {
            width:                 data_width,
            height:                data_height,
            depth_or_array_layers: 1
        });

        ImageBuffer
        {
            image_buffer,
            raw_width: data_width,
            raw_height: data_height,
            row_pitch
        }
    }

    pub fn create_video_frame_data_buffer(&self, device: &wgpu::Device, buffer_copy_encoder: &mut wgpu::CommandEncoder) -> ImageBuffer
    {
        let video_frame_width  = 1024;
        let video_frame_height = 1024;

        let row_alignment = 256 as usize;
        let row_pitch     = (video_frame_width * 4 + row_alignment - 1) & (!(row_alignment - 1));

        let video_frame_buffer = device.create_buffer(&wgpu::BufferDescriptor
        {
            label:              Some("Video frame staging buffer"),
            size:               (row_pitch * video_frame_height as usize) as u64,
            usage:              wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        buffer_copy_encoder.copy_texture_to_buffer(wgpu::ImageCopyTexture
        {
            texture:   &self.video_frame,
            mip_level: 0,
            origin:    wgpu::Origin3d
            {
               x: 0,
               y: 0,
               z: 0
            },
            aspect: wgpu::TextureAspect::All
        },
        wgpu::ImageCopyBuffer
        {
            buffer: &video_frame_buffer,
            layout: wgpu::ImageDataLayout
            {
               offset:         0,
               bytes_per_row:  std::num::NonZeroU32::new(row_pitch as u32),
               rows_per_image: std::num::NonZeroU32::new(video_frame_height)
            }
        },
        wgpu::Extent3d
        {
            width:                 video_frame_width as u32,
            height:                video_frame_height as u32,
            depth_or_array_layers: 1
        });

        ImageBuffer
        {
            image_buffer: video_frame_buffer,
            raw_width:    video_frame_width  as u32,
            raw_height:   video_frame_height as u32,
            row_pitch
        }
    }

    pub fn get_image_buffer_mapped_data(&self, image_buffer: &wgpu::Buffer, raw_width: u32, raw_height: u32, row_pitch: usize) -> ImageData
    {
        let image_width  = raw_width  * 2 - 1;
        let image_height = raw_height * 2 - 1;

        let padded_width  = image_width  + 1;
        let padded_height = image_height + 1;
        let mut image_array = vec![0u8; (padded_width * padded_height * 4) as usize];
        {
            let image_buffer_view = image_buffer.slice(..).get_mapped_range();
            for (quad_row_index, quad_row_chunk) in image_buffer_view.chunks(row_pitch).enumerate()
            {
                let image_row_index = (quad_row_index * 2) as u32;
                for (quad_column_index, quad_bytes) in quad_row_chunk.chunks(4).enumerate()
                {
                    let image_column_index = (quad_column_index * 2) as u32;
                    if image_column_index >= image_width
                    {
                        //Can get there if row_pitch is big enough
                        break;
                    }

                    //Decode the quad
                    let top_left     = quad_bytes[0] as f32;
                    let top_right    = quad_bytes[1] as f32;
                    let bottom_left  = quad_bytes[2] as f32;
                    let bottom_right = quad_bytes[3] as f32;

                    let top_left_texel_start     = (((image_row_index + 0) * image_width + image_column_index + 0) * 4) as usize;
                    let top_right_texel_start    = (((image_row_index + 0) * image_width + image_column_index + 1) * 4) as usize;
                    let bottom_left_texel_start  = (((image_row_index + 1) * image_width + image_column_index + 0) * 4) as usize;
                    let bottom_right_texel_start = (((image_row_index + 1) * image_width + image_column_index + 1) * 4) as usize;

                    image_array[bottom_right_texel_start + 0] = (bottom_right * 255.0) as u8; //Red
                    image_array[bottom_right_texel_start + 1] = 0u8;                          //Green
                    image_array[bottom_right_texel_start + 2] = (bottom_right * 255.0) as u8; //Blue
                    image_array[bottom_right_texel_start + 3] = 255u8;                        //Alpha

                    image_array[top_right_texel_start + 0] = (top_right * 255.0) as u8; //Red
                    image_array[top_right_texel_start + 1] = 0u8;                       //Green
                    image_array[top_right_texel_start + 2] = (top_right * 255.0) as u8; //Blue
                    image_array[top_right_texel_start + 3] = 255u8;                     //Alpha

                    image_array[bottom_left_texel_start + 0] = (bottom_left * 255.0) as u8; //Red
                    image_array[bottom_left_texel_start + 1] = 0u8;                         //Green
                    image_array[bottom_left_texel_start + 2] = (bottom_left * 255.0) as u8; //Blue
                    image_array[bottom_left_texel_start + 3] = 255u8;                       //Alpha

                    image_array[top_left_texel_start + 0] = (top_left * 255.0) as u8; //Red
                    image_array[top_left_texel_start + 1] = 0u8;                      //Green
                    image_array[top_left_texel_start + 2] = (top_left * 255.0) as u8; //Blue
                    image_array[top_left_texel_start + 3] = 255u8;                    //Alpha
                }
            }
        }

        image_array.truncate((image_width * image_height * 4) as usize);
        ImageData
        {
            pixel_data: image_array,
            image_width,
            image_height
        }
    }

    pub fn get_video_frame_buffer_mapped_data(&self, video_frame_buffer: &wgpu::Buffer, raw_width: u32, raw_height: u32) -> ImageData
    {
        //Because video_frame_width is a multiple of 256, row pitch is equal to width * 4.
        //We can copy the image contents directly to the buffer, which is A LOT faster
        let video_frame_image_array = video_frame_buffer.slice(..).get_mapped_range().to_vec();
        ImageData
        {
            pixel_data:   video_frame_image_array,
            image_width:  raw_width  as u32,
            image_height: raw_height as u32
        }
    }
}
