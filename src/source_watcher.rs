use filetime::FileTime;


use crate::{
    reducer::{reduce, ReducedClip},
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
    receiver: Receiver<ReducedClip>,
}

impl SourceWatcher {
    pub fn new(path: String) -> Self {
        let (sender, receiver) = channel();

        let input = fs::read_to_string(&path).expect("could not read input");

        let parsed = parse_main(&input).unwrap().1;
        let metadata = fs::metadata(&path).unwrap();
        println!("\n\n{:?}\n\n", parsed);

        let last_timestamp = FileTime::from_last_modification_time(&metadata);
        let path = path.clone();
        let path_ = path.clone();
        thread::spawn(move || {
            let last_timestamp = last_timestamp;

            loop {
                read_input(&path, last_timestamp, &sender);
            }
        });

        Self {
            file_path: path_.to_string(),
            video_path: parsed.directory_declaration.directory,
            extension: parsed.extension_declaration.extension,
            last_timestamp,
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

fn read_input(
    path: &String,
    last_timestamp: FileTime,
    sender: &std::sync::mpsc::Sender<ReducedClip>,
) -> Result<(), ()> {
    let metadata = fs::metadata(path).unwrap();
    let timestamp = FileTime::from_last_modification_time(&metadata);
    if last_timestamp != timestamp {
        let input = fs::read_to_string(path).map_err(|_e| ())?;

        let parsed = parse_main(&input).map_err(|_e| ())?.1;
        let interpreted = reduce(parsed);
        let _ = sender.send(interpreted);
    }
    Result::Ok(())
}
