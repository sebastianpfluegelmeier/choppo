extern crate ffmpeg_next as ffmpeg;

use std::collections::{HashMap};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;


use ffmpeg::ffi::{avformat_seek_file, AVSEEK_FLAG_FRAME};
use ffmpeg::format::{input, Pixel};
use ffmpeg::frame::Video;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};



pub struct VideoReader {
    scaler: Context,
    buffer: HashMap<usize, Video>,
    receiver: Receiver<(usize, Video)>,
    sender: Sender<ToVideoThread>,
}

enum ToVideoThread {
    LoadFrame(usize),
}

impl VideoReader {
    pub fn new(file_name: String, target_w: u32, target_h: u32) -> Option<Self> {
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
                let mut current_frame = 0;
                let mut last_frame = 0;
                let mut frames_to_read = 3;
                loop {
                    match frame_receiver.try_recv() {
                        Ok(ToVideoThread::LoadFrame(frame)) => {
                            last_frame = current_frame;
                            current_frame = frame;
                            frames_to_read += 1;

                            if current_frame != last_frame + 1 {
                                let timestamp = current_frame as i64 * timebase_denominator
                                    / timebase_numerator;
                                    println!("seek {}", frame);
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
                            }
                        }
                        Err(_) => (),
                    }
                    if frames_to_read > 0 {
                        for (stream, packet) in ictx.packets() {
                            if stream.index() == video_stream_index {
                                decoder.send_packet(&packet).unwrap();
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
                buffer: HashMap::new(),
            });
        }
        None
    }

    pub fn read_frame(&mut self, frame: usize) -> Option<Video> {
        let _ = self.sender.send(ToVideoThread::LoadFrame(frame));
        loop {
            for (frame, video) in self.receiver.try_iter() {
                self.buffer.insert(frame, video);
            }
            if self.buffer.contains_key(&frame) {
                break;
            }
        }
        if let Some(frame) = self.buffer.remove(&frame) {
            let mut rgb_frame = Video::empty();
            self.scaler.run(&frame, &mut rgb_frame).unwrap();
            Some(rgb_frame)
        } else {
            None
        }
    }
}
