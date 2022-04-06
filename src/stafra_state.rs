use
{
    futures::Future,
    std::num::{NonZeroU32, NonZeroU64},
    std::cmp::min
};

#[cfg(not(target_arch = "wasm32"))]
use
{
    winit::window::Window
};

#[cfg(target_arch = "wasm32")]
use
{
    super::dummy_waker,
    std::task::Context,
    std::pin::Pin
};

pub enum AcquireImageResult
{
    Pending,
    NoImagesRequested,
    AcquireSuccess
    {
        pixel_data: Vec<u8>,
        width:      u32,
        height:     u32,
    }
}

struct ImageCopyRequest
{
    image_buffer:      wgpu::Buffer,
    buffer_map_future: Box<dyn Future<Output = Result<(), wgpu::BufferAsyncError>> + Unpin>,
    row_pitch:         usize
}

struct StafraBindingLayouts
{
    main_render_state_bind_group_layout:             wgpu::BindGroupLayout,
    click_rule_render_state_bind_group_layout:       wgpu::BindGroupLayout,
    clear_default_bind_group_layout:                 wgpu::BindGroupLayout,
    bake_click_rule_bind_group_layout:               wgpu::BindGroupLayout,
    initial_state_transform_bind_group_layout:       wgpu::BindGroupLayout,
    initial_restriction_transform_bind_group_layout: wgpu::BindGroupLayout,
    filter_restriction_bind_group_layout:            wgpu::BindGroupLayout,
    final_state_transform_bind_group_layout:         wgpu::BindGroupLayout,
    clear_stability_bind_group_layout:               wgpu::BindGroupLayout,
    clear_restriction_bind_group_layout:             wgpu::BindGroupLayout,
    next_step_bind_group_layout:                     wgpu::BindGroupLayout,
    generate_mip_bind_group_layout:                  wgpu::BindGroupLayout,

    #[allow(dead_code)]
    render_state_sampler: wgpu::Sampler
}

struct StafraClickRuleBindings
{
    click_rule_render_state_bind_group: wgpu::BindGroup,
    bake_click_rule_bind_group:         wgpu::BindGroup,

    click_rule_render_flags: u32,

    #[allow(dead_code)]
    click_rule_texture:             wgpu::Texture,
    #[allow(dead_code)]
    click_rule_buffer:              wgpu::Buffer,
    #[allow(dead_code)]
    click_rule_render_flags_buffer: wgpu::Buffer,
}

struct StafraInitialStateBindings
{
    #[allow(dead_code)]
    initial_state: wgpu::Texture,
}

//We recreate these each time the board size changes
struct StafraBoardBindings
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
    #[allow(dead_code)]
    restriction:       wgpu::Texture,
    #[allow(dead_code)]
    final_state:       wgpu::Texture,
    #[allow(dead_code)]
    video_frame:       wgpu::Texture,

    #[allow(dead_code)]
    spawn_data_buffer: wgpu::Buffer
}

#[derive(Copy, Clone, PartialEq)]
pub enum StandardResetBoardType
{
    Corners,
    Edges,
    Center
}

#[derive(Copy, Clone, PartialEq)]
enum ResetBoardType
{
    Standard {reset_type: StandardResetBoardType},
    Custom
}

pub struct StafraState
{
    main_surface:       wgpu::Surface,
    click_rule_surface: wgpu::Surface,
    device:             wgpu::Device,
    queue:              wgpu::Queue,

    swapchain_format: wgpu::TextureFormat,
    frame_number:     u32,

    main_render_state_pipeline:             wgpu::RenderPipeline,
    click_rule_render_state_pipeline:       wgpu::RenderPipeline,
    clear_4_corners_pipeline:               wgpu::ComputePipeline,
    clear_4_sides_pipeline:                 wgpu::ComputePipeline,
    clear_center_pipeline:                  wgpu::ComputePipeline,
    bake_click_rule_pipeline:               wgpu::ComputePipeline,
    initial_state_transform_pipeline:       wgpu::ComputePipeline,
    initial_restriction_transform_pipeline: wgpu::ComputePipeline,
    filter_restriction_pipeline:            wgpu::ComputePipeline,
    final_state_transform_pipeline:         wgpu::ComputePipeline,
    clear_stability_pipeline:               wgpu::ComputePipeline,
    clear_restriction_pipeline:             wgpu::ComputePipeline,
    next_step_pipeline:                     wgpu::ComputePipeline,
    generate_mip_pipeline:                  wgpu::ComputePipeline,

    save_png_request:          Option<ImageCopyRequest>,
    video_frame_request_queue: std::collections::vec_deque::VecDeque<ImageCopyRequest>,
    last_reset_type:           ResetBoardType,

    initial_restriction_tex: Option<wgpu::Texture>,

    binding_layouts:        StafraBindingLayouts,
    click_rule_bindings:    StafraClickRuleBindings,
    initial_state_bindings: StafraInitialStateBindings,
    board_bindings:         StafraBoardBindings,
}

