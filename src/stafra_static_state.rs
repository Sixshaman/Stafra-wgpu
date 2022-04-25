use std::num::NonZeroU64;

//Binding layout and pipeline library for the main stafra state. Contains all meta-state that needs to be initialized only once
pub struct StafraStaticState
{
    main_render_bind_group_layout:                   wgpu::BindGroupLayout,
    click_rule_render_bind_group_layout:             wgpu::BindGroupLayout,
    clear_default_bind_group_layout:                 wgpu::BindGroupLayout,
    clear_stability_bind_group_layout:               wgpu::BindGroupLayout,
    clear_restriction_bind_group_layout:             wgpu::BindGroupLayout,
    initial_state_transform_bind_group_layout:       wgpu::BindGroupLayout,
    initial_restriction_transform_bind_group_layout: wgpu::BindGroupLayout,
    filter_restriction_bind_group_layout:            wgpu::BindGroupLayout,
    next_step_bind_group_layout:                     wgpu::BindGroupLayout,
    bake_click_rule_bind_group_layout:               wgpu::BindGroupLayout,
    final_state_transform_bind_group_layout:         wgpu::BindGroupLayout,
    generate_mip_bind_group_layout:                  wgpu::BindGroupLayout,

    main_render_pipeline:                   wgpu::RenderPipeline,
    click_rule_render_pipeline:             wgpu::RenderPipeline,
    clear_4_corners_pipeline:               wgpu::ComputePipeline,
    clear_4_sides_pipeline:                 wgpu::ComputePipeline,
    clear_center_pipeline:                  wgpu::ComputePipeline,
    clear_stability_pipeline:               wgpu::ComputePipeline,
    clear_restriction_pipeline:             wgpu::ComputePipeline,
    initial_state_transform_pipeline:       wgpu::ComputePipeline,
    initial_restriction_transform_pipeline: wgpu::ComputePipeline,
    filter_restriction_pipeline:            wgpu::ComputePipeline,
    next_step_pipeline:                     wgpu::ComputePipeline,
    bake_click_rule_pipeline:               wgpu::ComputePipeline,
    final_state_transform_pipeline:         wgpu::ComputePipeline,
    generate_mip_pipeline:                  wgpu::ComputePipeline,

    #[allow(dead_code)]
    render_state_sampler: wgpu::Sampler
}

