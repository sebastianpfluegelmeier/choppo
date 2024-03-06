extern crate ffmpeg_next as ffmpeg;

use ffmpeg::codec::decoder::Video as Decoder;
use ffmpeg::format::context::Input;
use ffmpeg::format::{input, Pixel};
use ffmpeg::frame::Video;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};

pub struct VideoReader {
    scaler: Context,
    decoder: Decoder,
    video_stream_index: usize,
    ictx: Input,
}

impl VideoReader {
    pub fn new(file_name: String, target_w: u32, target_h: u32) -> Self {
        ffmpeg::init().unwrap();
        if let Ok(ictx) = input(&file_name) {
            let input = ictx
                .streams()
                .best(Type::Video)
                .ok_or(ffmpeg::Error::StreamNotFound)
                .unwrap();
            let video_stream_index = input.index();

            let context_decoder =
                ffmpeg::codec::context::Context::from_parameters(input.parameters()).unwrap();
            let decoder = context_decoder.decoder().video().unwrap();

            let scaler = Context::get(
                decoder.format(),
                decoder.width(),
                decoder.height(),
                Pixel::RGBA,
                target_w,
                target_h,
                Flags::BILINEAR,
            )
            .unwrap();
            return Self {
                scaler,
                decoder,
                video_stream_index,
                ictx,
            };
        }
        panic!();
    }

    pub fn read_frame(&mut self) -> Video {
        let mut decoded = Video::empty();
        for (stream, packet) in self.ictx.packets() {
            if stream.index() == self.video_stream_index {
                self.decoder.send_packet(&packet).unwrap();
                break;
            }
        }
        if self.decoder.receive_frame(&mut decoded).is_ok() {
            let mut rgb_frame = Video::empty();
            self.scaler.run(&decoded, &mut rgb_frame).unwrap();

            return rgb_frame;
        }
        return Video::empty();
    }
}
