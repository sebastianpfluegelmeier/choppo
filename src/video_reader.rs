extern crate ffmpeg_next as ffmpeg;

use ffmpeg::codec::decoder::Video as Decoder;
use ffmpeg::format::context::Input;
use ffmpeg::format::{input, Pixel};
use ffmpeg::frame::Video;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;

struct VideoReader {
    target_w: u32,
    target_h: u32,
    scaler: Context,
    input: Input,
    video_stream_index: usize,
    decoder: Decoder,
}

impl VideoReader {
    fn new(file_name: String, target_w: u32, target_h: u32) -> Self {
        if let Ok(mut ictx) = input(&file_name) {
            let input = ictx
                .streams()
                .best(Type::Video)
                .ok_or(ffmpeg::Error::StreamNotFound)
                .unwrap();
            let video_stream_index = input.index();

            let context_decoder =
                ffmpeg::codec::context::Context::from_parameters(input.parameters()).unwrap();
            let mut decoder = context_decoder.decoder().video().unwrap();

            let mut scaler = Context::get(
                decoder.format(),
                decoder.width(),
                decoder.height(),
                Pixel::RGB24,
                target_w,
                target_h,
                Flags::BILINEAR,
            )
            .unwrap();
            for (stream, packet) in ictx.packets() {
                if stream.index() == video_stream_index {
                    decoder.send_packet(&packet).unwrap();
                }
            }
            return Self {
                target_h,
                target_w,
                scaler,
                input: ictx,
                video_stream_index,
                decoder,
            };
        }
        panic!();
    }

    fn read_frame(&mut self) -> Video {
        let mut decoded = Video::empty();
        if self.decoder.receive_frame(&mut decoded).is_ok() {
            let mut rgb_frame = Video::empty();
            self.scaler.run(&decoded, &mut rgb_frame).unwrap();

            return rgb_frame;
        }
        return Video::empty();
    }
}