impl StafraStaticState
{
    pub fn new(device: &wgpu::Device, swapchain_format: wgpu::TextureFormat) -> Self
    {
        let main_render_bind_group_layout       = create_main_render_bind_group_layout(device);
        let click_rule_render_bind_group_layout = create_click_rule_render_bind_group_layout(device);

        let clear_default_bind_group_layout     = create_clear_default_bind_group_layout(device);
        let clear_stability_bind_group_layout   = create_clear_stability_bind_group_layout(device);
        let clear_restriction_bind_group_layout = create_clear_restriction_bind_group_layout(device);

        let initial_state_transform_bind_group_layout       = create_initial_state_transform_bind_group_layout(device);
        let initial_restriction_transform_bind_group_layout = create_initial_restriction_transform_bind_group_layout(device);
        let filter_restriction_bind_group_layout            = create_filter_restriction_bind_group_layout(device);

        let next_step_bind_group_layout       = create_next_step_bind_group_layout(device);
        let bake_click_rule_bind_group_layout = create_bake_click_rule_bind_group_layout(device);

        let final_state_transform_bind_group_layout = create_final_state_transform_bind_group_layout(device);
        let generate_mip_bind_group_layout          = create_generate_mip_bind_group_layout(device);


        let clear_default_pipeline_layout = create_clear_default_pipeline_layout(device, &clear_default_bind_group_layout);

        let main_render_pipeline                   = create_main_render_pipeline(device, &main_render_bind_group_layout, swapchain_format);
        let click_rule_render_pipeline             = create_click_rule_render_pipeline(device, &click_rule_render_bind_group_layout, swapchain_format);
        let clear_4_corners_pipeline               = create_clear_4_corners_pipeline(device, &clear_default_pipeline_layout);
        let clear_4_sides_pipeline                 = create_clear_4_sides_pipeline(device, &clear_default_pipeline_layout);
        let clear_center_pipeline                  = create_clear_center_pipeline(device, &clear_default_pipeline_layout);
        let clear_stability_pipeline               = create_clear_stability_pipeline(device, &clear_stability_bind_group_layout);
        let clear_restriction_pipeline             = create_clear_restriction_pipeline(device, &clear_restriction_bind_group_layout);
        let initial_state_transform_pipeline       = create_initial_state_transform_pipeline(device, &initial_state_transform_bind_group_layout);
        let initial_restriction_transform_pipeline = create_initial_restriction_transform_pipeline(device, &initial_restriction_transform_bind_group_layout);
        let filter_restriction_pipeline            = create_filter_restriction_pipeline(device, &filter_restriction_bind_group_layout);
        let next_step_pipeline                     = create_next_step_pipeline(device, &next_step_bind_group_layout);
        let bake_click_rule_pipeline               = create_bake_click_rule_pipeline(device, &bake_click_rule_bind_group_layout);
        let final_state_transform_pipeline         = create_final_state_transform_pipeline(device, &final_state_transform_bind_group_layout);
        let generate_mip_pipeline                  = create_generate_mip_pipeline(device, &generate_mip_bind_group_layout);

        Self
        {
            main_render_bind_group_layout,
            click_rule_render_bind_group_layout,
            clear_default_bind_group_layout,
            clear_stability_bind_group_layout,
            clear_restriction_bind_group_layout,
            initial_state_transform_bind_group_layout,
            initial_restriction_transform_bind_group_layout,
            filter_restriction_bind_group_layout,
            next_step_bind_group_layout,
            bake_click_rule_bind_group_layout,
            final_state_transform_bind_group_layout,
            generate_mip_bind_group_layout,

            main_render_pipeline,
            click_rule_render_pipeline,
            clear_4_corners_pipeline,
            clear_4_sides_pipeline,
            clear_center_pipeline,
            clear_stability_pipeline,
            clear_restriction_pipeline,
            initial_state_transform_pipeline,
            initial_restriction_transform_pipeline,
            filter_restriction_pipeline,
            next_step_pipeline,
            bake_click_rule_pipeline,
            final_state_transform_pipeline,
            generate_mip_pipeline,

            render_state_sampler: create_render_state_sampler(device)
        }
    }

    pub fn create_render_main_bind_group(&self, device: &wgpu::Device, final_state_view: &wgpu::TextureView) -> wgpu::BindGroup
    {
        device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &self.main_render_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding:  0,
                    resource: wgpu::BindingResource::TextureView(&final_state_view),
                },

                wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.render_state_sampler),
                }
            ]
        })
    }

    pub fn create_render_click_rule_bind_group(&self, device: &wgpu::Device, click_rule_texture_view: &wgpu::TextureView, click_rule_render_flags_buffer: &wgpu::Buffer) -> wgpu::BindGroup
    {
        device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &self.click_rule_render_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding:  0,
                    resource: wgpu::BindingResource::TextureView(&click_rule_texture_view),
                },

                wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(click_rule_render_flags_buffer.as_entire_buffer_binding()),
                }
            ]
        })
    }

    pub fn create_clear_default_bind_group(&self, device: &wgpu::Device, board_view: &wgpu::TextureView) -> wgpu::BindGroup
    {
        device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &self.clear_default_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&board_view),
                },
            ]
        })
    }

    pub fn create_clear_stability_bind_group(&self, device: &wgpu::Device, stability_view: &wgpu::TextureView) -> wgpu::BindGroup
    {
        device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &self.clear_stability_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&stability_view),
                },
            ]
        })
    }

    pub fn create_clear_restriction_bind_group(&self, device: &wgpu::Device, restriction_view: &wgpu::TextureView) -> wgpu::BindGroup
    {
        device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &self.clear_restriction_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&restriction_view),
                },
            ]
        })
    }

    pub fn create_initial_transform_bind_group(&self, device: &wgpu::Device, initial_state_view: &wgpu::TextureView, board_view: &wgpu::TextureView) -> wgpu::BindGroup
    {
        device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &self.initial_state_transform_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&initial_state_view)
                },

                wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&board_view)
                }
            ]
        })
    }

    pub fn create_initial_restriction_transform_bind_group(&self, device: &wgpu::Device, initial_restriction_view: &wgpu::TextureView, restriction_view: &wgpu::TextureView) -> wgpu::BindGroup
    {
        device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &self.initial_restriction_transform_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&initial_restriction_view)
                },

                wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&restriction_view)
                }
            ]
        })
    }

    pub fn create_filter_restriction_bind_group(&self, device: &wgpu::Device, in_board_view: &wgpu::TextureView, restriction_view: &wgpu::TextureView, out_board_view: &wgpu::TextureView) -> wgpu::BindGroup
    {
        device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &self.filter_restriction_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&in_board_view)
                },

                wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&restriction_view)
                },

                wgpu::BindGroupEntry
                {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&out_board_view)
                }
            ]
        })
    }

    pub fn create_next_step_bind_group(&self, device: &wgpu::Device, current_board_view: &wgpu::TextureView, current_stability_view: &wgpu::TextureView, next_board_view: &wgpu::TextureView, next_stability_view: &wgpu::TextureView, restriction_view: &wgpu::TextureView, click_rule_buffer_binding: wgpu::BufferBinding) -> wgpu::BindGroup
    {
        device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &self.next_step_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&current_board_view),
                },

                wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&current_stability_view),
                },

                wgpu::BindGroupEntry
                {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&next_board_view),
                },

                wgpu::BindGroupEntry
                {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&next_stability_view),
                },

                wgpu::BindGroupEntry
                {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&restriction_view),
                },

                wgpu::BindGroupEntry
                {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(click_rule_buffer_binding)
                }
            ]
        })
    }

    pub fn create_bake_click_rule_bind_group(&self, device: &wgpu::Device, click_rule_texture_view: &wgpu::TextureView, click_rule_buffer: &wgpu::Buffer) -> wgpu::BindGroup
    {
        device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &self.bake_click_rule_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&click_rule_texture_view),
                },

                wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(click_rule_buffer.as_entire_buffer_binding())
                }
            ]
        })
    }

    pub fn create_final_transform_bind_group(&self, device: &wgpu::Device, stability_view: &wgpu::TextureView, final_state_view: &wgpu::TextureView, spawn_buffer_binding: wgpu::BufferBinding) -> wgpu::BindGroup
    {
        device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label:   None,
            layout:  &self.final_state_transform_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&stability_view),
                },

                wgpu::BindGroupEntry
                {
                    binding:  1,
                    resource: wgpu::BindingResource::TextureView(&final_state_view),
                },

                wgpu::BindGroupEntry
                {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(spawn_buffer_binding)
                }
            ]
        })
    }

    pub fn create_generate_mip_bind_group(&self, device: &wgpu::Device, current_mip: &wgpu::TextureView, next_mip: &wgpu::TextureView) -> wgpu::BindGroup
    {
        device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &self.generate_mip_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(current_mip),
                },

                wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(next_mip),
                },
            ]
        })
    }

    pub fn create_main_draw_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, main_frame_view: &'a wgpu::TextureView) -> wgpu::RenderPass<'a>
    {
        let mut main_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor
        {
            label: None,
            color_attachments:
            &[
                wgpu::RenderPassColorAttachment
                {
                    view:           &main_frame_view,
                    resolve_target: None,
                    ops:            wgpu::Operations
                    {
                        load:  wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: true,
                    },
                }
            ],

            depth_stencil_attachment: None,
        });

        main_render_pass.set_pipeline(&self.main_render_pipeline);
        main_render_pass
    }

    pub fn create_click_rule_draw_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder, click_rule_frame_view: &'a wgpu::TextureView) -> wgpu::RenderPass<'a>
    {
        let mut click_rule_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor
        {
            label: None,
            color_attachments:
            &[
                wgpu::RenderPassColorAttachment
                {
                    view: &click_rule_frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations
                    {
                        load:  wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    }
                }
            ],

            depth_stencil_attachment: None
        });

        click_rule_render_pass.set_pipeline(&self.click_rule_render_pipeline);
        click_rule_render_pass
    }

    pub fn create_clear_stability_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::ComputePass<'a>
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
        pass.set_pipeline(&self.clear_stability_pipeline);
        pass
    }

    pub fn create_clear_restriction_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::ComputePass<'a>
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
        pass.set_pipeline(&self.clear_restriction_pipeline);
        pass
    }

    pub fn create_clear_4_corners_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::ComputePass<'a>
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
        pass.set_pipeline(&self.clear_4_corners_pipeline);
        pass
    }

    pub fn create_clear_4_sides_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::ComputePass<'a>
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
        pass.set_pipeline(&self.clear_4_sides_pipeline);
        pass
    }

    pub fn create_clear_center_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::ComputePass<'a>
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
        pass.set_pipeline(&self.clear_center_pipeline);
        pass
    }

    pub fn create_initial_transform_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::ComputePass<'a>
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
        pass.set_pipeline(&self.initial_state_transform_pipeline);
        pass
    }

    pub fn create_initial_restriction_transform_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::ComputePass<'a>
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
        pass.set_pipeline(&self.initial_restriction_transform_pipeline);
        pass
    }

    pub fn create_filter_restriction_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::ComputePass<'a>
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
        pass.set_pipeline(&self.filter_restriction_pipeline);
        pass
    }

    pub fn create_next_step_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::ComputePass<'a>
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
        pass.set_pipeline(&self.next_step_pipeline);
        pass
    }

    pub fn create_bake_click_rule_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::ComputePass<'a>
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
        pass.set_pipeline(&self.bake_click_rule_pipeline);
        pass
    }

    pub fn create_generate_final_image_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::ComputePass<'a>
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
        pass.set_pipeline(&self.final_state_transform_pipeline);
        pass
    }

    pub fn create_generate_mip_pass<'a>(&'a self, encoder: &'a mut wgpu::CommandEncoder) -> wgpu::ComputePass<'a>
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
        pass.set_pipeline(&self.generate_mip_pipeline);
        pass
    }
}

