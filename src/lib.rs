use winit::
{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
    platform::web::WindowExtWebSys,
    platform::web::WindowBuilderExtWebSys
};

use std::borrow::Cow;
use web_sys::console;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::prelude::*;

struct StafraState 
{
    surface:    wgpu::Surface,
    device:     wgpu::Device,
    queue:      wgpu::Queue,
    sc_desc:    wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size:       winit::dpi::PhysicalSize<u32>,

    render_state_pipeline:            wgpu::RenderPipeline,
    clear_4_corners_pipeline:         wgpu::ComputePipeline,
    initial_state_transform_pipeline: wgpu::ComputePipeline,
    final_state_transform_pipeline:   wgpu::ComputePipeline,

    //NO NEED FOR THESE, ONLY FOR BIND GROUPS
    current_board:     wgpu::Texture,
    next_board:        wgpu::Texture,
    current_stability: wgpu::Texture,
    next_stability:    wgpu::Texture
}

impl StafraState 
{
    async fn new(window: &Window) -> Self 
    {
        let size = window.inner_size();

        let wgpu_instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe{ wgpu_instance.create_surface(window) };

        let adapter = wgpu_instance.request_adapter(&wgpu::RequestAdapterOptions 
        {
            power_preference:   wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor 
        {
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(), 
            label: None,
        },
        None).await.unwrap();

        let swapchain_format = adapter.get_swap_chain_preferred_format(&surface).unwrap();
        let sc_desc = wgpu::SwapChainDescriptor 
        {
            usage:        wgpu::TextureUsage::RENDER_ATTACHMENT,
            format:       swapchain_format,
            width:        size.width,
            height:       size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let render_state_vs_module = device.create_shader_module(&wgpu::include_spirv!("../static/render_state_vs.spv"));
        let render_state_fs_module = device.create_shader_module(&wgpu::include_spirv!("../static/render_state_fs.spv"));

        let clear_4_corners_module = device.create_shader_module(&wgpu::include_spirv!("../static/clear_4_corners.spv"));

        let initial_state_transform_module = device.create_shader_module(&wgpu::include_spirv!("../static/initial_state_transform.spv"));
        let final_state_transform_module   = device.create_shader_module(&wgpu::include_spirv!("../static/final_state_transform.spv"));

        let render_state_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
        {
            label: None,
            entries: 
            &[
                wgpu::BindGroupLayoutEntry
                {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty:         wgpu::BindingType::Texture
                    {
                        sample_type:    wgpu::TextureSampleType::Float {filterable: true},
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled:   false,
                    },
                    count: None
                },

                wgpu::BindGroupLayoutEntry
                {
                    binding:    1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty:         wgpu::BindingType::Sampler
                    {
                        filtering:  true,
                        comparison: false,
                    },
                    count: None
                },
            ]
        });

        let render_state_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor 
        {
            label: None,
            bind_group_layouts: 
            &[
                &render_state_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let clear_4_corners_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor 
        {
            label: None,
            bind_group_layouts: 
            &[
                &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
                {
                    label: None,
                    entries: 
                    &[
                        wgpu::BindGroupLayoutEntry
                        {
                            binding:    0,
                            visibility: wgpu::ShaderStage::COMPUTE,
                            ty:         wgpu::BindingType::Buffer
                            {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: core::num::NonZeroU64::new(8)
                            },
                            count: None
                        },

                        wgpu::BindGroupLayoutEntry
                        {
                            binding: 1,
                            visibility: wgpu::ShaderStage::COMPUTE,
                            ty:         wgpu::BindingType::StorageTexture
                            {
                                access:         wgpu::StorageTextureAccess::WriteOnly,
                                format:         wgpu::TextureFormat::R8Uint,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None
                        }
                    ]
                })
            ],
            push_constant_ranges: 
            &[
                wgpu::PushConstantRange
                {
                    stages: wgpu::ShaderStage::COMPUTE,
                    range:  0..8
                }
            ],
        });

        let initial_state_transform_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor 
        {
            label: None,
            bind_group_layouts: 
            &[
                &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
                {
                    label: None,
                    entries: 
                    &[
                        wgpu::BindGroupLayoutEntry
                        {
                            binding: 0,
                            visibility: wgpu::ShaderStage::COMPUTE,
                            ty:         wgpu::BindingType::Texture
                            {
                                sample_type:    wgpu::TextureSampleType::Float {filterable: false},
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled:   false,
                            },
                            count: None
                        },

                        wgpu::BindGroupLayoutEntry
                        {
                            binding: 1,
                            visibility: wgpu::ShaderStage::COMPUTE,
                            ty:         wgpu::BindingType::StorageTexture
                            {
                                access:         wgpu::StorageTextureAccess::WriteOnly,
                                format:         wgpu::TextureFormat::R8Uint,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None
                        }
                    ]
                })
            ],
            push_constant_ranges: &[],
        });

        let final_state_transform_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor 
        {
            label: None,
            bind_group_layouts: 
            &[
                &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor
                {
                    label: None,
                    entries: 
                    &[
                        wgpu::BindGroupLayoutEntry
                        {
                            binding: 0,
                            visibility: wgpu::ShaderStage::COMPUTE,
                            ty:         wgpu::BindingType::Texture
                            {
                                sample_type:    wgpu::TextureSampleType::Uint,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled:   false,
                            },
                            count: None
                        },

                        wgpu::BindGroupLayoutEntry
                        {
                            binding: 1,
                            visibility: wgpu::ShaderStage::COMPUTE,
                            ty:         wgpu::BindingType::StorageTexture
                            {
                                access:         wgpu::StorageTextureAccess::WriteOnly,
                                format:         wgpu::TextureFormat::R32Float,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None
                        }
                    ]
                })
            ],
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

        let board_texture_descriptor = wgpu::TextureDescriptor
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
            format:          wgpu::TextureFormat::R8Uint,
            usage:           wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::STORAGE
        };

        let current_board     = device.create_texture(&board_texture_descriptor);
        let next_board        = device.create_texture(&board_texture_descriptor);
        let current_stability = device.create_texture(&board_texture_descriptor);
        let next_stability    = device.create_texture(&board_texture_descriptor);

        let initial_state_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor 
        {
            label: None,
            layout: &render_state_bind_group_layout,
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
                    resource: wgpu::BindingResource::Sampler(&stafra_sampler),
                }
            ]
        });        

        Self
        {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,

            render_state_pipeline,
            clear_4_corners_pipeline,
            initial_state_transform_pipeline,
            final_state_transform_pipeline,

            current_board,
            next_board,
            current_stability,
            next_stability
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) 
    {
        self.size           = new_size;
        self.sc_desc.width  = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain     = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn update(&mut self) 
    {
        todo!()
    }

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> 
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
                        view: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations 
                        {
                            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                            store: true,
                        },
                    }
                ],

                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_state_pipeline);
            render_pass.draw(0..3, 0..1);
        }
    
        self.queue.submit(std::iter::once(encoder.finish()));
    
        Ok(())
    }
}

