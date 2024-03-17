extern crate ffmpeg_next as ffmpeg;

use ffmpeg::format::context::Input;
use ffmpeg::format::input;
use ffmpeg::frame::Video;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::format::pixel::Pixel;
use ffmpeg::util::format::sample::Type as SampleType;
use ffmpeg::util::frame::video::Video as FfmpegVideo;
use ffmpeg::util::rational::Rational;
use ffmpeg_sys_next::SwsContext;
use std::path::Path;

pub struct VideoReader {
    input: Input,
    video_stream_index: usize,
    video_info: VideoInfo, // Fixed the type of video_info
    sws_context: SwsContext,
    current_frame: usize,
    target_w: u32,
    target_h: u32,
}

impl VideoReader {
    pub fn new(
        file_name: String,
        start_frame: usize,
        target_w: u32,
        target_h: u32,
    ) -> Option<Self> {
        let path = Path::new(&file_name);
        let input = input(&path).ok()?;
        let input_ = input
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)
            .unwrap();
        let video_stream_index = input
            .streams()
            .best(Type::Video)
            .map(|stream| stream.index())
            .unwrap_or(0);
        let video_info = input.streams().best(Type::Video)?;
        let context_decoder =
            ffmpeg::codec::context::Context::from_parameters(input_.parameters()).unwrap();
        let mut decoder = context_decoder.decoder().video().unwrap();
        let sws_context = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            target_w as u32,
            target_h as u32,
            Flags::BILINEAR,
        )
        .unwrap();
        let current_frame = start_frame;
        Some(Self {
            input,
            video_stream_index,
            video_info,
            sws_context,
            current_frame,
            target_w,
            target_h,
        })
    }

    pub fn read_next_frame(&mut self) -> Option<Video> {
        let mut packet = ffmpeg::packet::Packet::empty();
        for (stream, packet) in self.input.packets() {
            if packet.stream() == self.video_stream_index {
                let mut frame = FfmpegVideo::empty();
                match self.input.decode(&packet, &mut frame) {
                    Ok(true) => {
                        let mut rgb_frame = FfmpegVideo::empty();
                        self.sws_context.scale(&frame, &mut rgb_frame).ok()?;
                        self.current_frame += 1;
                        return Some(rgb_frame.into());
                    }
                    Ok(false) => return None,
                    Err(_) => return None,
                }
            }
        }
        return None;
    }
}
