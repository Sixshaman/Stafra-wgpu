mod dummy_waker;

use winit::
{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
    platform::web::WindowBuilderExtWebSys
};

use wasm_bindgen::{JsCast, Clamped};
use wasm_bindgen::prelude::*;
use winit::event_loop::EventLoopProxy;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Context;
use futures::Future;
use std::convert::TryInto;

struct BoardDimensions
{
    width:  u32,
    height: u32
}

struct StafraState 
{
    surface:     wgpu::Surface,
    device:      wgpu::Device,
    queue:       wgpu::Queue,
    sc_desc:     wgpu::SwapChainDescriptor,
    swap_chain:  wgpu::SwapChain,
    window_size: winit::dpi::PhysicalSize<u32>,

    board_size: BoardDimensions,

    render_state_pipeline:            wgpu::RenderPipeline,
    clear_4_corners_pipeline:         wgpu::ComputePipeline,
    initial_state_transform_pipeline: wgpu::ComputePipeline,
    final_state_transform_pipeline:   wgpu::ComputePipeline,

    render_state_bind_group: wgpu::BindGroup,

    #[allow(dead_code)]
    current_board:     wgpu::Texture,
    next_board:        wgpu::Texture,
    current_stability: wgpu::Texture,
    next_stability:    wgpu::Texture,
    final_state:       wgpu::Texture
}

struct AppState
{
    document: web_sys::Document,

    event_loop:       EventLoop<AppEvent>,
    event_loop_proxy: Rc<EventLoopProxy<AppEvent>>,
    canvas_window:    Window,

    save_png_function: Closure<dyn Fn()>,
}

enum AppEvent
{
    SavePng
    {
    },
}

impl StafraState
{
    async fn new(window: &Window, board_size: BoardDimensions) -> Self
    {
        let window_size = window.inner_size();

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
            width:        window_size.width,
            height:       window_size.height,
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
                            binding: 0,
                            visibility: wgpu::ShaderStage::COMPUTE,
                            ty:         wgpu::BindingType::StorageTexture
                            {
                                access:         wgpu::StorageTextureAccess::WriteOnly,
                                format:         wgpu::TextureFormat::R32Uint,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None
                        }
                    ]
                })
            ],
            push_constant_ranges: &[]
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
                                sample_type:    wgpu::TextureSampleType::Float {filterable: true},
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled:   false,
                            },
                            count: None
                        },

                        wgpu::BindGroupLayoutEntry
                        {
                            binding:    1,
                            visibility: wgpu::ShaderStage::COMPUTE,
                            ty:         wgpu::BindingType::Sampler
                            {
                                filtering:  false,
                                comparison: false,
                            },
                            count: None
                        },

                        wgpu::BindGroupLayoutEntry
                        {
                            binding: 2,
                            visibility: wgpu::ShaderStage::COMPUTE,
                            ty:         wgpu::BindingType::StorageTexture
                            {
                                access:         wgpu::StorageTextureAccess::WriteOnly,
                                format:         wgpu::TextureFormat::R32Uint,
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

        let final_state_texture_descriptor = wgpu::TextureDescriptor
        {
            label: None,
            size:  wgpu::Extent3d
            {
                width:                 board_size.width,
                height:                board_size.height,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count:    1,
            dimension:       wgpu::TextureDimension::D2,
            format:          wgpu::TextureFormat::R32Float,
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
            format:            Some(wgpu::TextureFormat::R32Float),
            dimension:         Some(wgpu::TextureViewDimension::D2),
            aspect:            wgpu::TextureAspect::All,
            base_mip_level:    0,
            mip_level_count:   None,
            base_array_layer:  0,
            array_layer_count: None
        };

        let current_board_view     = current_board.create_view(&board_view_descriptor);
        let next_board_view        = next_board.create_view(&board_view_descriptor);
        let current_stability_view = current_stability.create_view(&board_view_descriptor);
        let next_stability_view    = next_stability.create_view(&board_view_descriptor);
        let final_state_view       = final_state.create_view(&final_state_view_descriptor);


        let render_state_sampler = device.create_sampler(&wgpu::SamplerDescriptor
        {
            label: None,
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
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&final_state_view),
                },

                wgpu::BindGroupEntry 
                {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&render_state_sampler),
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
            window_size,

            board_size,

            render_state_pipeline,
            clear_4_corners_pipeline,
            initial_state_transform_pipeline,
            final_state_transform_pipeline,

            render_state_bind_group,

            current_board,
            next_board,
            current_stability,
            next_stability,
            final_state
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) 
    {
        self.window_size    = new_size;
        self.sc_desc.width  = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain     = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn get_png_data(&self) -> Result<Vec<u8>, &str>
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
            //Busy wait because asyncs in winit don't work at the time of writing this code
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

    fn update(&mut self) 
    {

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

impl AppState
{
    fn new() -> Self
    {
        let event_loop: EventLoop<AppEvent> = EventLoop::with_user_event();
        let event_loop_proxy                = Rc::new(event_loop.create_proxy());

        let window   = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas   = document.get_element_by_id("STAFRA_canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().ok();

        let canvas_window = WindowBuilder::new().with_canvas(canvas).build(&event_loop).unwrap();

        let event_loop_proxy_cloned = event_loop_proxy.clone();
        let save_png_function = Closure::wrap(Box::new(move ||
        {
            event_loop_proxy_cloned.send_event(AppEvent::SavePng {});
        }) as Box<dyn Fn()>);

        let save_png_button = document.get_element_by_id("save_png_button").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
        save_png_button.set_onclick(Some(save_png_function.as_ref().unchecked_ref()));

        Self
        {
            document,

            event_loop,
            event_loop_proxy,
            canvas_window,

            save_png_function
        }
    }

    async fn run(self)
    {
        let document = self.document;

        let canvas_window = self.canvas_window;
        let event_loop    = self.event_loop;

        let mut state = StafraState::new(&canvas_window, BoardDimensions {width: 1023, height: 1023}).await;
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

                WindowEvent::ScaleFactorChanged {new_inner_size, ..} =>
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
                        state.resize(state.window_size);
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

            Event::UserEvent(app_event) =>
            {
                match app_event
                {
                    AppEvent::SavePng {} =>
                    {
                        match state.get_png_data()
                        {
                            Ok(mut image_array) =>
                            {
                                let image_data = web_sys::ImageData::new_with_u8_clamped_array(Clamped(image_array.as_mut_slice()), state.board_size.width).unwrap();

                                let canvas = document.create_element("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
                                canvas.set_width(state.board_size.width);
                                canvas.set_height(state.board_size.height);

                                let canvas_context = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
                                canvas_context.put_image_data(&image_data, 0.0, 0.0);

                                let link = document.create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
                                link.set_target(&canvas.to_data_url_with_type("image/png").unwrap());
                                link.set_download(&"LightsOutMatrix.png");
                                link.click();

                                link.remove();
                                canvas.remove();
                            },

                            Err(message) =>
                            {
                                web_sys::console::log_1(&format!("Error saving PNG image: {}", message).into());
                            }
                        }
                    }
                }
            }

            _ => {}
        });
    }
}

#[wasm_bindgen(start)]
pub fn entry_point()
{
    let mut app_state = AppState::new();
    wasm_bindgen_futures::spawn_local(app_state.run());
}