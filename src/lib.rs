use winit::
{
    event::{Event, WindowEvent, KeyboardInput},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
    platform::web::WindowExtWebSys,
    platform::web::WindowBuilderExtWebSys
};

use std::borrow::Cow;
use web_sys::console;
use futures::executor::block_on;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::prelude::*;

struct State 
{
    surface:    wgpu::Surface,
    device:     wgpu::Device,
    queue:      wgpu::Queue,
    sc_desc:    wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size:       winit::dpi::PhysicalSize<u32>,

    pipeline: wgpu::RenderPipeline
}

impl State 
{
    async fn new(window: &Window) -> Self 
    {
        let size = window.inner_size();

        let wgpu_instance = wgpu::Instance::new(wgpu::BackendBit::all());
        let surface = unsafe{ wgpu_instance.create_surface(window) };

        let adapter = wgpu_instance.request_adapter(&wgpu::RequestAdapterOptions 
        {
            power_preference:   wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor 
        {
            features: wgpu::Features::empty(),
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

        let main_vs_module = device.create_shader_module(&wgpu::include_spirv!("../static/main_vs.spv"));
        let main_fs_module = device.create_shader_module(&wgpu::include_spirv!("../static/main_fs.spv"));

        let main_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor 
        {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let main_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor 
        {
            label: None,
            layout: Some(&main_pipeline_layout),
            
            vertex: wgpu::VertexState 
            {
                module: &main_vs_module,
                entry_point: "main",
                buffers: &[],
            },

            fragment: Some(wgpu::FragmentState 
            {
                module: &main_fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState 
                {
                    format:     swapchain_format,
                    blend:      None,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
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

        Self
        {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            pipeline: main_render_pipeline
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) 
    {
        self.size           = new_size;
        self.sc_desc.width  = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain     = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn input(&mut self, event: &WindowEvent) -> bool 
    {
        todo!()
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

            render_pass.set_pipeline(&self.pipeline);
            render_pass.draw(0..3, 0..1);
        }
    
        self.queue.submit(std::iter::once(encoder.finish()));
    
        Ok(())
    }
}

async fn run(event_loop: EventLoop<()>, canvas_window: Window)
{
    let mut state = State::new(&canvas_window).await; 

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
    let canvas = document.get_element_by_id("STAFRA_canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().ok();

    let canvas_window = WindowBuilder::new().with_canvas(canvas).build(&event_loop).unwrap();
    wasm_bindgen_futures::spawn_local(run(event_loop, canvas_window));
}