use filetime::FileTime;

use crate::{
    parser::parse_main,
    reducer::{reduce, ReducedClip},
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
    receiver: Receiver<ReducedClip>,
}

impl SourceWatcher {
    pub fn new(path: String) -> Self {
        let (sender, receiver) = channel();

        let input = fs::read_to_string(&path).expect("could not read input");

        let parsed = parse_main(&input).unwrap().1;
        let path = path.clone();
        let path_ = path.clone();
        thread::spawn(move || {
            let mut last_timestamp = FileTime::now();

            loop {
                let metadata = fs::metadata(path.clone()).unwrap();
                let timestamp = FileTime::from_last_modification_time(&metadata);
                if last_timestamp != timestamp {
                    let input = fs::read_to_string(path.clone())
                        .map_err(|_e| ())
                        .expect("could not find file");
                    let _ = read_input(input, &sender);
                    last_timestamp = timestamp;
                }
            }
        });

        Self {
            file_path: path_.to_string(),
            video_path: parsed.directory_declaration.directory,
            extension: parsed.extension_declaration.extension,
            receiver,
        }
    }

    pub fn get_new_interpreted(&mut self) -> Option<ReducedClip> {
        let mut interpreted = None;
        for i in self.receiver.try_iter() {
            interpreted = Some(i);
        }
        interpreted
    }

    pub fn get_first_interpreted(&self) -> ReducedClip {
        let input = fs::read_to_string(&self.file_path).expect("could not read input");

        let parsed = parse_main(&input).unwrap().1;
        reduce(parsed)
    }
    pub fn get_file_path(&self) -> &String {
        &self.video_path
    }

    pub fn get_extension(&self) -> &String {
        &self.extension
    }
}

fn read_input(input: String, sender: &std::sync::mpsc::Sender<ReducedClip>) -> Result<(), ()> {
    let parsed = parse_main(&input).map_err(|_e| ())?.1;
    let interpreted = reduce(parsed);
    let _ = sender.send(interpreted);
    Result::Ok(())
}
