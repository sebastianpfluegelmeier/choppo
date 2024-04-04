extern crate ffmpeg_next as ffmpeg;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

use ffmpeg::ffi::{avformat_seek_file, AVSEEK_FLAG_FRAME};
use ffmpeg::format::{input, Pixel};
use ffmpeg::frame::Video;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};

pub struct VideoReader {
    scaler: Context,
    buffer: Vec<Video>,
    receiver: Receiver<(usize, Video)>,
    sender: Sender<ToVideoThread>,
}

enum ToVideoThread {
    LoadFrame,
    Stop,
}

impl VideoReader {
    pub fn new(file_name: String, frame: usize, target_w: u32, target_h: u32) -> Option<Self> {
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
            let (video_sender, video_receiver): (Sender<(usize, Video)>, Receiver<(usize, Video)>) =
                channel();
            let (frame_sender, frame_receiver): (Sender<ToVideoThread>, Receiver<ToVideoThread>) =
                channel();
            let timebase_numerator = input.time_base().numerator() as i64;
            let timebase_denominator = input.time_base().denominator() as i64;
            thread::spawn(move || {
                let mut current_frame = frame;
                let mut frames_to_read = 3;
                let timestamp = current_frame as i64 * timebase_denominator / timebase_numerator;
                unsafe {
                    avformat_seek_file(
                        ictx.as_mut_ptr(),
                        -1,
                        timestamp,
                        timestamp,
                        timestamp,
                        AVSEEK_FLAG_FRAME,
                    );
                }
                loop {
                    match frame_receiver.try_recv() {
                        Ok(ToVideoThread::Stop) => {
                            return;
                        }
                        Ok(ToVideoThread::LoadFrame) => {
                            current_frame += 1;

                            frames_to_read += 1;
                        }
                        Err(_) => (),
                    }
                    if frames_to_read > 0 {
                        for (stream, packet) in ictx.packets() {
                            if stream.index() == video_stream_index {
                                let _ = decoder.send_packet(&packet);
                                frames_to_read -= 1;
                                break;
                            }
                        }
                    }
                    let mut decoded = Video::empty();
                    if decoder.receive_frame(&mut decoded).is_ok() {
                        let _ = video_sender.send((current_frame, decoded));
                    }
                }
            });
            return Some(Self {
                scaler,
                sender: frame_sender,
                receiver: video_receiver,
                buffer: Vec::new(),
            });
        }
        None
    }

    pub fn read_next_frame(&mut self) -> Option<Video> {
        let _ = self.sender.send(ToVideoThread::LoadFrame);
        for _ in 0..50 {
            for (_frame, video) in self.receiver.try_iter() {
                self.buffer.push(video);
            }
            if !self.buffer.is_empty() {
                break;
            }
            thread::sleep(Duration::from_secs_f64(0.001))
        }
        if let Some(frame) = self.buffer.pop() {
            let mut rgb_frame = Video::empty();
            let _ = self.scaler.run(&frame, &mut rgb_frame);
            Some(rgb_frame)
        } else {
            None
        }
    }
    pub fn stop(&mut self) {
        let _ = self.sender.send(ToVideoThread::Stop);
    }
}
