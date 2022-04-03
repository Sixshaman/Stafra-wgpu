#![cfg(target_arch = "wasm32")]

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

struct FrameCounter
{
    sent_frame_count:     u32,
    encoded_frame_count:  u32,
    recorded_frame_count: u32,
    max_frame_count:      u32
}

impl FrameCounter
{
    fn new() -> Self
    {
        Self
        {
            sent_frame_count:     0,
            encoded_frame_count:  0,
            recorded_frame_count: 0,
            max_frame_count:      u32::MAX
        }
    }

    fn reset(&mut self)
    {
        self.sent_frame_count     = 0;
        self.encoded_frame_count  = 0;
        self.recorded_frame_count = 0;
        self.max_frame_count      = u32::MAX;
    }
}

//Records a video, using a chain: Raw frame data -> VideoFrame -> VideoEncoder -> VideoDecoder -> MediaStreamTrackGenerator -> MediaRecorder
//Converting to VideoFrame is necessary because MediaStreamTrackGenerator can only handle VideoFrame objects.
//Piping the frame through VideoEncoder and VideoDecoder is necessary because MediaRecorder doesn't understand RGBA frames.
pub struct VideoRecordState
{
    frame_counter:  Rc<RefCell<FrameCounter>>,
    media_recorder: Rc<RefCell<web_sys::MediaRecorder>>,
    video_encoder:  web_sys::VideoEncoder,

    #[allow(dead_code)]
    media_stream: web_sys::MediaStream
}

impl VideoRecordState
{
    pub fn new() -> Self
    {
        let video_frame_width  = 1024;
        let video_frame_height = 1024;

        let frame_counter = Rc::new(RefCell::new(FrameCounter::new()));

        let error_callback = Closure::wrap(Box::new(move |exception: web_sys::DomException|
        {
            web_sys::console::log_1(&exception.to_string());
        }) as Box<dyn Fn(web_sys::DomException)>);


        //Create the stream
        let media_stream_track_generator_init = web_sys::MediaStreamTrackGeneratorInit::new("video");
        let media_stream_track_generator = web_sys::MediaStreamTrackGenerator::new(&media_stream_track_generator_init).unwrap();

        let media_stream = web_sys::MediaStream::new().unwrap();
        media_stream.add_track(&media_stream_track_generator);


        //Create the recorder
        let frame_counter_clone_for_stop = frame_counter.clone();
        let data_available_callback = Closure::wrap(Box::new(move |event: web_sys::BlobEvent|
        {
            let mut frame_counter = frame_counter_clone_for_stop.borrow_mut();

            let data_blob = event.data().unwrap();
            let url       = web_sys::Url::create_object_url_with_blob(&data_blob).unwrap();

            let document = web_sys::window().unwrap().document().unwrap();
            let link = document.create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
            link.set_href(&url);
            link.set_download(&"Stability.webm");
            link.click();

            link.remove();

            frame_counter.reset();

        }) as Box<dyn Fn(web_sys::BlobEvent)>);

        let mut media_recorder_options = web_sys::MediaRecorderOptions::new();
        media_recorder_options.mime_type("video/webm;codecs=vp8");

        let media_recorder = Rc::new(RefCell::new(web_sys::MediaRecorder::new_with_media_stream_and_media_recorder_options(&media_stream, &media_recorder_options).unwrap()));
        media_recorder.borrow_mut().set_ondataavailable(Some(data_available_callback.as_ref().unchecked_ref()));


        //Create the video decoder
        let frame_counter_clone_for_write = frame_counter.clone();
        let media_recorder_clone_for_write = media_recorder.clone();
        let after_write_callback = Closure::wrap(Box::new(move |_js_value: JsValue|
        {
            //Pause the recorder after each frame, to record at constant FPS
            let mut frame_counter = frame_counter_clone_for_write.borrow_mut();
            let media_recorder = media_recorder_clone_for_write.borrow_mut();

            frame_counter.recorded_frame_count += 1;

            if frame_counter.recorded_frame_count >= frame_counter.max_frame_count
            {
                media_recorder.stop().expect("Exception: MediaRecorder stop error");
            }
            else
            {
                media_recorder.pause().expect("Exception: MediaRecorder pause error");
            }

        }) as Box<dyn FnMut(JsValue)>);

        let timeout_write_callback = Closure::wrap(Box::new(move |_js_value: JsValue|
        {
            //Offset the time of pausing the recorder, to make the frame longer
            let timeout_arguments = js_sys::Array::new();
            let window            = web_sys::window().unwrap();

            window.set_timeout_with_callback_and_timeout_and_arguments(&after_write_callback.as_ref().unchecked_ref(), 16, &timeout_arguments).unwrap();

        }) as Box<dyn FnMut(JsValue)>);

        let media_recorder_clone_for_decoder = media_recorder.clone();
        let video_frame_output_callback = Closure::wrap(Box::new(move |video_frame: web_sys::VideoFrame|
        {
            let media_recorder = media_recorder_clone_for_decoder.borrow_mut();
            media_recorder.resume().expect("Exception: MediaRecorder resume error");

            #[allow(unused_must_use)]
            {
                let stream_writer = media_stream_track_generator.writable().get_writer();
                stream_writer.write_with_chunk(&video_frame).then(&timeout_write_callback);
                stream_writer.release_lock();
            }

        }) as Box<dyn Fn(web_sys::VideoFrame)>);

        let video_decoder_init = web_sys::VideoDecoderInit::new(error_callback.as_ref().unchecked_ref(), video_frame_output_callback.as_ref().unchecked_ref());

        let mut video_decoder_config = web_sys::VideoDecoderConfig::new("vp8");
        video_decoder_config.coded_width(video_frame_width);
        video_decoder_config.coded_height(video_frame_height);
        video_decoder_config.display_aspect_width(video_frame_width);
        video_decoder_config.display_aspect_height(video_frame_height);

        let video_decoder = web_sys::VideoDecoder::new(&video_decoder_init).unwrap();
        video_decoder.configure(&video_decoder_config);


        //Create the encoder
        let frame_counter_clone_for_encoder = frame_counter.clone();
        let chunk_output_callback = Closure::wrap(Box::new(move |video_chunk: web_sys::EncodedVideoChunk|
        {
            let mut frame_counter = frame_counter_clone_for_encoder.borrow_mut();
            video_decoder.decode(&video_chunk);

            frame_counter.encoded_frame_count += 1;
            if frame_counter.encoded_frame_count >= frame_counter.max_frame_count
            {
                #[allow(unused_must_use)]
                {
                    video_decoder.flush();
                }
            }

        }) as Box<dyn Fn(web_sys::EncodedVideoChunk)>);

        let video_encoder_init = web_sys::VideoEncoderInit::new(error_callback.as_ref().unchecked_ref(), chunk_output_callback.as_ref().unchecked_ref());

        let mut video_encoder_config = web_sys::VideoEncoderConfig::new("vp8", video_frame_width, video_frame_height);
        video_encoder_config.bitrate(1000000.0);
        video_encoder_config.framerate(60.0);

        let video_encoder = web_sys::VideoEncoder::new(&video_encoder_init).unwrap();
        video_encoder.configure(&video_encoder_config);


        data_available_callback.forget();
        video_frame_output_callback.forget();
        chunk_output_callback.forget();
        error_callback.forget();

        Self
        {
            frame_counter,
            media_recorder,
            video_encoder,

            media_stream
        }
    }

