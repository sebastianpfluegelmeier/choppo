use std::collections::HashMap;

use ffmpeg::frame::Video;

use crate::video_reader::VideoReader;

pub struct VideoLoader {
    readers: HashMap<String, VideoReader>,
    target_w: u32,
    target_h: u32,
}

impl VideoLoader {
    pub fn new(target_w: u32, target_h: u32) -> Self {
        Self {
            readers: HashMap::new(),
            target_w,
            target_h,
        }
    }

    pub fn preload(&mut self, name: &str) {
        self.readers.insert(
            name.to_string(),
            VideoReader::new(name.to_string(), self.target_w, self.target_h).unwrap(),
        );
    }

    pub fn load(&mut self, name: &str, frame: usize) -> Option<Video> {
        if let Some(reader) = self.readers.get_mut(name) {
            reader.read_frame(frame)
        } else {
            self.readers.insert(
                name.to_string(),
                VideoReader::new(name.to_string(), self.target_w, self.target_h).unwrap(),
            );
            self.load(name, frame)
        }
    }
}
