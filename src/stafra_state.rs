use futures::Future;
use std::num::NonZeroU32;
use winit::window::Window;
use std::convert::TryInto;
use std::pin::Pin;
use std::task::Context;
use super::dummy_waker;
use wgpu::Adapter;

pub struct BoardDimensions
{
    pub width:  u32,
    pub height: u32
}

pub struct StafraState
{
    surface:     wgpu::Surface,
    device:      wgpu::Device,
    queue:       wgpu::Queue,
    sc_desc:     wgpu::SwapChainDescriptor,
    swap_chain:  wgpu::SwapChain,

    board_size:       BoardDimensions,
    final_state_mips: u32,

    render_state_pipeline:            wgpu::RenderPipeline,
    clear_4_corners_pipeline:         wgpu::ComputePipeline,
    initial_state_transform_pipeline: wgpu::ComputePipeline,
    final_state_transform_pipeline:   wgpu::ComputePipeline,
    clear_stability_pipeline:         wgpu::ComputePipeline,
    next_step_pipeline:               wgpu::ComputePipeline,
    generate_mip_pipeline:            wgpu::ComputePipeline,

    render_state_bind_group:      wgpu::BindGroup,
    clear_4_corners_bind_group:   wgpu::BindGroup,
    next_step_bind_group_a:       wgpu::BindGroup,
    next_step_bind_group_b:       wgpu::BindGroup,
    final_transform_bind_group_a: wgpu::BindGroup,
    final_transform_bind_group_b: wgpu::BindGroup,
    clear_stability_bind_group:   wgpu::BindGroup,
    generate_mip_bind_groups:     Vec<wgpu::BindGroup>,

    #[allow(dead_code)]
    current_board:     wgpu::Texture,
    #[allow(dead_code)]
    next_board:        wgpu::Texture,
    #[allow(dead_code)]
    current_stability: wgpu::Texture,
    #[allow(dead_code)]
    next_stability:    wgpu::Texture,
    #[allow(dead_code)]
    final_state:       wgpu::Texture,

    #[allow(dead_code)]
    render_state_sampler: wgpu::Sampler,
}

