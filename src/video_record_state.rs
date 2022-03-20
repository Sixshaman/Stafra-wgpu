pub struct VideoRecordState
{
    pub recorded_frame_count: u32,
    pub max_frame_count:      u32,
}

impl VideoRecordState
{
    pub fn new() -> Self
    {
        Self
        {
            recorded_frame_count: 0,
            max_frame_count:      0
        }
    }

    pub fn add_video_frame(&mut self, pixel_data: Vec<u8>, width: u32, height: u32)
    {
        let frame_duration  = 1000.0 / 60.0;
        let frame_timestamp = self.recorded_frame_count as f64 * frame_duration;

        let mut video_frame_buffer_init = web_sys::VideoFrameBufferInit::new(width, height, web_sys::VideoPixelFormat::Rgba, frame_timestamp);
        video_frame_buffer_init.duration(frame_duration);

        let video_frame_buffer_array = js_sys::Uint8Array::new_with_length(width * height * 4);
        video_frame_buffer_array.copy_from(pixel_data.as_slice());

        self.recorded_frame_count += 1;
    }

    pub fn save_video(&mut self)
    {
        self.recorded_frame_count = 0;
        self.max_frame_count      = 0;
    }
}