impl StafraBindingLayouts
{
    fn new(device: &wgpu::Device) -> Self
    {
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

        let main_render_state_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries:
            &[
                main_render_texture_binding!(0),
                main_render_sampler_binding!(1)
            ]
        });

        let click_rule_render_state_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries:
            &[
                click_rule_render_texture_binding!(0),
                click_rule_render_flags_binding!(1)
            ]
        });

        let clear_default_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries:
            &[
                board_image_binding!(0)
            ]
        });

        let bake_click_rule_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries:
            &[
                board_texture_binding!(0),
                click_rule_storage_binding!(1)
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

        let initial_restriction_transform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries:
            &[
                initial_texture_binding!(0),
                board_image_binding!(1)
            ]
        });

        let filter_restriction_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries:
            &[
                board_texture_binding!(0),
                board_texture_binding!(1),
                board_image_binding!(2)
            ]
        });

        let final_state_transform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label:   None,
            entries:
            &[
                board_texture_binding!(0),
                final_image_mip_binding!(1),
                spawn_data_uniform_binding!(2)
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

        let clear_restriction_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries:
            &[
                board_texture_binding!(0),
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

                board_texture_binding!(4),

                click_rule_uniform_binding!(5)
            ]
        });

        let generate_mip_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries:
            &[
                final_texture_mip_binding!(0),
                final_image_mip_binding!(1),
            ]
        });

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

        Self
        {
            main_render_state_bind_group_layout,
            click_rule_render_state_bind_group_layout,
            clear_default_bind_group_layout,
            bake_click_rule_bind_group_layout,
            initial_state_transform_bind_group_layout,
            initial_restriction_transform_bind_group_layout,
            filter_restriction_bind_group_layout,
            final_state_transform_bind_group_layout,
            clear_stability_bind_group_layout,
            clear_restriction_bind_group_layout,
            next_step_bind_group_layout,
            generate_mip_bind_group_layout,

            render_state_sampler
        }
    }
}

impl StafraClickRuleBindings
{
    fn new(device: &wgpu::Device, binding_layouts: &StafraBindingLayouts) -> Self
    {
        let click_rule_size = 32;

        let click_rule_texture_descriptor = wgpu::TextureDescriptor
        {
            label: None,
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
            label:              None,
            size:               4 * std::mem::size_of::<i32>() as u64 + ((click_rule_size * click_rule_size * 2) as u64) * std::mem::size_of::<i32>() as u64,
            usage:              wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        };

        let click_rule_render_flags_buffer_descriptor = wgpu::BufferDescriptor
        {
            label:              None,
            size:               std::mem::size_of::<u32>() as u64,
            usage:              wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        };

        let click_rule_texture             = device.create_texture(&click_rule_texture_descriptor);
        let click_rule_buffer              = device.create_buffer(&click_rule_buffer_descriptor);
        let click_rule_render_flags_buffer = device.create_buffer(&click_rule_render_flags_buffer_descriptor);

        let click_rule_texture_view_descriptor = wgpu::TextureViewDescriptor
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

        let click_rule_texture_view = click_rule_texture.create_view(&click_rule_texture_view_descriptor);

        let click_rule_render_state_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &binding_layouts.click_rule_render_state_bind_group_layout,
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
        });

        let bake_click_rule_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &binding_layouts.bake_click_rule_bind_group_layout,
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
        });

        Self
        {
            click_rule_render_state_bind_group,
            bake_click_rule_bind_group,

            click_rule_render_flags: 0,

            click_rule_texture,
            click_rule_buffer,
            click_rule_render_flags_buffer
        }
    }
}

impl StafraInitialStateBindings
{
    fn new(device: &wgpu::Device, board_width: u32, board_height: u32) -> Self
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

        let initial_state = device.create_texture(&initial_state_texture_descriptor);
        Self
        {
            initial_state
        }
    }
}

impl StafraBoardBindings
{
    fn new(device: &wgpu::Device, binding_layouts: &StafraBindingLayouts, click_rule_bindings: &StafraClickRuleBindings, initial_state_bindings: &StafraInitialStateBindings, width: u32, height: u32) -> Self
    {
        assert!((width  + 1).is_power_of_two());
        assert!((height + 1).is_power_of_two());

        let board_width  = width;
        let board_height = height;

        let board_texture_descriptor = wgpu::TextureDescriptor
        {
            label: None,
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
            label: None,
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
            label: None,
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

        let spawn_data_buffer_descriptor = wgpu::BufferDescriptor
        {
            label:              None,
            size:               2 * std::mem::size_of::<u32>() as u64,
            usage:              wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        };

        let current_board      = device.create_texture(&board_texture_descriptor);
        let next_board         = device.create_texture(&board_texture_descriptor);
        let current_stability  = device.create_texture(&board_texture_descriptor);
        let next_stability     = device.create_texture(&board_texture_descriptor);
        let restriction        = device.create_texture(&board_texture_descriptor);
        let final_state        = device.create_texture(&final_state_texture_descriptor);
        let video_frame        = device.create_texture(&video_frame_texture_descriptor);

        let spawn_data_buffer = device.create_buffer(&spawn_data_buffer_descriptor);

        let initial_state_view_descriptor = wgpu::TextureViewDescriptor
        {
            label:             None,
            format:            Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension:         Some(wgpu::TextureViewDimension::D2),
            aspect:            wgpu::TextureAspect::All,
            base_mip_level:    0,
            mip_level_count:   None,
            base_array_layer:  0,
            array_layer_count: None
        };

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

        let initial_state_view     = initial_state_bindings.initial_state.create_view(&initial_state_view_descriptor);
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

        let main_render_state_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &binding_layouts.main_render_state_bind_group_layout,
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
                    resource: wgpu::BindingResource::Sampler(&binding_layouts.render_state_sampler),
                }
            ]
        });

        let clear_default_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &binding_layouts.clear_default_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&next_board_view),
                },
            ]
        });

        let initial_transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &binding_layouts.initial_state_transform_bind_group_layout,
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
                    resource: wgpu::BindingResource::TextureView(&next_board_view)
                }
            ]
        });

        let filter_restriction_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &binding_layouts.filter_restriction_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&next_board_view)
                },

                wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&restriction_view)
                },

                wgpu::BindGroupEntry
                {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&current_board_view)
                }
            ]
        });

        let next_step_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &binding_layouts.next_step_bind_group_layout,
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
                    resource: wgpu::BindingResource::Buffer(click_rule_bindings.click_rule_buffer.as_entire_buffer_binding())
                }
            ]
        });

        let next_step_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &binding_layouts.next_step_bind_group_layout,
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

                wgpu::BindGroupEntry
                {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&restriction_view),
                },

                wgpu::BindGroupEntry
                {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(click_rule_bindings.click_rule_buffer.as_entire_buffer_binding())
                }
            ]
        });

        let final_transform_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label:   None,
            layout:  &binding_layouts.final_state_transform_bind_group_layout,
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
                },

                wgpu::BindGroupEntry
                {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(spawn_data_buffer.as_entire_buffer_binding())
                }
            ]
        });

        let final_transform_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label:   None,
            layout:  &binding_layouts.final_state_transform_bind_group_layout,
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
                },

                wgpu::BindGroupEntry
                {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(spawn_data_buffer.as_entire_buffer_binding())
                }
            ]
        });

        let clear_stability_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &binding_layouts.clear_stability_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&current_stability_view),
                },
            ]
        });

        let clear_stability_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &binding_layouts.clear_stability_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&next_stability_view),
                },
            ]
        });

        let clear_restriction_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &binding_layouts.clear_restriction_bind_group_layout,
            entries:
            &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&restriction_view),
                },
            ]
        });

        let mut generate_mip_bind_groups = Vec::with_capacity(final_state_mips as usize - 1);
        for i in 0..(final_state_mips - 1)
        {
            generate_mip_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor
            {
                label: None,
                layout: &binding_layouts.generate_mip_bind_group_layout,
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
            video_frame,

            spawn_data_buffer
        }
    }
}

