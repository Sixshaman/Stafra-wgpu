#![cfg(target_arch = "wasm32")]

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::MediaSource;

//MediaRecorder <- MediaStream
//Records a video, using a chain: Raw frame data -> VideoFrame -> MediaSource -> HTMLVideoElement -> MediaRecorder
pub struct VideoRecordState
{
    recorded_frame_count: u32,
    max_frame_count:      u32,

    video_encoder:  web_sys::VideoEncoder,
    //media_recorder: web_sys::MediaRecorder
}

impl VideoRecordState
{
    pub fn new() -> Self
    {
        let media_source_open_callback = Closure::wrap(Box::new(move |event: web_sys::Event|
        {
            let media_source = event.target().unwrap().dyn_into::<web_sys::MediaSource>().unwrap();
            media_source.add_source_buffer("video/mp4; codecs=\"avc1.4D401E\"").unwrap(); //H.264 Main profile with constraint flags "40" (?) and level "1E" (???)

        }) as Box<dyn Fn(web_sys::Event)>);

        let media_source = web_sys::MediaSource::new().unwrap();
        media_source.set_onsourceopen(Some(media_source_open_callback.as_ref().unchecked_ref()));

        let document = web_sys::window().unwrap().document().unwrap();
        let video_element = document.create_element("video").unwrap().dyn_into::<web_sys::HtmlVideoElement>().unwrap();

        let media_source_url = web_sys::Url::create_object_url_with_source(&media_source).unwrap();
        video_element.set_src(&media_source_url);

        let chunk_output_callback = Closure::wrap(Box::new(move |video_chunk: web_sys::EncodedVideoChunk, _chunk_metadata: web_sys::EncodedVideoChunkMetadata|
        {
            let source_buffer_list = media_source.source_buffers();
            if source_buffer_list.length() > 0
            {
                let mut encoded_chunk_buffer = vec![0u8; video_chunk.byte_length() as usize];
                video_chunk.copy_to_with_u8_array(encoded_chunk_buffer.as_mut_slice());
                source_buffer_list.get(0).unwrap().append_buffer_with_u8_array(encoded_chunk_buffer.as_mut_slice()).unwrap();
            }
        }) as Box<dyn Fn(web_sys::EncodedVideoChunk, web_sys::EncodedVideoChunkMetadata)>);

        let error_callback = Closure::wrap(Box::new(move |exception: web_sys::DomException|
        {
            web_sys::console::log_1(&format!("Video recording error: {}", exception.name()).into());
        }) as Box<dyn Fn(web_sys::DomException)>);

        let video_encoder_init = web_sys::VideoEncoderInit::new(chunk_output_callback.as_ref().unchecked_ref(), error_callback.as_ref().unchecked_ref());
        let video_encoder      = web_sys::VideoEncoder::new(&video_encoder_init).unwrap();

        //let media_recorder = web_sys::MediaRecorder::new_with_media_stream(&video_element.src_object().unwrap()).unwrap();

        chunk_output_callback.forget();
        error_callback.forget();
        media_source_open_callback.forget();

        Self
        {
            recorded_frame_count: 0,
            max_frame_count:      0,

            video_encoder,
            //media_recorder
        }
    }

    pub fn add_video_frame(&mut self, mut pixel_data: Vec<u8>, width: u32, height: u32)
    {
        let frame_duration  = 1000.0 / 60.0;
        let frame_timestamp = self.recorded_frame_count as f64 * frame_duration;

        let mut video_frame_buffer_init = web_sys::VideoFrameBufferInit::new(width, height, web_sys::VideoPixelFormat::Rgba, frame_timestamp);
        video_frame_buffer_init.duration(frame_duration);

        let video_frame = web_sys::VideoFrame::new_with_u8_array_and_video_frame_buffer_init(pixel_data.as_mut_slice(), &video_frame_buffer_init).unwrap();
        self.video_encoder.encode(&video_frame);

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

        self.video_encoder.reset();

        let video_frame_width  = 1024;
        let video_frame_height = 1024;

        let h264_string = "avc1.*";
        let mut record_config = web_sys::VideoEncoderConfig::new(h264_string, video_frame_width, video_frame_height);
        record_config.bitrate(40000.0);
        record_config.framerate(16.6);
        record_config.hardware_acceleration(web_sys::HardwareAcceleration::PreferHardware);

        self.video_encoder.configure(&record_config);
        //self.media_recorder.start().unwrap();
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