macro_rules! initial_texture_binding
{
    ($bd:literal) =>
    {
        wgpu::BindGroupLayoutEntry
        {
            binding:    $bd,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty:         wgpu::BindingType::Texture
            {
                sample_type:    wgpu::TextureSampleType::Float {filterable: true},
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled:   false,
            },
            count: None
        }
    }
}

macro_rules! main_render_texture_binding
{
    ($bd:literal) =>
    {
        wgpu::BindGroupLayoutEntry
        {
            binding:    $bd,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty:         wgpu::BindingType::Texture
            {
                sample_type:    wgpu::TextureSampleType::Float {filterable: true},
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled:   false,
            },
            count: None
        }
    }
}

macro_rules! click_rule_render_texture_binding
{
    ($bd:literal) =>
    {
        wgpu::BindGroupLayoutEntry
        {
            binding:    $bd,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty:         wgpu::BindingType::Texture
            {
                sample_type:    wgpu::TextureSampleType::Uint,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled:   false,
            },
            count: None
        }
    }
}

macro_rules! main_render_sampler_binding
{
    ($bd:literal) =>
    {
        wgpu::BindGroupLayoutEntry
        {
            binding:    $bd,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty:         wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None
        }
    }
}

macro_rules! board_texture_binding
{
    ($bd:literal) =>
    {
        wgpu::BindGroupLayoutEntry
        {
            binding:    $bd,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty:         wgpu::BindingType::Texture
            {
                sample_type:    wgpu::TextureSampleType::Uint,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled:   false,
            },
            count: None
        }
    }
}