impl StafraState
{
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new_native(main_window: &Window, click_rule_window: &Window, width: u32, height: u32) -> Self
    {
        let window_size     = main_window.inner_size();
        let click_rule_size = click_rule_window.inner_size();

        let wgpu_instance      = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let main_surface       = unsafe{wgpu_instance.create_surface(main_window)};
        let click_rule_surface = unsafe{wgpu_instance.create_surface(click_rule_window)};

        StafraState::new_impl(wgpu_instance, main_surface, click_rule_surface, window_size.width, window_size.height, click_rule_size.width, click_rule_size.height, width, height).await
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new_web(main_canvas: &web_sys::HtmlCanvasElement, click_rule_canvas: &web_sys::HtmlCanvasElement, width: u32, height: u32) -> Self
    {
        let canvas_width  = main_canvas.width();
        let canvas_height = main_canvas.height();

        let click_rule_width  = click_rule_canvas.width();
        let click_rule_height = click_rule_canvas.height();

        let wgpu_instance      = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let main_surface       = unsafe{wgpu_instance.create_surface_from_canvas(main_canvas)};
        let click_rule_surface = unsafe{wgpu_instance.create_surface_from_canvas(click_rule_canvas)};

        StafraState::new_impl(wgpu_instance, main_surface, click_rule_surface, canvas_width as u32, canvas_height as u32, click_rule_width as u32, click_rule_height as u32, width, height).await
    }

    async fn new_impl(instance: wgpu::Instance, main_surface: wgpu::Surface, click_rule_surface: wgpu::Surface, window_width: u32, window_height: u32, click_rule_width: u32, click_rule_height: u32, board_width: u32, board_height: u32) -> Self
    {
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions
        {
            power_preference:       wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface:     Some(&main_surface),
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
        });

        let swapchain_format = main_surface.get_preferred_format(&adapter).unwrap();
        main_surface.configure(&device, &wgpu::SurfaceConfiguration
        {
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT,
            format:       swapchain_format,
            width:        window_width,
            height:       window_height,
            present_mode: wgpu::PresentMode::Fifo
        });

        click_rule_surface.configure(&device, &wgpu::SurfaceConfiguration
        {
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT,
            format:       swapchain_format,
            width:        click_rule_width,
            height:       click_rule_height,
            present_mode: wgpu::PresentMode::Fifo
        });

        let main_render_state_vs_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/render/render_state_vs.wgsl"));
        let main_render_state_fs_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/render/render_state_fs.wgsl"));

        let click_rule_render_state_vs_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/render/click_rule_render_state_vs.wgsl"));
        let click_rule_render_state_fs_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/render/click_rule_render_state_fs.wgsl"));

        let clear_4_corners_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/clear_board/clear_4_corners.wgsl"));
        let clear_4_sides_module   = device.create_shader_module(&wgpu::include_wgsl!("shaders/clear_board/clear_4_sides.wgsl"));
        let clear_center_module    = device.create_shader_module(&wgpu::include_wgsl!("shaders/clear_board/clear_center.wgsl"));

        let initial_state_transform_module       = device.create_shader_module(&wgpu::include_wgsl!("shaders/state_transform/initial_state_transform.wgsl"));
        let initial_restriction_transform_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/state_transform/initial_restriction_transform.wgsl"));
        let filter_restriction_module            = device.create_shader_module(&wgpu::include_wgsl!("shaders/state_transform/filter_restriction.wgsl"));
        let final_state_transform_module         = device.create_shader_module(&wgpu::include_wgsl!("shaders/state_transform/final_state_transform.wgsl"));

        let clear_stability_module   = device.create_shader_module(&wgpu::include_wgsl!("shaders/state_transform/clear_stability.wgsl"));
        let clear_restriction_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/state_transform/clear_restriction.wgsl"));

        let bake_click_rule_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/click_rule/bake_click_rule.wgsl"));

        let next_step_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/next_step/next_step.wgsl"));

        let generate_mip_module = device.create_shader_module(&wgpu::include_wgsl!("shaders/mip/final_state_generate_next_mip.wgsl"));

        let binding_layouts        = StafraBindingLayouts::new(&device);
        let click_rule_bindings    = StafraClickRuleBindings::new(&device, &binding_layouts);
        let initial_state_bindings = StafraInitialStateBindings::new(&device, board_width, board_height);
        let board_bindings         = StafraBoardBindings::new(&device, &binding_layouts, &click_rule_bindings, &initial_state_bindings, board_width, board_height);

        let main_render_state_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&binding_layouts.main_render_state_bind_group_layout],
            push_constant_ranges: &[],
        });

        let click_rule_render_state_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&binding_layouts.click_rule_render_state_bind_group_layout],
            push_constant_ranges: &[],
        });

        let clear_default_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&binding_layouts.clear_default_bind_group_layout],
            push_constant_ranges: &[]
        });

        let bake_click_rule_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&binding_layouts.bake_click_rule_bind_group_layout],
            push_constant_ranges: &[]
        });

        let initial_state_transform_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&binding_layouts.initial_state_transform_bind_group_layout],
            push_constant_ranges: &[]
        });

        let initial_restriction_transform_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&binding_layouts.initial_restriction_transform_bind_group_layout],
            push_constant_ranges: &[]
        });

        let filter_restriction_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&binding_layouts.filter_restriction_bind_group_layout],
            push_constant_ranges: &[]
        });

        let final_state_transform_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&binding_layouts.final_state_transform_bind_group_layout],
            push_constant_ranges: &[]
        });

        let clear_stability_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&binding_layouts.clear_stability_bind_group_layout],
            push_constant_ranges: &[],
        });

        let clear_restriction_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&binding_layouts.clear_stability_bind_group_layout],
            push_constant_ranges: &[],
        });

        let next_step_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&binding_layouts.next_step_bind_group_layout],
            push_constant_ranges: &[],
        });

        let generate_mip_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: None,
            bind_group_layouts: &[&binding_layouts.generate_mip_bind_group_layout],
            push_constant_ranges: &[],
        });

        let main_render_state_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor
        {
            label: None,
            layout: Some(&main_render_state_pipeline_layout),

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
        });

        let click_rule_render_state_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor
        {
            label: None,
            layout: Some(&click_rule_render_state_pipeline_layout),

            vertex: wgpu::VertexState
            {
                module: &click_rule_render_state_vs_module,
                entry_point: "main",
                buffers: &[],
            },

            fragment: Some(wgpu::FragmentState
            {
                module: &click_rule_render_state_fs_module,
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
        });

        let clear_4_corners_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
        {
            label:       None,
            layout:      Some(&clear_default_pipeline_layout),
            module:      &clear_4_corners_module,
            entry_point: "main"
        });

        let clear_4_sides_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
        {
            label:       None,
            layout:      Some(&clear_default_pipeline_layout),
            module:      &clear_4_sides_module,
            entry_point: "main"
        });

        let clear_center_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
        {
            label:       None,
            layout:      Some(&clear_default_pipeline_layout),
            module:      &clear_center_module,
            entry_point: "main"
        });

        let bake_click_rule_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
        {
            label:       None,
            layout:      Some(&bake_click_rule_pipeline_layout),
            module:      &bake_click_rule_module,
            entry_point: "main"
        });

        let initial_state_transform_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
        {
            label:       None,
            layout:      Some(&initial_state_transform_pipeline_layout),
            module:      &initial_state_transform_module,
            entry_point: "main"
        });

        let initial_restriction_transform_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
        {
            label:       None,
            layout:      Some(&initial_restriction_transform_pipeline_layout),
            module:      &initial_restriction_transform_module,
            entry_point: "main"
        });

        let filter_restriction_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
        {
            label:       None,
            layout:      Some(&filter_restriction_pipeline_layout),
            module:      &filter_restriction_module,
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

        let clear_restriction_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor
        {
            label:       None,
            layout:      Some(&clear_restriction_pipeline_layout),
            module:      &clear_restriction_module,
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

        let max_video_frame_requests = 3;
        Self
        {
            main_surface,
            click_rule_surface,
            device,
            queue,

            swapchain_format,
            frame_number: 0,

            main_render_state_pipeline,
            click_rule_render_state_pipeline,
            clear_4_corners_pipeline,
            clear_4_sides_pipeline,
            clear_center_pipeline,
            bake_click_rule_pipeline,
            initial_state_transform_pipeline,
            initial_restriction_transform_pipeline,
            filter_restriction_pipeline,
            final_state_transform_pipeline,
            next_step_pipeline,
            clear_stability_pipeline,
            clear_restriction_pipeline,
            generate_mip_pipeline,

            save_png_request:          None,
            video_frame_request_queue: std::collections::vec_deque::VecDeque::with_capacity(max_video_frame_requests),
            last_reset_type:           ResetBoardType::Standard{reset_type: StandardResetBoardType::Corners},

            initial_restriction_tex: None,

            binding_layouts,
            click_rule_bindings,
            initial_state_bindings,
            board_bindings
        }
    }

    pub fn board_width(&self) -> u32
    {
        self.board_bindings.board_width
    }

    pub fn board_height(&self) -> u32
    {
        self.board_bindings.board_height
    }

    pub fn frame_number(&self) -> u32
    {
        self.frame_number
    }

    pub fn video_frame_queue_full(&self) -> bool
    {
        self.video_frame_request_queue.len() == self.video_frame_request_queue.capacity()
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32)
    {
        self.main_surface.configure(&self.device, &wgpu::SurfaceConfiguration
        {
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT,
            format:       self.swapchain_format,
            width:        new_width,
            height:       new_height,
            present_mode: wgpu::PresentMode::Fifo
        });
    }

    pub fn resize_click_rule(&mut self, new_width: u32, new_height: u32)
    {
        self.click_rule_surface.configure(&self.device, &wgpu::SurfaceConfiguration
        {
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT,
            format:       self.swapchain_format,
            width:        new_width,
            height:       new_height,
            present_mode: wgpu::PresentMode::Fifo
        });
    }

    #[cfg(target_arch = "wasm32")]
    pub fn post_save_png_request(&mut self)
    {
        if let Some(_) = &self.save_png_request
        {
            return;
        }

        let real_width  = (self.board_bindings.board_width  + 1) / 2;
        let real_height = (self.board_bindings.board_height + 1) / 2;

        let row_alignment = 256 as usize;
        let row_pitch     = ((real_width as usize * std::mem::size_of::<f32>()) + (row_alignment - 1)) & (!(row_alignment - 1));

        let save_png_buffer = self.device.create_buffer(&wgpu::BufferDescriptor
        {
            label:              None,
            size:               (row_pitch * real_height as usize) as u64,
            usage:              wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let mut buffer_copy_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});
        buffer_copy_encoder.copy_texture_to_buffer(wgpu::ImageCopyTexture
        {
            texture:   &self.board_bindings.final_state,
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
            buffer: &save_png_buffer,
            layout: wgpu::ImageDataLayout
            {
                offset:         0,
                bytes_per_row:  std::num::NonZeroU32::new(row_pitch as u32),
                rows_per_image: std::num::NonZeroU32::new(real_height)
            }
        },
        wgpu::Extent3d
        {
            width:                 real_width,
            height:                real_height,
            depth_or_array_layers: 1
        });

        self.queue.submit(std::iter::once(buffer_copy_encoder.finish()));

        let save_png_buffer_slice = save_png_buffer.slice(..);
        let save_png_future       = Box::new(save_png_buffer_slice.map_async(wgpu::MapMode::Read));

        self.save_png_request = Some(ImageCopyRequest
        {
            image_buffer:      save_png_buffer,
            buffer_map_future: save_png_future,
            row_pitch
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn post_video_frame_request(&mut self)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});
        self.render_video_frame(&mut encoder);

        let video_frame_width  = 1024;
        let video_frame_height = 1024;

        let row_alignment = 256 as usize;
        let row_pitch     = (video_frame_width * 4 + row_alignment - 1) & (!(row_alignment - 1));

        let video_frame_buffer = self.device.create_buffer(&wgpu::BufferDescriptor
        {
            label:              None,
            size:               (row_pitch * video_frame_height as usize) as u64,
            usage:              wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        encoder.copy_texture_to_buffer(wgpu::ImageCopyTexture
        {
            texture:   &self.board_bindings.video_frame,
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

        self.queue.submit(std::iter::once(encoder.finish()));

        let video_frame_buffer_slice = video_frame_buffer.slice(..);
        let video_frame_future       = Box::new(video_frame_buffer_slice.map_async(wgpu::MapMode::Read));

        self.video_frame_request_queue.push_back(ImageCopyRequest
        {
            image_buffer:      video_frame_buffer,
            buffer_map_future: video_frame_future,
            row_pitch
        });
    }

    #[cfg(target_arch = "wasm32")]
    pub fn check_save_png_request(&mut self) -> AcquireImageResult
    {
        if let None = &self.save_png_request
        {
            return AcquireImageResult::NoImagesRequested;
        }

        let unwrapped_request = &mut self.save_png_request.as_mut().unwrap();

        let save_png_buffer = &unwrapped_request.image_buffer;
        let save_png_future = &mut unwrapped_request.buffer_map_future;
        let row_pitch       = unwrapped_request.row_pitch;

        let waker       = dummy_waker::dummy_waker();
        let mut context = Context::from_waker(&waker);

        let pinned_future = Pin::new(save_png_future.as_mut());
        match Future::poll(pinned_future, &mut context)
        {
            std::task::Poll::Ready(_) =>
            {
                let padded_width  = self.board_bindings.board_width  + 1;
                let padded_height = self.board_bindings.board_height + 1;

                let mut image_array = vec![0; (padded_width * padded_height * 4) as usize];
                {
                    let png_buffer_view = save_png_buffer.slice(..).get_mapped_range();
                    for (row_index, row_chunk) in png_buffer_view.chunks(row_pitch).enumerate()
                    {
                        let real_row_index = (row_index * 2) as u32;
                        for (column_index, texel_bytes) in row_chunk.chunks(4).enumerate()
                        {
                            let real_column_index = (column_index * 2) as u32;
                            if real_column_index >= padded_width
                            {
                                break; //Can be bigger than real width if row_pitch is big enough
                            }

                            //Decode the quad
                            let top_left     = texel_bytes[0] as f32;
                            let top_right    = texel_bytes[1] as f32;
                            let bottom_left  = texel_bytes[2] as f32;
                            let bottom_right = texel_bytes[3] as f32;

                            let top_left_start     = (((real_row_index + 0) * padded_width + real_column_index + 0) * 4) as usize;
                            let top_right_start    = (((real_row_index + 0) * padded_width + real_column_index + 1) * 4) as usize;
                            let bottom_left_start  = (((real_row_index + 1) * padded_width + real_column_index + 0) * 4) as usize;
                            let bottom_right_start = (((real_row_index + 1) * padded_width + real_column_index + 1) * 4) as usize;

                            image_array[top_left_start + 0] = (top_left * 255.0) as u8; //Red
                            image_array[top_left_start + 1] = 0u8;                      //Green
                            image_array[top_left_start + 2] = (top_left * 255.0) as u8; //Blue
                            image_array[top_left_start + 3] = 255u8;                    //Alpha

                            image_array[top_right_start + 0] = (top_right * 255.0) as u8; //Red
                            image_array[top_right_start + 1] = 0u8;                       //Green
                            image_array[top_right_start + 2] = (top_right * 255.0) as u8; //Blue
                            image_array[top_right_start + 3] = 255u8;                     //Alpha

                            image_array[bottom_left_start + 0] = (bottom_left * 255.0) as u8; //Red
                            image_array[bottom_left_start + 1] = 0u8;                         //Green
                            image_array[bottom_left_start + 2] = (bottom_left * 255.0) as u8; //Blue
                            image_array[bottom_left_start + 3] = 255u8;                       //Alpha

                            image_array[bottom_right_start + 0] = (bottom_right * 255.0) as u8; //Red
                            image_array[bottom_right_start + 1] = 0u8;                          //Green
                            image_array[bottom_right_start + 2] = (bottom_right * 255.0) as u8; //Blue
                            image_array[bottom_right_start + 3] = 255u8;                        //Alpha
                        }
                    }
                }

                save_png_buffer.unmap();

                self.save_png_request = None;
                AcquireImageResult::AcquireSuccess
                {
                    pixel_data: image_array,
                    width:      padded_width,
                    height:     padded_height,
                }
            }

            std::task::Poll::Pending => AcquireImageResult::Pending
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn grab_video_frame(&mut self) -> AcquireImageResult
    {
        if self.video_frame_request_queue.is_empty()
        {
            return AcquireImageResult::NoImagesRequested;
        }

        let video_frame_buffer_request = self.video_frame_request_queue.front_mut().unwrap();
        let video_frame_future         = video_frame_buffer_request.buffer_map_future.as_mut();

        let waker       = dummy_waker::dummy_waker();
        let mut context = Context::from_waker(&waker);

        let pinned_future = Pin::new(video_frame_future);
        match Future::poll(pinned_future, &mut context)
        {
            std::task::Poll::Ready(_) =>
            {
                let video_frame_request_data = self.video_frame_request_queue.pop_front().unwrap();

                let video_frame_width  = 1024;
                let video_frame_height = 1024;

                //Because video_frame_width is a multiple of 256, row pitch is equal to width * 4.
                //We can copy the image contents directly to the buffer, which is MUCH faster
                let video_frame_image_array = video_frame_request_data.image_buffer.slice(..).get_mapped_range().to_vec();
                video_frame_request_data.image_buffer.unmap();

                AcquireImageResult::AcquireSuccess
                {
                    pixel_data: video_frame_image_array,
                    width:      video_frame_width as u32,
                    height:     video_frame_height as u32,
                }
            }

            std::task::Poll::Pending => AcquireImageResult::Pending
        }
    }

    pub fn set_click_rule_grid_enabled(&mut self, enable: bool)
    {
        let render_grid_flag = 0x01;
        if enable
        {
            self.click_rule_bindings.click_rule_render_flags |= render_grid_flag;
        }
        else
        {
            self.click_rule_bindings.click_rule_render_flags &= !render_grid_flag;
        }

        let buffer_data = self.click_rule_bindings.click_rule_render_flags.to_le_bytes();
        self.queue.write_buffer(&self.click_rule_bindings.click_rule_render_flags_buffer, 0, &buffer_data);
    }

    pub fn set_click_rule_read_only(&mut self, is_read_only: bool)
    {
        let click_rule_read_only_flag = 0x02;
        if is_read_only
        {
            self.click_rule_bindings.click_rule_render_flags |= click_rule_read_only_flag;
        }
        else
        {
            self.click_rule_bindings.click_rule_render_flags &= !click_rule_read_only_flag;
        }

        let buffer_data = self.click_rule_bindings.click_rule_render_flags.to_le_bytes();
        self.queue.write_buffer(&self.click_rule_bindings.click_rule_render_flags_buffer, 0, &buffer_data);
    }

    pub fn reset_board_unchanged(&mut self)
    {
        match self.last_reset_type
        {
            ResetBoardType::Standard {reset_type} =>
            {
                self.reset_board_standard_impl(reset_type);
            }

            ResetBoardType::Custom =>
            {
                self.reset_board_custom_impl();
            }
        }
    }

    pub fn reset_board_standard(&mut self, reset_type: StandardResetBoardType)
    {
        self.reset_board_standard_impl(reset_type);
        self.last_reset_type = ResetBoardType::Standard {reset_type};
    }

    pub fn reset_board_custom(&mut self, image_array: Vec<u8>, width: u32, height: u32)
    {
        //Crop to the largest possible square with sides of 2^n - 1
        let cropped_size = (min(width, height) + 2).next_power_of_two() / 2 - 1;
        self.initial_state_bindings = StafraInitialStateBindings::new(&self.device, cropped_size, cropped_size);
        self.board_bindings = StafraBoardBindings::new(&self.device, &self.binding_layouts, &self.click_rule_bindings, &self.initial_state_bindings, cropped_size, cropped_size);

        self.queue.write_texture(wgpu::ImageCopyTexture
        {
            texture:   &self.initial_state_bindings.initial_state,
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
            width:                 cropped_size,
            height:                cropped_size,
            depth_or_array_layers: 1
        });

        self.reset_board_custom_impl();
        self.last_reset_type = ResetBoardType::Custom;
    }

    pub fn upload_restriction(&mut self, image_array: Vec<u8>, width: u32, height: u32)
    {
        self.upload_restriction_impl(image_array, width, height);
        self.initial_transform_restriction();
        self.reset_board_unchanged();
    }

    pub fn clear_restriction(&mut self)
    {
        self.clear_restriction_impl();
        self.reset_board_unchanged();
    }

    pub fn resize_board(&mut self, new_width: u32, new_height: u32)
    {
        let cropped_size = (min(new_width, new_height) + 2).next_power_of_two() / 2 - 1;
        self.board_bindings = StafraBoardBindings::new(&self.device, &self.binding_layouts, &self.click_rule_bindings, &self.initial_state_bindings, cropped_size, cropped_size);

        if let Some(_) = &self.initial_restriction_tex
        {
            self.initial_transform_restriction();
        }
        else
        {
            self.clear_restriction_impl();
        }

        self.reset_board_unchanged();
    }

    pub fn reset_click_rule(&mut self, click_rule_data: &[u32; 32 * 32])
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

        self.queue.write_texture(wgpu::ImageCopyTexture
        {
            texture:   &self.click_rule_bindings.click_rule_texture,
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

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});
        self.bake_click_rule(&mut encoder);

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn set_spawn_period(&mut self, spawn_period: u32)
    {
        self.queue.write_buffer(&self.board_bindings.spawn_data_buffer, 0, &spawn_period.to_le_bytes());
    }

    pub fn set_smooth_transform_enabled(&mut self, enable: bool)
    {
        self.queue.write_buffer(&self.board_bindings.spawn_data_buffer, std::mem::size_of::<u32>() as wgpu::BufferAddress, &(enable as u32).to_le_bytes());
    }

    pub fn update(&mut self)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});

        self.calc_next_frame(&mut encoder);
        self.generate_final_image(&mut encoder);

        std::mem::swap(&mut self.board_bindings.next_step_bind_group_a,       &mut self.board_bindings.next_step_bind_group_b);
        std::mem::swap(&mut self.board_bindings.final_transform_bind_group_a, &mut self.board_bindings.final_transform_bind_group_b);
        self.frame_number += 1;

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError>
    {
        let main_frame       = self.main_surface.get_current_texture()?;
        let click_rule_frame = self.click_rule_surface.get_current_texture()?;

        let main_frame_view       = main_frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let click_rule_frame_view = click_rule_frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});
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

            main_render_pass.set_pipeline(&self.main_render_state_pipeline);
            main_render_pass.set_bind_group(0, &self.board_bindings.main_render_state_bind_group, &[]);
            main_render_pass.draw(0..3, 0..1);
        }

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

            click_rule_render_pass.set_pipeline(&self.click_rule_render_state_pipeline);
            click_rule_render_pass.set_bind_group(0, &self.click_rule_bindings.click_rule_render_state_bind_group, &[]);
            click_rule_render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        main_frame.present();
        click_rule_frame.present();

        Ok(())
    }

    fn render_video_frame(&mut self, encoder: &mut wgpu::CommandEncoder)
    {
        let video_frame_view = self.board_bindings.video_frame.create_view(&wgpu::TextureViewDescriptor::default());

        let mut main_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor
        {
            label: None,
            color_attachments:
            &[
                wgpu::RenderPassColorAttachment
                {
                    view:           &video_frame_view,
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

        main_render_pass.set_pipeline(&self.main_render_state_pipeline);
        main_render_pass.set_bind_group(0, &self.board_bindings.main_render_state_bind_group, &[]);
        main_render_pass.draw(0..3, 0..1);
    }

    fn calc_next_frame(&self, encoder: &mut wgpu::CommandEncoder)
    {
        let thread_groups_x = std::cmp::max((self.board_bindings.board_width + 1) / (2 * 8), 1u32);
        let thread_groups_y = std::cmp::max((self.board_bindings.board_height + 1) / (2 * 8), 1u32);

        {
            let mut next_step_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});

            next_step_pass.set_pipeline(&self.next_step_pipeline);
            next_step_pass.set_bind_group(0, &self.board_bindings.next_step_bind_group_a, &[]);
            next_step_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }
    }

    fn generate_final_image(&self, encoder: &mut wgpu::CommandEncoder)
    {
        let thread_groups_x = std::cmp::max((self.board_bindings.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_bindings.board_height + 1) / (2 * 16), 1u32);

        {
            let mut final_transform_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});

            final_transform_pass.set_pipeline(&self.final_state_transform_pipeline);
            final_transform_pass.set_bind_group(0, &self.board_bindings.final_transform_bind_group_a, &[]);
            final_transform_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }

        let mut thread_groups_mip_x = std::cmp::max(thread_groups_x / 2, 1u32);
        let mut thread_groups_mip_y = std::cmp::max(thread_groups_y / 2, 1u32);
        for gen_mip_bind_group in &self.board_bindings.generate_mip_bind_groups
        {
            let mut generate_mip_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});

            generate_mip_pass.set_pipeline(&self.generate_mip_pipeline);
            generate_mip_pass.set_bind_group(0, &gen_mip_bind_group, &[]);
            generate_mip_pass.dispatch(thread_groups_mip_x, thread_groups_mip_y, 1);

            thread_groups_mip_x = std::cmp::max(thread_groups_mip_x / 2, 1u32);
            thread_groups_mip_y = std::cmp::max(thread_groups_mip_y / 2, 1u32);
        }
    }

    fn reset_board_standard_impl(&mut self, reset_type: StandardResetBoardType)
    {
        let thread_groups_x = std::cmp::max((self.board_bindings.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_bindings.board_height + 1) / (2 * 16), 1u32);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});

        if self.frame_number % 2 == 1
        {
            //Make sure we're clearing the right one
            std::mem::swap(&mut self.board_bindings.next_step_bind_group_a,       &mut self.board_bindings.next_step_bind_group_b);
            std::mem::swap(&mut self.board_bindings.final_transform_bind_group_a, &mut self.board_bindings.final_transform_bind_group_b);
        }

        self.frame_number = 0;

        {
            let mut reset_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});
            match reset_type
            {
                StandardResetBoardType::Corners =>
                {
                    reset_pass.set_pipeline(&self.clear_4_corners_pipeline);
                    reset_pass.set_bind_group(0, &self.board_bindings.clear_default_bind_group, &[]);
                },

                StandardResetBoardType::Edges =>
                {
                    reset_pass.set_pipeline(&self.clear_4_sides_pipeline);
                    reset_pass.set_bind_group(0, &self.board_bindings.clear_default_bind_group, &[]);
                },

                StandardResetBoardType::Center =>
                {
                    reset_pass.set_pipeline(&self.clear_center_pipeline);
                    reset_pass.set_bind_group(0, &self.board_bindings.clear_default_bind_group, &[]);
                }
            }

            reset_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }

        self.filter_restriction(&mut encoder);

        self.clear_stability(&mut encoder);
        self.generate_final_image(&mut encoder);

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn reset_board_custom_impl(&mut self)
    {
        let thread_groups_x = std::cmp::max((self.board_bindings.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_bindings.board_height + 1) / (2 * 16), 1u32);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});

        if self.frame_number % 2 == 1
        {
            //Make sure we're clearing the right one
            std::mem::swap(&mut self.board_bindings.next_step_bind_group_a,       &mut self.board_bindings.next_step_bind_group_b);
            std::mem::swap(&mut self.board_bindings.final_transform_bind_group_a, &mut self.board_bindings.final_transform_bind_group_b);
        }

        self.frame_number = 0;

        {
            let mut initial_transform_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});

            initial_transform_pass.set_pipeline(&self.initial_state_transform_pipeline);
            initial_transform_pass.set_bind_group(0, &self.board_bindings.initial_transform_bind_group, &[]);

            initial_transform_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }

        self.filter_restriction(&mut encoder);

        self.clear_stability(&mut encoder);
        self.generate_final_image(&mut encoder);

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn upload_restriction_impl(&mut self, image_array: Vec<u8>, width: u32, height: u32)
    {
        let restriction_texture_descriptor = wgpu::TextureDescriptor
        {
            label: None,
            size:  wgpu::Extent3d
            {
                width:                 width,
                height:                height,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count:    1,
            dimension:       wgpu::TextureDimension::D2,
            format:          wgpu::TextureFormat::Rgba8Unorm,
            usage:           wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
        };

        let restriction_tex = self.device.create_texture(&restriction_texture_descriptor);
        self.queue.write_texture(wgpu::ImageCopyTexture
        {
            texture:   &restriction_tex,
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
            width:                 width,
            height:                height,
            depth_or_array_layers: 1
        });

        self.initial_restriction_tex = Some(restriction_tex);
    }

    fn initial_transform_restriction(&mut self)
    {
        let initial_restriction_view = self.initial_restriction_tex.as_ref().unwrap().create_view(&wgpu::TextureViewDescriptor::default());
        let restriction_view         = self.board_bindings.restriction.create_view(&wgpu::TextureViewDescriptor
        {
            label:             None,
            format:            Some(wgpu::TextureFormat::R32Uint),
            dimension:         Some(wgpu::TextureViewDimension::D2),
            aspect:            wgpu::TextureAspect::All,
            base_mip_level:    0,
            mip_level_count:   None,
            base_array_layer:  0,
            array_layer_count: None
        });

        let initial_restriction_transform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor
        {
            label: None,
            layout: &self.binding_layouts.initial_restriction_transform_bind_group_layout,
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
        });

        let thread_groups_x = std::cmp::max((self.board_bindings.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_bindings.board_height + 1) / (2 * 16), 1u32);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});

        {
            let mut initial_restriction_transform_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});

            initial_restriction_transform_pass.set_pipeline(&self.initial_restriction_transform_pipeline);
            initial_restriction_transform_pass.set_bind_group(0, &initial_restriction_transform_bind_group, &[]);

            initial_restriction_transform_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn clear_restriction_impl(&mut self)
    {
        let thread_groups_x = std::cmp::max((self.board_bindings.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_bindings.board_height + 1) / (2 * 16), 1u32);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: None});

        {
            let mut clear_restriction_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});

            clear_restriction_pass.set_pipeline(&self.clear_restriction_pipeline);
            clear_restriction_pass.set_bind_group(0, &self.board_bindings.clear_restriction_bind_group, &[]);
            clear_restriction_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn filter_restriction(&self, encoder: &mut wgpu::CommandEncoder)
    {
        let thread_groups_x = std::cmp::max((self.board_bindings.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_bindings.board_height + 1) / (2 * 16), 1u32);

        {
            let mut filter_restriction_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});

            filter_restriction_pass.set_pipeline(&self.filter_restriction_pipeline);
            filter_restriction_pass.set_bind_group(0, &self.board_bindings.filter_restriction_bind_group, &[]);
            filter_restriction_pass.dispatch(thread_groups_x, thread_groups_y, 1);
        }
    }

    fn clear_stability(&self, encoder: &mut wgpu::CommandEncoder)
    {
        let thread_groups_x = std::cmp::max((self.board_bindings.board_width  + 1) / (2 * 16), 1u32);
        let thread_groups_y = std::cmp::max((self.board_bindings.board_height + 1) / (2 * 16), 1u32);

        {
            let mut clear_stability_pass_a = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});

            clear_stability_pass_a.set_pipeline(&self.clear_stability_pipeline);
            clear_stability_pass_a.set_bind_group(0, &self.board_bindings.clear_stability_bind_group_a, &[]);
            clear_stability_pass_a.dispatch(thread_groups_x, thread_groups_y, 1);
        }

        {
            let mut clear_stability_pass_b = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});

            clear_stability_pass_b.set_pipeline(&self.clear_stability_pipeline);
            clear_stability_pass_b.set_bind_group(0, &self.board_bindings.clear_stability_bind_group_b, &[]);
            clear_stability_pass_b.dispatch(thread_groups_x, thread_groups_y, 1);
        }
    }

    fn bake_click_rule(&self, encoder: &mut wgpu::CommandEncoder)
    {
        let click_rule_size = 32;
        let workgroup_size  = 8;

        let thread_group_size = (click_rule_size / workgroup_size) as u32;

        let click_rule_buffer_size = 4 * std::mem::size_of::<u32>() + click_rule_size * click_rule_size * 2 * std::mem::size_of::<i32>();
        let click_rule_buffer_data = vec![0u8; click_rule_buffer_size];

        self.queue.write_buffer(&self.click_rule_bindings.click_rule_buffer, 0, click_rule_buffer_data.as_slice());

        {
            let mut bake_click_rule_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None});

            bake_click_rule_pass.set_pipeline(&self.bake_click_rule_pipeline);
            bake_click_rule_pass.set_bind_group(0, &self.click_rule_bindings.bake_click_rule_bind_group, &[]);
            bake_click_rule_pass.dispatch(thread_group_size, thread_group_size, 1);
        }
    }
}
