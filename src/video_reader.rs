extern crate ffmpeg_next as ffmpeg;

use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

use ffmpeg::format::{input, Pixel};
use ffmpeg::frame::Video;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};

pub struct VideoReader {
    scaler: Context,
    buffer: VecDeque<Video>,
    receiver: Receiver<Video>,
    sender: Sender<usize>,
    frame: usize,
}

impl VideoReader {
    pub fn new(file_name: String, target_w: u32, target_h: u32) -> Self {
        ffmpeg::init().unwrap();
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
            let (frame_sender, frame_receiver) = channel();
            let (buffer_size_sender, buffer_siz_receiver) = channel();
            let mut buffer_size = 0;
            thread::spawn(move || loop {
                for (stream, packet) in ictx.packets() {
                    if stream.index() == video_stream_index {
                        decoder.send_packet(&packet).unwrap();
                        break;
                    }
                }
                let mut decoded = Video::empty();
                if decoder.receive_frame(&mut decoded).is_ok() {
                    'inner: loop {
                        if let Result::Ok(bs) = buffer_siz_receiver.recv_timeout(Duration::from_millis(1)) {
                            buffer_size = bs;
                        };
                        if buffer_size < 5 {
                            break 'inner;
                        }
                    }
                    let _ = frame_sender.send(decoded);
                    buffer_size += 1;
                }
            });
            return Self {
                scaler,
                sender: buffer_size_sender,
                receiver: frame_receiver,
                buffer: VecDeque::new(),
                frame: 0,
            };
        }

        panic!();
    }

    pub fn read_frame(&mut self) -> Option<Video> {
        let _ = self.sender.send(self.buffer.len());
        for video in self.receiver.try_iter() {
            self.buffer.push_front(video);
            let _ = self.sender.send(self.buffer.len());
        }
        if !self.buffer.is_empty() {
            let mut rgb_frame = Video::empty();
            self.scaler
                .run(&self.buffer.pop_back().unwrap(), &mut rgb_frame)
                .unwrap();
            self.frame += 1;
            Some(rgb_frame)
        } else {
            None
        }
    }
}