    pub fn add_video_frame(&mut self, mut pixel_data: Vec<u8>, width: u32, height: u32)
    {
        let mut frame_counter = self.frame_counter.borrow_mut();

        let frame_duration  = 1000000.0 / 60.0;
        let frame_timestamp = frame_counter.sent_frame_count as f64 * frame_duration;

        let mut video_frame_buffer_init = web_sys::VideoFrameBufferInit::new(height, width, web_sys::VideoPixelFormat::Rgba, frame_timestamp);
        video_frame_buffer_init.duration(frame_duration);

        let mut video_encoder_options = web_sys::VideoEncoderEncodeOptions::new();
        video_encoder_options.key_frame(frame_counter.sent_frame_count % 120 == 0 || frame_counter.sent_frame_count == frame_counter.max_frame_count - 1);

        let video_frame = web_sys::VideoFrame::new_with_u8_array_and_video_frame_buffer_init(pixel_data.as_mut_slice(), &video_frame_buffer_init).unwrap();
        self.video_encoder.encode_with_options(&video_frame, &video_encoder_options);
        video_frame.close();

        frame_counter.sent_frame_count += 1;
        if frame_counter.sent_frame_count >= frame_counter.max_frame_count
        {
            #[allow(unused_must_use)]
            {
                self.video_encoder.flush();
            }
        }
    }

    pub fn pending(&self) -> bool
    {
        self.frame_counter.borrow().sent_frame_count > 0
    }

    pub fn restart(&mut self)
    {
        self.frame_counter.borrow_mut().reset();
        self.media_recorder.borrow().start().unwrap();
    }

    pub fn set_frame_limit(&mut self, final_frame: u32)
    {
        let mut frame_counter = self.frame_counter.borrow_mut();
        frame_counter.max_frame_count = final_frame;
    }
}