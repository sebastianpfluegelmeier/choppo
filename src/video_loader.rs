use std::{collections::HashMap};

use ffmpeg::frame::Video;

use crate::video_reader::VideoReader;

pub struct VideoLoader {
    readers: HashMap<(usize, String), (VideoReader, usize)>,
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

    pub fn load(&mut self, name: &str, frame: usize, layer: usize) -> Option<Video> {
        if let Some((reader, last_frame)) = self.readers.get_mut(&(layer, name.to_string())) {
            if *last_frame + 1 != frame {
                *last_frame = frame;
                reader.stop();
                let mut reader =
                    VideoReader::new(name.to_string(), frame, self.target_w, self.target_h);
                reader.as_ref()?;
                let frame_ = reader.as_mut()?.read_next_frame();
                self.readers
                    .insert((layer, name.to_string()), (reader?, frame));
                frame_
            } else {
                *last_frame = frame;
                reader.read_next_frame()
            }
        } else {
            self.readers.insert(
                (layer, name.to_string()),
                (
                    VideoReader::new(name.to_string(), frame, self.target_w, self.target_h)?,
                    frame,
                ),
            );
            self.load(name, frame, layer)
        }
    }
}
