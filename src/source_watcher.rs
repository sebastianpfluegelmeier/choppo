use filetime::FileTime;
use sdl2::libc::times;

use crate::{
    interpreter::{interpret, InterpretedClip},
    parser::parse_main,
};
use std::{
    fs,
    sync::mpsc::{channel, Receiver},
    thread,
};

pub struct SourceWatcher {
    file_path: String,
    video_path: String,
    extension: String,
    last_timestamp: FileTime,
    receiver: Receiver<InterpretedClip>,
}

impl SourceWatcher {
    pub fn new(path: &str) -> Self {
        let (sender, receiver) = channel();

        let input = fs::read_to_string(path).expect("could not read input");

        let parsed = parse_main(&input).unwrap().1;
        let metadata = fs::metadata(path).unwrap();

        let last_timestamp = FileTime::from_last_modification_time(&metadata);
        // thread::spawn(move || {
        //     let mut last_timestamp = last_timestamp.clone();

        //     loop {
        //         let metadata = fs::metadata(path).unwrap();
        //         let timestamp = FileTime::from_last_modification_time(&metadata);
        //         if last_timestamp != timestamp {
        //             let input = fs::read_to_string(path).expect("could not read input");

        //             let parsed = parse_main(&input).unwrap().1;
        //             let path = parsed.directory_declaration.directory.clone();
        //             let extension = parsed.extension_declaration.extension.clone();
        //             let interpreted = interpret(parsed);
        //             sender.send(interpreted);
        //         }
        //     }
        // });

        Self {
            file_path: path.to_string(),
            video_path: parsed.directory_declaration.directory,
            extension: parsed.extension_declaration.extension,
            last_timestamp,
            receiver,
        }
    }

    pub fn get_new_interpreted(&mut self) -> Option<InterpretedClip> {
        let mut interpreted = None;
        for i in self.receiver.try_iter() {
            interpreted = Some(i);
        }
        return interpreted;
    }

    pub fn get_first_interpreted(&mut self) -> InterpretedClip {
        let input = fs::read_to_string(&self.file_path).expect("could not read input");

        let parsed = parse_main(&input).unwrap().1;
        interpret(parsed)
    }
    pub fn get_file_path(&self) -> &String {
        &self.video_path
    }

    pub fn get_extension(&self) -> &String {
        &self.extension
    }
}