macro_rules! board_image_binding
{
    ($bd:literal) =>
    {
        wgpu::BindGroupLayoutEntry
        {
            binding:    $bd,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty:         wgpu::BindingType::StorageTexture
            {
                access:         wgpu::StorageTextureAccess::WriteOnly,
                format:         wgpu::TextureFormat::R32Uint,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None
        }
    }
}

macro_rules! final_texture_mip_binding
{
    ($bd:literal) =>
    {
        wgpu::BindGroupLayoutEntry
        {
            binding:    $bd,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty:         wgpu::BindingType::Texture
            {
                sample_type:    wgpu::TextureSampleType::Float {filterable: true},
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled:   false,
            },
            count: None
        }
    }
}

macro_rules! final_image_mip_binding
{
    ($bd:literal) =>
    {
        wgpu::BindGroupLayoutEntry
        {
            binding:    $bd,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty:         wgpu::BindingType::StorageTexture
            {
                access:         wgpu::StorageTextureAccess::WriteOnly,
                format:         wgpu::TextureFormat::Rgba8Unorm,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None
        }
    }
}

macro_rules! spawn_data_uniform_binding
{
    ($bd:literal) =>
    {
        wgpu::BindGroupLayoutEntry
        {
            binding:    $bd,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty:         wgpu::BindingType::Buffer
            {
                ty:                 wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size:   NonZeroU64::new(2 * std::mem::size_of::<u32>() as u64)
            },
            count: None
        }
    }
}

macro_rules! click_rule_uniform_binding
{
    ($bd:literal) =>
    {
        wgpu::BindGroupLayoutEntry
        {
            binding:    $bd,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty:         wgpu::BindingType::Buffer
            {
                ty:                 wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size:   NonZeroU64::new(4 * std::mem::size_of::<i32>() as u64 + 32 * 32 * 2 * std::mem::size_of::<i32>() as u64)
            },
            count: None
        }
    }
}

macro_rules! click_rule_storage_binding
{
    ($bd:literal) =>
    {
        wgpu::BindGroupLayoutEntry
        {
            binding:    $bd,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty:         wgpu::BindingType::Buffer
            {
                ty: wgpu::BufferBindingType::Storage
                {
                    read_only: false
                },
                has_dynamic_offset: false,
                min_binding_size:   NonZeroU64::new(4 * std::mem::size_of::<i32>() as u64 + 32 * 32 * 2 * std::mem::size_of::<i32>() as u64)
            },
            count: None
        }
    }
}

macro_rules! click_rule_render_flags_binding
{
    ($bd:literal) =>
    {
        wgpu::BindGroupLayoutEntry
        {
            binding:    $bd,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty:         wgpu::BindingType::Buffer
            {
                ty:                 wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size:   NonZeroU64::new(std::mem::size_of::<u32>() as u64)
            },
            count: None
        }
    }
}

fn create_main_render_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
{
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
    {
        label: None,
        entries:
        &[
            main_render_texture_binding!(0),
            main_render_sampler_binding!(1)
        ]
    })
}

fn create_click_rule_render_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
{
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
    {
        label: None,
        entries:
        &[
            click_rule_render_texture_binding!(0),
            click_rule_render_flags_binding!(1)
        ]
    })
}

fn create_clear_default_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
{
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
    {
        label: None,
        entries:
        &[
            board_image_binding!(0)
        ]
    })
}

fn create_bake_click_rule_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
{
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
    {
        label: None,
        entries:
        &[
            board_texture_binding!(0),
            click_rule_storage_binding!(1)
        ]
    })
}

fn create_initial_state_transform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
{
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
    {
        label: None,
        entries:
        &[
            initial_texture_binding!(0),
            board_image_binding!(1)
        ]
    })
}

fn create_initial_restriction_transform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
{
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
    {
        label: None,
        entries:
        &[
            initial_texture_binding!(0),
            board_image_binding!(1)
        ]
    })
}

fn create_filter_restriction_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
{
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
    {
        label: None,
        entries:
        &[
            board_texture_binding!(0),
            board_texture_binding!(1),
            board_image_binding!(2)
        ]
    })
}

fn create_final_state_transform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
{
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
    {
        label: None,
        entries:
        &[
            board_texture_binding!(0),
            final_image_mip_binding!(1),
            spawn_data_uniform_binding!(2)
        ]
    })
}

fn create_clear_stability_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
{
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
    {
        label: None,
        entries:
        &[
            board_image_binding!(0),
        ]
    })
}

fn create_clear_restriction_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
{
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
    {
        label: None,
        entries:
        &[
            board_image_binding!(0),
        ]
    })
}

fn create_next_step_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
{
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
    {
        label: None,
        entries:
        &[
            board_texture_binding!(0),
            board_texture_binding!(1),

            board_image_binding!(2),
            board_image_binding!(3),

            board_texture_binding!(4),

            click_rule_uniform_binding!(5)
        ]
    })
}

fn create_generate_mip_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
{
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
    {
        label: None,
        entries:
        &[
            final_texture_mip_binding!(0),
            final_image_mip_binding!(1),
        ]
    })
}

fn create_clear_default_pipeline_layout(device: &wgpu::Device, clear_default_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::PipelineLayout
{
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
    {
        label: None,
        bind_group_layouts: &[&clear_default_bind_group_layout],
        push_constant_ranges: &[]
    })
}

