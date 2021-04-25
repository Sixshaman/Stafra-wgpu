use winit::
{
    event::{Event, WindowEvent, KeyboardInput},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
    platform::web::WindowExtWebSys,
    platform::web::WindowBuilderExtWebSys
};

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

        let sc_desc = wgpu::SwapChainDescriptor 
        {
            usage:        wgpu::TextureUsage::RENDER_ATTACHMENT,
            format:       adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width:        size.width,
            height:       size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Self 
        {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
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

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor 
        {
            label: Some("MainStafraCmdEncoder"),
        });
    
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor 
            {
                label: Some("Render Pass"),
                color_attachments: 
                &[
                    wgpu::RenderPassColorAttachment
                    {
                        view: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations 
                        {
                            load: wgpu::LoadOp::Clear
                            (
                                wgpu::Color 
                                {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }
                            ),
                            store: true,
                        }
                    }
                ],

                depth_stencil_attachment: None,
            });
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