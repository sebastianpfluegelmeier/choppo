use std::collections::HashMap;

use ffmpeg::frame::Video;

use crate::video_reader::VideoReader;

pub struct VideoLoader {
    readers: HashMap<String, VideoReader>,
    last_video: String,
    target_w: u32,
    target_h: u32,
}

impl VideoLoader {
    pub fn new(target_w: u32, target_h: u32) -> Self {
        Self {
            readers: HashMap::new(),
            target_w,
            target_h,
            last_video: "".to_string(),
        }
    }

    pub fn preload(&mut self, name: &str) {
        self.readers.insert(
            name.to_string(),
            VideoReader::new(name.to_string(), self.target_w, self.target_h).unwrap(),
        );
    }

    pub fn load(&mut self, name: &str, frame: usize) -> Option<Video> {
        if name != &self.last_video {
            self.readers.get_mut(name).map(|r| {
                // THIS IS THE UGLIEST HACK FIX IN EXISTENCE BUT SEEMS TO WORK
                r.read_frame(frame);
                while r.read_frame(frame).is_none() {}
                r.read_frame(frame);
            });
            self.last_video = name.to_string();
        }
        if let Some(reader) = self.readers.get_mut(name) {
            reader.read_frame(frame)
        } else {
            println!("filename {}", name);
            self.readers.insert(
                name.to_string(),
                VideoReader::new(name.to_string(), self.target_w, self.target_h).unwrap(),
            );
            self.load(name, frame)
        }
    }
}
