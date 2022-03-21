#![cfg(target_arch = "wasm32")]

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

pub struct VideoRecordState
{
    recorded_frame_count: u32,
    max_frame_count:      u32,
}

impl VideoRecordState
{
    pub fn new() -> Self
    {
        let chunk_output_callback = Closure::wrap(Box::new(move |video_chunk: web_sys::EncodedVideoChunk, chunk_metadata: web_sys::EncodedVideoChunkMetadata|
        {

        }) as Box<dyn Fn(web_sys::EncodedVideoChunk, web_sys::EncodedVideoChunkMetadata)>);

        let error_callback = Closure::wrap(Box::new(move |exception: web_sys::DomException|
        {

        }) as Box<dyn Fn(web_sys::DomException)>);

        let video_encoder = web_sys::VideoEncoderInit::new(chunk_output_callback.as_ref().unchecked_ref(), error_callback.as_ref().unchecked_ref());

        chunk_output_callback.forget();
        error_callback.forget();

        Self
        {
            recorded_frame_count: 0,
            max_frame_count:      0
        }
    }

    pub fn add_video_frame(&mut self, mut pixel_data: Vec<u8>, width: u32, height: u32)
    {
        let frame_duration  = 1000.0 / 60.0;
        let frame_timestamp = self.recorded_frame_count as f64 * frame_duration;

        let mut video_frame_buffer_init = web_sys::VideoFrameBufferInit::new(width, height, web_sys::VideoPixelFormat::Rgba, frame_timestamp);
        video_frame_buffer_init.duration(frame_duration);

        let video_frame = web_sys::VideoFrame::new_with_u8_array_and_video_frame_buffer_init(pixel_data.as_mut_slice(), &video_frame_buffer_init);

        self.recorded_frame_count += 1;
        if self.recorded_frame_count == self.max_frame_count
        {
            self.save_video();
        }
    }

    pub fn restart(&mut self)
    {
        self.recorded_frame_count = 0;
        self.max_frame_count      = u32::MAX;

        let video_frame_width  = 1024;
        let video_frame_height = 1024;

        let h264_string = "avc1.*";
        let record_config = web_sys::VideoEncoderConfig::new(h264_string, video_frame_width, video_frame_height);
    }

    pub fn is_recording(&self) -> bool
    {
        self.recorded_frame_count < self.max_frame_count
    }

    pub fn is_recording_finished(&self) -> bool
    {
        self.max_frame_count != 0 && self.recorded_frame_count >= self.max_frame_count
    }

    pub fn stop_recording(&mut self, final_frame: u32)
    {
        self.max_frame_count = final_frame;
    }

    fn save_video(&mut self)
    {
        self.recorded_frame_count = 0;
        self.max_frame_count      = 0;
    }
}