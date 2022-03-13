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

    pub fn add_video_frame(&mut self, pixel_data: Vec<u8>, width: u32, height: u32, row_pitch:  u32)
    {
        self.recorded_frame_count += 1;
    }

    pub fn save_video(&mut self)
    {
        self.recorded_frame_count = 0;
        self.max_frame_count      = 0;
    }
}