async fn run(event_loop: EventLoop<()>, canvas_window: Window)
{
    let mut state = StafraState::new(&canvas_window).await; 

    event_loop.run(move |event, _, control_flow| match event
    {
        Event::WindowEvent 
        {
            ref event,
            window_id,
        } 
        if window_id == canvas_window.id() => match event 
        {
            WindowEvent::CloseRequested => 
            {
                *control_flow = ControlFlow::Exit;
            }

            WindowEvent::Resized(physical_size) => 
            {
                state.resize(*physical_size);
            }

            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => 
            {
                state.resize(**new_inner_size);
            }            
            _ => {}
        },

        Event::RedrawRequested(_) => 
        {
            //state.update();
            match state.render() 
            {
                Ok(_) => 
                {
                }

                Err(wgpu::SwapChainError::Lost) => 
                {
                    state.resize(state.size);
                }

                Err(wgpu::SwapChainError::OutOfMemory) => 
                {
                    
                    *control_flow = ControlFlow::Exit;
                }

                Err(e) => 
                {
                    let err_str = format!("{:?}", e);
                    web_sys::console::log_1(&err_str.into());
                }
            }
        }
        
        Event::MainEventsCleared => 
        {
            canvas_window.request_redraw();
        }

        _ => {}
    });
}

#[wasm_bindgen(start)]
pub fn entry_point()
{
    let event_loop = EventLoop::new();

    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas   = document.get_element_by_id("STAFRA_canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().ok();

    let canvas_window = WindowBuilder::new().with_canvas(canvas).build(&event_loop).unwrap();
    wasm_bindgen_futures::spawn_local(run(event_loop, canvas_window));
}