fn create_main_render_pipeline(device: &wgpu::Device, main_render_bind_group_layout: &wgpu::BindGroupLayout, swapchain_format: wgpu::TextureFormat) -> wgpu::RenderPipeline
{
    let main_render_state_vs_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/render/render_state_vs.wgsl"));
    let main_render_state_fs_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/render/render_state_fs.wgsl"));

    let main_render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
    {
        label: None,
        bind_group_layouts: &[&main_render_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor
    {
        label: None,
        layout: Some(&main_render_pipeline_layout),

        vertex: wgpu::VertexState
        {
            module: &main_render_state_vs_module,
            entry_point: "main",
            buffers: &[],
        },

        fragment: Some(wgpu::FragmentState
        {
            module: &main_render_state_fs_module,
            entry_point: "main",
            targets:
            &[
                wgpu::ColorTargetState
                {
                    format:     swapchain_format,
                    blend:      None,
                    write_mask: wgpu::ColorWrites::ALL,
                }
            ],
        }),

        depth_stencil: None,

        primitive: wgpu::PrimitiveState
        {
            topology:           wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face:         wgpu::FrontFace::Cw,
            cull_mode:          None,
            unclipped_depth:    true,
            polygon_mode:       wgpu::PolygonMode::Fill,
            conservative:       false
        },

        multisample: wgpu::MultisampleState::default(),
        multiview: None
    })
}

fn create_click_rule_render_pipeline(device: &wgpu::Device, click_rule_render_bind_group_layout: &wgpu::BindGroupLayout, swapchain_format: wgpu::TextureFormat) -> wgpu::RenderPipeline
{
    let render_click_rule_vs_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/render/click_rule_render_state_vs.wgsl"));
    let render_click_rule_fs_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/render/click_rule_render_state_fs.wgsl"));

    let render_click_rule_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
    {
        label: None,
        bind_group_layouts: &[&click_rule_render_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor
    {
        label: None,
        layout: Some(&render_click_rule_pipeline_layout),

        vertex: wgpu::VertexState
        {
            module: &render_click_rule_vs_module,
            entry_point: "main",
            buffers: &[],
        },

        fragment: Some(wgpu::FragmentState
        {
            module: &render_click_rule_fs_module,
            entry_point: "main",
            targets:
            &[
                wgpu::ColorTargetState
                {
                    format:     swapchain_format,
                    blend:      None,
                    write_mask: wgpu::ColorWrites::ALL,
                }
            ],
        }),

        depth_stencil: None,

        primitive: wgpu::PrimitiveState
        {
            topology:           wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face:         wgpu::FrontFace::Cw,
            cull_mode:          None,
            unclipped_depth:    true,
            polygon_mode:       wgpu::PolygonMode::Fill,
            conservative:       false
        },

        multisample: wgpu::MultisampleState::default(),
        multiview: None
    })
}

fn create_clear_4_corners_pipeline(device: &wgpu::Device, clear_default_pipeline_layout: &wgpu::PipelineLayout) -> wgpu::ComputePipeline
{
    let clear_4_corners_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/clear_board/clear_4_corners.wgsl"));

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
    {
        label:       None,
        layout:      Some(&clear_default_pipeline_layout),
        module:      &clear_4_corners_module,
        entry_point: "main"
    })
}

fn create_clear_4_sides_pipeline(device: &wgpu::Device, clear_default_pipeline_layout: &wgpu::PipelineLayout) -> wgpu::ComputePipeline
{
    let clear_4_sides_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/clear_board/clear_4_sides.wgsl"));

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
    {
        label:       None,
        layout:      Some(&clear_default_pipeline_layout),
        module:      &clear_4_sides_module,
        entry_point: "main"
    })
}

fn create_clear_center_pipeline(device: &wgpu::Device, clear_default_pipeline_layout: &wgpu::PipelineLayout) -> wgpu::ComputePipeline
{
    let clear_center_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/clear_board/clear_center.wgsl"));

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
    {
        label:       None,
        layout:      Some(&clear_default_pipeline_layout),
        module:      &clear_center_module,
        entry_point: "main"
    })
}

