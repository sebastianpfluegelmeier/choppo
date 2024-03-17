extern crate ffmpeg_next as ffmpeg;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use ffmpeg::format::{input, Pixel};
use ffmpeg::frame::Video;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg_sys_next::avformat_seek_file;

pub struct VideoReader {
    scaler: Context,
    buffer: Vec<Video>,
    receiver: Receiver<(usize, Video)>,
    sender: Sender<ToVideoThread>,
}

enum ToVideoThread {
    LoadFrame,
    StopReader,
}

impl VideoReader {
    pub fn new(
        file_name: String,
        start_frame: usize,
        target_w: u32,
        target_h: u32,
    ) -> Option<Self> {
    }

    pub fn read_next_frame(&mut self) -> Option<Video> {
    }

    pub fn stop_reader(&mut self) {
    }
}