impl StafraState
{
    pub async fn new(window: &Window, board_size: BoardDimensions) -> Self
    {
        let window_size = window.inner_size();

        let wgpu_instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe{wgpu_instance.create_surface(window)};

        let adapter = wgpu_instance.request_adapter(&wgpu::RequestAdapterOptions
        {
            power_preference:   wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor
        {
            features: wgpu::Features::default(),
            limits:   wgpu::Limits::default(),
            label:    None,
        },
        None).await.unwrap();

        device.on_uncaptured_error(|error|
        {
            println!("Wgpu error: {}", error);
            //web_sys::console::log_1(&format!("Wgpu error: {}", error).into());
        });

        let swapchain_format = adapter.get_swap_chain_preferred_format(&surface).unwrap();
        let sc_desc = wgpu::SwapChainDescriptor
        {
            usage:        wgpu::TextureUsage::RENDER_ATTACHMENT,
            format:       swapchain_format,
            width:        window_size.width,
            height:       window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let render_state_vs_module = device.create_shader_module(&wgpu::include_spirv!("../target/shaders/render_state_vs.spv"));
        let render_state_fs_module = device.create_shader_module(&wgpu::include_spirv!("../target/shaders/render_state_fs.spv"));

        let clear_4_corners_module = device.create_shader_module(&wgpu::include_spirv!("../target/shaders/clear_4_corners.spv"));

        let initial_state_transform_module = device.create_shader_module(&wgpu::include_spirv!("../target/shaders/initial_state_transform.spv"));

        let final_state_transform_module   = device.create_shader_module(&wgpu::include_spirv!("../target/shaders/final_state_transform.spv"));
        let clear_stability_module         = device.create_shader_module(&wgpu::include_spirv!("../target/shaders/clear_stability.spv"));

        let next_step_module = device.create_shader_module(&wgpu::include_spirv!("../target/shaders/next_step.spv"));

        let generate_mip_module = device.create_shader_module(&wgpu::include_spirv!("../target/shaders/final_state_generate_next_mip.spv"));

        macro_rules! initial_texture_binding
        {
            ($bd:literal) =>
            {
                wgpu::BindGroupLayoutEntry
                {
                    binding:    $bd,
                    visibility: wgpu::ShaderStage::COMPUTE,
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

        macro_rules! render_texture_binding
        {
            ($bd:literal) =>
            {
                wgpu::BindGroupLayoutEntry
                {
                    binding:    $bd,
                    visibility: wgpu::ShaderStage::FRAGMENT,
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

        macro_rules! render_sampler_binding
        {
            ($bd:literal) =>
            {
                wgpu::BindGroupLayoutEntry
                {
                    binding:    $bd,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty:         wgpu::BindingType::Sampler
                    {
                        filtering:  true,
                        comparison: false,
                    },
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
                    visibility: wgpu::ShaderStage::COMPUTE,
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
                    visibility: wgpu::ShaderStage::COMPUTE,
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
                    visibility: wgpu::ShaderStage::COMPUTE,
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
                    visibility: wgpu::ShaderStage::COMPUTE,
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

        let render_state_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries:
            &[
                render_texture_binding!(0),
                render_sampler_binding!(1)
            ]
        });

        let clear_4_corners_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries:
            &[
                board_image_binding!(0)
            ]
        });

        let initial_state_transform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries:
            &[
                initial_texture_binding!(0),
                board_image_binding!(1)
            ]
        });

        let final_state_transform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label:   None,
            entries:
            &[
                board_texture_binding!(0),
                final_image_mip_binding!(1)
            ]
        });

        let clear_stability_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries:
            &[
                board_image_binding!(0),
            ]
        });

        let next_step_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries:
            &[
                board_texture_binding!(0),
                board_texture_binding!(1),

                board_image_binding!(2),
                board_image_binding!(3),
            ]
        });