fn create_clear_stability_pipeline(device: &wgpu::Device, clear_stability_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::ComputePipeline
{
    let clear_stability_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/state_transform/clear_stability.wgsl"));

    let clear_stability_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
    {
        label: None,
        bind_group_layouts: &[&clear_stability_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
    {
        label:       None,
        layout:      Some(&clear_stability_pipeline_layout),
        module:      &clear_stability_module,
        entry_point: "main"
    })
}

fn create_clear_restriction_pipeline(device: &wgpu::Device, clear_restriction_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::ComputePipeline
{
    let clear_restriction_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/state_transform/clear_restriction.wgsl"));

    let clear_restriction_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
    {
        label: None,
        bind_group_layouts: &[&clear_restriction_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
    {
        label:       None,
        layout:      Some(&clear_restriction_pipeline_layout),
        module:      &clear_restriction_module,
        entry_point: "main"
    })
}

fn create_initial_state_transform_pipeline(device: &wgpu::Device, initial_state_transform_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::ComputePipeline
{
    let initial_state_transform_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/state_transform/initial_state_transform.wgsl"));

    let initial_state_transform_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
    {
        label: None,
        bind_group_layouts: &[&initial_state_transform_bind_group_layout],
        push_constant_ranges: &[]
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
    {
        label:       None,
        layout:      Some(&initial_state_transform_pipeline_layout),
        module:      &initial_state_transform_module,
        entry_point: "main"
    })
}

fn create_initial_restriction_transform_pipeline(device: &wgpu::Device, initial_restriction_transform_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::ComputePipeline
{
    let initial_restriction_transform_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/state_transform/initial_restriction_transform.wgsl"));

    let initial_restriction_transform_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
    {
        label: None,
        bind_group_layouts: &[&initial_restriction_transform_bind_group_layout],
        push_constant_ranges: &[]
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
    {
        label:       None,
        layout:      Some(&initial_restriction_transform_pipeline_layout),
        module:      &initial_restriction_transform_module,
        entry_point: "main"
    })
}

fn create_filter_restriction_pipeline(device: &wgpu::Device, filter_restriction_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::ComputePipeline
{
    let filter_restriction_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/state_transform/filter_restriction.wgsl"));

    let filter_restriction_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
    {
        label: None,
        bind_group_layouts: &[&filter_restriction_bind_group_layout],
        push_constant_ranges: &[]
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
    {
        label:       None,
        layout:      Some(&filter_restriction_pipeline_layout),
        module:      &filter_restriction_module,
        entry_point: "main"
    })
}

fn create_next_step_pipeline(device: &wgpu::Device, next_step_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::ComputePipeline
{
    let next_step_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/next_step/next_step.wgsl"));

    let next_step_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
    {
        label: None,
        bind_group_layouts: &[&next_step_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
    {
        label:       None,
        layout:      Some(&next_step_pipeline_layout),
        module:      &next_step_module,
        entry_point: "main"
    })
}

fn create_bake_click_rule_pipeline(device: &wgpu::Device, bake_click_rule_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::ComputePipeline
{
    let bake_click_rule_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/click_rule/bake_click_rule.wgsl"));

    let bake_click_rule_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
    {
        label: None,
        bind_group_layouts: &[&bake_click_rule_bind_group_layout],
        push_constant_ranges: &[]
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
    {
        label:       None,
        layout:      Some(&bake_click_rule_pipeline_layout),
        module:      &bake_click_rule_module,
        entry_point: "main"
    })
}

fn create_final_state_transform_pipeline(device: &wgpu::Device, final_state_transform_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::ComputePipeline
{
    let final_state_transform_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/state_transform/final_state_transform.wgsl"));

    let final_state_transform_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
    {
        label: None,
        bind_group_layouts: &[&final_state_transform_bind_group_layout],
        push_constant_ranges: &[]
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
    {
        label:       None,
        layout:      Some(&final_state_transform_pipeline_layout),
        module:      &final_state_transform_module,
        entry_point: "main"
    })
}

fn create_generate_mip_pipeline(device: &wgpu::Device, generate_mip_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::ComputePipeline
{
    let generate_mip_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/mip/final_state_generate_next_mip.wgsl"));

    let generate_mip_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
    {
        label: None,
        bind_group_layouts: &[&generate_mip_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
    {
        label:       None,
        layout:      Some(&generate_mip_pipeline_layout),
        module:      &generate_mip_module,
        entry_point: "main"
    })
}

fn create_render_state_sampler(device: &wgpu::Device) -> wgpu::Sampler
{
    device.create_sampler(&wgpu::SamplerDescriptor
    {
        label:            None,
        address_mode_u:   wgpu::AddressMode::Repeat,
        address_mode_v:   wgpu::AddressMode::Repeat,
        address_mode_w:   wgpu::AddressMode::Repeat,
        mag_filter:       wgpu::FilterMode::Nearest,
        min_filter:       wgpu::FilterMode::Linear,
        mipmap_filter:    wgpu::FilterMode::Linear,
        lod_min_clamp:    0.0,
        lod_max_clamp:    0.0,
        compare:          None,
        anisotropy_clamp: None,
        border_color:     None
    })
}