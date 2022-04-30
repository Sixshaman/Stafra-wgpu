use
{
    futures::Future,
    std::num::NonZeroU32,
    std::cmp::min,
    std::collections::vec_deque::VecDeque,
};

use
{
    super::stafra_static_state::StafraStaticState,
    super::stafra_static_state_bindings::StafraStaticBindings,
    super::stafra_board_state_bindings::StafraBoardBindings,
    super::stafra_initial_state_bindings::StafraInitialStateBindings
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
    raw_width:         u32,
    raw_height:        u32,
    row_pitch:         usize
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

    save_png_request:          Option<ImageCopyRequest>,
    video_frame_request_queue: VecDeque<ImageCopyRequest>,
    last_reset_type:           ResetBoardType,

    initial_restriction_tex: Option<wgpu::Texture>,

    static_state:           StafraStaticState,
    static_bindings:        StafraStaticBindings,
    initial_state_bindings: StafraInitialStateBindings,
    board_bindings:         StafraBoardBindings,
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
        let adapter_option = instance.request_adapter(&wgpu::RequestAdapterOptions
        {
            power_preference:       wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface:     Some(&main_surface),
        }).await;

        #[cfg(target_arch = "wasm32")]
        if let None = adapter_option
        {
            web_sys::window().unwrap().alert_with_message("Wgpu is not supported").unwrap();
        }

        let adapter = adapter_option.unwrap();
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor
        {
            features: wgpu::Features::default(),
            limits:   wgpu::Limits::default(),
            label:    Some("Device"),
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

        let static_state           = StafraStaticState::new(&device, swapchain_format);
        let static_bindings        = StafraStaticBindings::new(&device, &static_state);
        let initial_state_bindings = StafraInitialStateBindings::new(&device, board_width, board_height);
        let board_bindings         = StafraBoardBindings::new(&device, &static_state, &static_bindings, &initial_state_bindings, board_width, board_height);

        let max_video_frame_requests = 3;
        Self
        {
            main_surface,
            click_rule_surface,
            device,
            queue,

            swapchain_format,
            frame_number: 0,

            save_png_request:          None,
            video_frame_request_queue: VecDeque::with_capacity(max_video_frame_requests),
            last_reset_type:           ResetBoardType::Standard{reset_type: StandardResetBoardType::Corners},

            initial_restriction_tex: None,

            static_state,
            static_bindings,
            initial_state_bindings,
            board_bindings
        }
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

    pub fn post_save_png_request(&mut self)
    {
        if let Some(_) = &self.save_png_request
        {
            return;
        }

        let mut buffer_copy_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("PNG buffer copy encoder")});
        let save_png_buffer = self.board_bindings.create_image_data_buffer(&self.device, &mut buffer_copy_encoder);
        self.queue.submit(std::iter::once(buffer_copy_encoder.finish()));

        let save_png_buffer_slice = save_png_buffer.image_buffer.slice(..);
        let save_png_future       = Box::new(save_png_buffer_slice.map_async(wgpu::MapMode::Read));

        self.save_png_request = Some(ImageCopyRequest
        {
            image_buffer:      save_png_buffer.image_buffer,
            buffer_map_future: save_png_future,
            raw_width:         save_png_buffer.raw_width,
            raw_height:        save_png_buffer.raw_height,
            row_pitch:         save_png_buffer.row_pitch
        })
    }

    pub fn post_video_frame_request(&mut self)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("Video frame copy encoder")});
        self.board_bindings.render_video_frame(&mut encoder, &self.static_state);

        let video_frame_buffer = self.board_bindings.create_video_frame_data_buffer(&self.device, &mut encoder);
        self.queue.submit(std::iter::once(encoder.finish()));

        let video_frame_buffer_slice = video_frame_buffer.image_buffer.slice(..);
        let video_frame_future       = Box::new(video_frame_buffer_slice.map_async(wgpu::MapMode::Read));

        self.video_frame_request_queue.push_back(ImageCopyRequest
        {
            image_buffer:      video_frame_buffer.image_buffer,
            buffer_map_future: video_frame_future,
            raw_width:         video_frame_buffer.raw_width,
            raw_height:        video_frame_buffer.raw_height,
            row_pitch:         video_frame_buffer.row_pitch
        });
    }

    pub fn check_save_png_request(&mut self) -> AcquireImageResult
    {
        if let None = &self.save_png_request
        {
            return AcquireImageResult::NoImagesRequested;
        }

        let unwrapped_request = &mut self.save_png_request.as_mut().unwrap();

        let save_png_buffer = &unwrapped_request.image_buffer;
        let save_png_future = &mut unwrapped_request.buffer_map_future;

        let raw_width  = unwrapped_request.raw_width;
        let raw_height = unwrapped_request.raw_height;
        let row_pitch  = unwrapped_request.row_pitch;

        let waker       = dummy_waker::dummy_waker();
        let mut context = Context::from_waker(&waker);

        let pinned_future = Pin::new(save_png_future.as_mut());
        match Future::poll(pinned_future, &mut context)
        {
            std::task::Poll::Ready(_) =>
            {
                let image_data = self.board_bindings.get_image_buffer_mapped_data(&save_png_buffer, raw_width, raw_height, row_pitch);
                save_png_buffer.unmap();

                self.save_png_request = None;
                AcquireImageResult::AcquireSuccess
                {
                    pixel_data: image_data.pixel_data,
                    width:      image_data.image_width,
                    height:     image_data.image_height,
                }
            }

            std::task::Poll::Pending => AcquireImageResult::Pending
        }
    }

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

                let video_frame_buffer = video_frame_request_data.image_buffer;
                let raw_width          = video_frame_request_data.raw_width;
                let raw_height         = video_frame_request_data.raw_height;

                let video_frame_data = self.board_bindings.get_video_frame_buffer_mapped_data(&video_frame_buffer, raw_width, raw_height);
                video_frame_buffer.unmap();

                AcquireImageResult::AcquireSuccess
                {
                    pixel_data: video_frame_data.pixel_data,
                    width:      video_frame_data.image_width,
                    height:     video_frame_data.image_height,
                }
            }

            std::task::Poll::Pending => AcquireImageResult::Pending
        }
    }

    pub fn set_click_rule_grid_enabled(&mut self, enable: bool)
    {
        self.static_bindings.set_click_rule_grid_enabled(enable);
    }

    pub fn set_click_rule_read_only(&mut self, is_read_only: bool)
    {
        self.static_bindings.set_click_rule_read_only(is_read_only);
    }

    pub fn reset_board_unchanged(&mut self)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("Reset board unchanged encoder")});
        self.reset_board_unchanged_impl(&mut encoder);
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn reset_board_unchanged_impl(&mut self, encoder: &mut wgpu::CommandEncoder)
    {
        match self.last_reset_type
        {
            ResetBoardType::Standard {reset_type} =>
            {
                self.reset_board_standard_impl(encoder, reset_type);
            }

            ResetBoardType::Custom =>
            {
                self.reset_board_custom_impl(encoder);
            }
        }
    }

    pub fn reset_board_standard(&mut self, reset_type: StandardResetBoardType)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("Reset board standard encoder")});
        self.reset_board_standard_impl(&mut encoder, reset_type);
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn reset_board_standard_impl(&mut self, encoder: &mut wgpu::CommandEncoder, reset_type: StandardResetBoardType)
    {
        match reset_type
        {
            StandardResetBoardType::Corners => {self.board_bindings.reset_board_standard_corners(encoder, &self.static_state);}
            StandardResetBoardType::Edges   => {self.board_bindings.reset_board_standard_edges(encoder, &self.static_state);}
            StandardResetBoardType::Center  => {self.board_bindings.reset_board_standard_center(encoder, &self.static_state);}
        }

        self.board_bindings.filter_restriction(encoder, &self.static_state);
        self.board_bindings.clear_stability(encoder, &self.static_state);
        self.board_bindings.generate_final_image(encoder, &self.static_state, 0);

        self.last_reset_type = ResetBoardType::Standard {reset_type};
        self.frame_number = 0;
    }

    pub fn reset_board_custom(&mut self, image_array: Vec<u8>, width: u32, height: u32) -> u32
    {
        //Crop to the largest possible square with sides of 2^n - 1
        let cropped_size = (min(width, height) + 2).next_power_of_two() / 2 - 1;
        self.initial_state_bindings = StafraInitialStateBindings::new(&self.device, cropped_size, cropped_size);
        self.board_bindings = StafraBoardBindings::new(&self.device, &self.static_state, &self.static_bindings, &self.initial_state_bindings, cropped_size, cropped_size);

        self.initial_state_bindings.upload_texture(&self.queue, image_array, width, height);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("Reset board custom encoder")});
        self.reset_board_custom_impl(&mut encoder);
        self.queue.submit(std::iter::once(encoder.finish()));

        cropped_size
    }

    pub fn reset_board_custom_impl(&mut self, encoder: &mut wgpu::CommandEncoder)
    {
        self.board_bindings.reset_board_custom(encoder, &self.static_state);

        self.board_bindings.filter_restriction(encoder, &self.static_state);
        self.board_bindings.clear_stability(encoder, &self.static_state);
        self.board_bindings.generate_final_image(encoder, &self.static_state, 0);

        self.last_reset_type = ResetBoardType::Custom;
        self.frame_number = 0;
    }

    pub fn upload_restriction(&mut self, image_array: Vec<u8>, width: u32, height: u32)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("Upload restriction encoder")});

        self.upload_restriction_impl(image_array, width, height);

        let initial_restriction_view = self.initial_restriction_tex.as_ref().unwrap().create_view(&wgpu::TextureViewDescriptor::default());
        self.board_bindings.initial_transform_restriction(&self.device, &mut encoder, &initial_restriction_view, &self.static_state);
        self.reset_board_unchanged_impl(&mut encoder);

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn clear_restriction(&mut self)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("Clear restriction encoder")});

        self.board_bindings.clear_restriction(&mut encoder, &self.static_state);
        self.reset_board_unchanged_impl(&mut encoder);

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn resize_board(&mut self, new_width: u32, new_height: u32)
    {
        let cropped_size = (min(new_width, new_height) + 2).next_power_of_two() / 2 - 1;
        self.board_bindings = StafraBoardBindings::new(&self.device, &self.static_state, &self.static_bindings, &self.initial_state_bindings, cropped_size, cropped_size);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("Resize board encoder")});

        if let Some(_) = &self.initial_restriction_tex
        {
            let initial_restriction_view = self.initial_restriction_tex.as_ref().unwrap().create_view(&wgpu::TextureViewDescriptor::default());
            self.board_bindings.initial_transform_restriction(&self.device, &mut encoder, &initial_restriction_view, &self.static_state);
        }
        else
        {
            self.board_bindings.clear_restriction(&mut encoder, &self.static_state);
        }

        self.reset_board_unchanged_impl(&mut encoder);

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn reset_click_rule(&mut self, click_rule_data: &[u8; 32 * 32])
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("Reset click rule encoder")});
        self.static_bindings.reset_click_rule(&self.queue, &mut encoder, &self.static_state, click_rule_data);
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn set_spawn_period(&mut self, spawn_period: u32)
    {
        self.static_bindings.set_spawn_period(spawn_period);
    }

    pub fn set_smooth_transform_enabled(&mut self, enable: bool)
    {
        self.static_bindings.set_smooth_transform_enabled(enable);
    }

    pub fn update(&mut self)
    {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("Update encoder")});

        self.board_bindings.calc_next_frame(&mut encoder, &self.static_state, self.frame_number);
        self.board_bindings.generate_final_image(&mut encoder, &self.static_state, self.frame_number);

        self.frame_number += 1;

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn update_visual_info(&mut self)
    {
        self.static_bindings.update_draw_state(&self.queue);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError>
    {
        let main_frame       = self.main_surface.get_current_texture()?;
        let click_rule_frame = self.click_rule_surface.get_current_texture()?;

        let main_frame_view       = main_frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let click_rule_frame_view = click_rule_frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label: Some("Render encoder")});

        self.board_bindings.draw_main_state(&mut encoder, &main_frame_view, &self.static_state);
        self.static_bindings.draw_click_rule(&mut encoder, &click_rule_frame_view, &self.static_state);

        self.queue.submit(std::iter::once(encoder.finish()));

        main_frame.present();
        click_rule_frame.present();

        Ok(())
    }

    fn upload_restriction_impl(&mut self, image_array: Vec<u8>, width: u32, height: u32)
    {
        let restriction_texture_descriptor = wgpu::TextureDescriptor
        {
            label: Some("Restriction texture"),
            size:  wgpu::Extent3d
            {
                width,
                height,
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
            width,
            height,
            depth_or_array_layers: 1
        });

        self.initial_restriction_tex = Some(restriction_tex);
    }
}