        let generate_mip_bind_group_latout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries:
            &[
                final_texture_mip_binding!(0),
                final_image_mip_binding!(1),
            ]
        });

        let render_state_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&render_state_bind_group_layout],
            push_constant_ranges: &[],
        });

        let clear_4_corners_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&clear_4_corners_bind_group_layout],
            push_constant_ranges: &[]
        });

        let initial_state_transform_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&initial_state_transform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let final_state_transform_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&final_state_transform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let clear_stability_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&clear_stability_bind_group_layout],
            push_constant_ranges: &[],
        });

        let next_step_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&next_step_bind_group_layout],
            push_constant_ranges: &[],
        });

        let generate_mip_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&generate_mip_bind_group_latout],
            push_constant_ranges: &[],
        });

        let render_state_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor
        {
            label: None,
            layout: Some(&render_state_pipeline_layout),

            vertex: wgpu::VertexState
            {
                module: &render_state_vs_module,
                entry_point: "main",
                buffers: &[],
            },

            fragment: Some(wgpu::FragmentState
            {
                module: &render_state_fs_module,
                entry_point: "main",
                targets:
                &[
                    wgpu::ColorTargetState
                    {
                        format:     swapchain_format,
                        blend:      None,
                        write_mask: wgpu::ColorWrite::ALL,
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
                clamp_depth:        false,
                polygon_mode:       wgpu::PolygonMode::Fill,
                conservative:       false
            },

            multisample: wgpu::MultisampleState::default(),
        });

        let clear_4_corners_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
        {
            label:       None,
            layout:      Some(&clear_4_corners_pipeline_layout),
            module:      &clear_4_corners_module,
            entry_point: "main"
        });

        let initial_state_transform_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
        {
            label:       None,
            layout:      Some(&initial_state_transform_pipeline_layout),
            module:      &initial_state_transform_module,
            entry_point: "main"
        });

        let final_state_transform_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
        {
            label:       None,
            layout:      Some(&final_state_transform_pipeline_layout),
            module:      &final_state_transform_module,
            entry_point: "main"
        });

        let clear_stability_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
        {
            label:       None,
            layout:      Some(&clear_stability_pipeline_layout),
            module:      &clear_stability_module,
            entry_point: "main"
        });

        let next_step_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
        {
            label:       None,
            layout:      Some(&next_step_pipeline_layout),
            module:      &next_step_module,
            entry_point: "main"
        });

        let generate_mip_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
        {
            label:       None,
            layout:      Some(&generate_mip_pipeline_layout),
            module:      &generate_mip_module,
            entry_point: "main"
        });

        let board_texture_descriptor = wgpu::TextureDescriptor
        {
            label: None,
            size:  wgpu::Extent3d
            {
                width:                 (board_size.width  + 1) / 2,
                height:                (board_size.height + 1) / 2,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count:    1,
            dimension:       wgpu::TextureDimension::D2,
            format:          wgpu::TextureFormat::R32Uint,
            usage:           wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::STORAGE
        };

        let final_state_mips = (std::cmp::max(board_size.width, board_size.height) as f32).log2().ceil() as u32;
        let final_state_texture_descriptor = wgpu::TextureDescriptor
        {
            label: None,
            size:  wgpu::Extent3d
            {
                width:                 (board_size.width  + 1) / 2,
                height:                (board_size.height + 1) / 2,
                depth_or_array_layers: 1
            },
            mip_level_count: final_state_mips,
            sample_count:    1,
            dimension:       wgpu::TextureDimension::D2,
            format:          wgpu::TextureFormat::Rgba8Unorm,
            usage:           wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::STORAGE | wgpu::TextureUsage::COPY_SRC
        };

        let current_board     = device.create_texture(&board_texture_descriptor);
        let next_board        = device.create_texture(&board_texture_descriptor);
        let current_stability = device.create_texture(&board_texture_descriptor);
        let next_stability    = device.create_texture(&board_texture_descriptor);
        let final_state       = device.create_texture(&final_state_texture_descriptor);

        let board_view_descriptor = wgpu::TextureViewDescriptor
        {
            label:             None,
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
            label:             None,
            format:            Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension:         Some(wgpu::TextureViewDimension::D2),
            aspect:            wgpu::TextureAspect::All,
            base_mip_level:    0,
            mip_level_count:   NonZeroU32::new(final_state_mips),
            base_array_layer:  0,
            array_layer_count: None
        };

        let current_board_view     = current_board.create_view(&board_view_descriptor);
        let next_board_view        = next_board.create_view(&board_view_descriptor);
        let current_stability_view = current_stability.create_view(&board_view_descriptor);
        let next_stability_view    = next_stability.create_view(&board_view_descriptor);
        let final_state_view       = final_state.create_view(&final_state_view_descriptor);

        let mut final_state_mip_views = Vec::with_capacity(final_state_mips as usize);
        for i in 0..final_state_mips
        {
            final_state_mip_views.push(final_state.create_view(&wgpu::TextureViewDescriptor
            {
                label:             None,
                format:            Some(wgpu::TextureFormat::Rgba8Unorm),
                dimension:         Some(wgpu::TextureViewDimension::D2),
                aspect:            wgpu::TextureAspect::All,
                base_mip_level:    i,
                mip_level_count:   NonZeroU32::new(1),
                base_array_layer:  0,
                array_layer_count: None
            }));
        }

        let render_state_sampler = device.create_sampler(&wgpu::SamplerDescriptor
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
        });

        let render_state_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &render_state_bind_group_layout,
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
                    resource: wgpu::BindingResource::Sampler(&render_state_sampler),
                }
            ]
        });

        let clear_4_corners_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &clear_4_corners_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&current_board_view),
                },
            ]
        });

        let next_step_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &next_step_bind_group_layout,
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
            ]
        });

        let next_step_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &next_step_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&next_board_view),
                },

                wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&next_stability_view),
                },

                wgpu::BindGroupEntry
                {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&current_board_view),
                },

                wgpu::BindGroupEntry
                {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&current_stability_view),
                },
            ]
        });

        let final_transform_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label:   None,
            layout:  &final_state_transform_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&next_stability_view),
                },

                wgpu::BindGroupEntry
                {
                    binding:  1,
                    resource: wgpu::BindingResource::TextureView(&final_state_mip_views[0]),
                }
            ]
        });

        let final_transform_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label:   None,
            layout:  &final_state_transform_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&current_stability_view),
                },

                wgpu::BindGroupEntry
                {
                    binding:  1,
                    resource: wgpu::BindingResource::TextureView(&final_state_mip_views[0]),
                }
            ]
        });

        let clear_stability_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &clear_stability_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&current_stability_view),
                },
            ]
        });

        let mut generate_mip_bind_groups = Vec::with_capacity(final_state_mips as usize - 1);
        for i in 0..(final_state_mips - 1)
        {
            generate_mip_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor
            {
                label: None,
                layout: &generate_mip_bind_group_latout,
                entries:
                &[
                    wgpu::BindGroupEntry
                    {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&final_state_mip_views[i as usize]),
                    },

                    wgpu::BindGroupEntry
                    {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&final_state_mip_views[i as usize + 1]),
                    },
                ]
            }));
        }

        Self
        {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,

            final_state_mips,
            board_size,

            render_state_pipeline,
            clear_4_corners_pipeline,
            initial_state_transform_pipeline,
            final_state_transform_pipeline,
            next_step_pipeline,
            clear_stability_pipeline,
            generate_mip_pipeline,
            generate_mip_bind_groups,

            render_state_bind_group,
            clear_4_corners_bind_group,
            next_step_bind_group_a,
            next_step_bind_group_b,
            final_transform_bind_group_a,
            final_transform_bind_group_b,
            clear_stability_bind_group,

            current_board,
            next_board,
            current_stability,
            next_stability,
            final_state,

            render_state_sampler
        }
    }

    pub fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>)
    {
        self.sc_desc.width  = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain     = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    pub fn get_png_data(&self) -> Result<Vec<u8>, &str>
    {
        let row_alignment = 256 as usize;
        let row_pitch     = ((self.board_size.width as usize * std::mem::size_of::<f32>()) + (row_alignment - 1)) & (!(row_alignment - 1));

        let board_buffer = self.device.create_buffer(&wgpu::BufferDescriptor
        {
            label:              None,
            size:               (row_pitch * self.board_size.height as usize) as u64,
            usage:              wgpu::BufferUsage::MAP_READ | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false
        });

        let mut buffer_copy_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});

        buffer_copy_encoder.copy_texture_to_buffer(wgpu::ImageCopyTexture
        {
            texture:   &self.final_state,
            mip_level: 0,
            origin:    wgpu::Origin3d
            {
                x: 0,
                y: 0,
                z: 0
            }
        },
        wgpu::ImageCopyBuffer
        {
            buffer: &board_buffer,
            layout: wgpu::ImageDataLayout
            {
                offset:         0,
                bytes_per_row:  std::num::NonZeroU32::new(row_pitch as u32),
                rows_per_image: std::num::NonZeroU32::new(self.board_size.height)
            }
        },
        wgpu::Extent3d
        {
            width:                 self.board_size.width,
            height:                self.board_size.height,
            depth_or_array_layers: 1
        });

        self.queue.submit(std::iter::once(buffer_copy_encoder.finish()));

        let png_buffer_slice      = board_buffer.slice(..);
        let mut buffer_map_future = png_buffer_slice.map_async(wgpu::MapMode::Read);

        let mut image_array = Vec::with_capacity((self.board_size.width * self.board_size.height * 4) as usize);

        let waker = dummy_waker::dummy_waker();
        let mut context = Context::from_waker(&waker);

        let performance = web_sys::window().unwrap().performance().unwrap();
        let start_time  = performance.now();
        loop
        {
            //Busy wait because asyncs in winit don't work
            let pinned_future = Pin::new(&mut buffer_map_future);
            match Future::poll(pinned_future, &mut context)
            {
                std::task::Poll::Ready(_) => break,
                std::task::Poll::Pending =>
                {
                    let current_time = performance.now();
                    if current_time - start_time > 2000.0 //Max busy wait time is 2 seconds
                    {
                        board_buffer.unmap();
                        return Err("Timeout");
                    }
                }
            }
        }

        let png_buffer_view = png_buffer_slice.get_mapped_range();
        for row_chunk in png_buffer_view.chunks(row_pitch)
        {
            for texel in row_chunk.chunks(4)
            {
                let val = f32::from_le_bytes(texel[0..4].try_into().unwrap());

                image_array.push((val * 255.0) as u8); //Red
                image_array.push(0u8);                 //Green
                image_array.push((val * 255.0) as u8); //Blue
                image_array.push(255u8);               //Alpha
            }
        }

        board_buffer.unmap();

        Ok(image_array)
    }

    pub fn reset_board(&self)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});

        let thread_groups_x = std::cmp::max((self.board_size.width + 1) / (2 * 32), 1u32);
        let thread_groups_y = std::cmp::max((self.board_size.width + 1) / (2 * 32), 1u32);

        {
            let mut reset_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});

            reset_pass.set_pipeline(&self.clear_4_corners_pipeline);
            reset_pass.set_bind_group(0, &self.clear_4_corners_bind_group, &[]);
            reset_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }

        {
            let mut clear_stability_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});

            clear_stability_pass.set_pipeline(&self.clear_stability_pipeline);
            clear_stability_pass.set_bind_group(0, &self.clear_stability_bind_group, &[]);
            clear_stability_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn update(&mut self)
    {
        let thread_groups_x = std::cmp::max((self.board_size.width + 1) / (2 * 32), 1u32);
        let thread_groups_y = std::cmp::max((self.board_size.width + 1) / (2 * 32), 1u32);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});

        {
            let mut reset_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});

            reset_pass.set_pipeline(&self.next_step_pipeline);
            reset_pass.set_bind_group(0, &self.next_step_bind_group_a, &[]);
            reset_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }

        {
            let mut final_transform_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});

            final_transform_pass.set_pipeline(&self.final_state_transform_pipeline);
            final_transform_pass.set_bind_group(0, &self.final_transform_bind_group_a, &[]);
            final_transform_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }

        let mut thread_groups_mip_x = std::cmp::max(thread_groups_x / 2, 1u32);
        let mut thread_groups_mip_y = std::cmp::max(thread_groups_y / 2, 1u32);
        for i in 0..(self.final_state_mips - 1)
        {
            let mut generate_mip_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});

            generate_mip_pass.set_pipeline(&self.generate_mip_pipeline);
            generate_mip_pass.set_bind_group(0, &self.generate_mip_bind_groups[i as usize], &[]);
            generate_mip_pass.dispatch(thread_groups_mip_x, thread_groups_mip_y, 1);

            thread_groups_mip_x = std::cmp::max(thread_groups_mip_x / 2, 1u32);
            thread_groups_mip_y = std::cmp::max(thread_groups_mip_y / 2, 1u32);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        std::mem::swap(&mut self.next_step_bind_group_a,       &mut self.next_step_bind_group_b);
        std::mem::swap(&mut self.final_transform_bind_group_a, &mut self.final_transform_bind_group_b);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SwapChainError>
    {
        let frame = self.swap_chain.get_current_frame()?.output;

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor
            {
                label: None,
                color_attachments:
                &[
                    wgpu::RenderPassColorAttachment
                    {
                        view:           &frame.view,
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

            render_pass.set_pipeline(&self.render_state_pipeline);
            render_pass.set_bind_group(0, &self.render_state_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
