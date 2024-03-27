use ffmpeg_sys_next::AVAdler;
use filetime::FileTime;
use std::fs;
use std::path::PathBuf;

use crate::{
    parser::parse_main,
    reducer::{reduce, ReducedClip},
};
use std::{
    collections::HashSet,
    sync::mpsc::{channel, Receiver},
    thread::{self, JoinHandle},
};

pub struct SourceWatcher {
    receiver: Receiver<ReducedClip>,
    handle: JoinHandle<()>,
    path: String,
}

impl SourceWatcher {
    pub fn new(path: String) -> Self {
        let (_sender, receiver) = channel();
        let handle = thread::spawn(|| {});
        Self {
            receiver,
            handle,
            path,
        }
    }

    pub fn get_new_interpreted(&mut self) -> Option<ReducedClip> {
        let mut interpreted = None;
        for i in self.receiver.try_iter() {
            interpreted = Some(i);
        }
        self.restart();
        interpreted
    }

    fn restart(&mut self) {
        if self.handle.is_finished() {
            let path = self.path.clone();
            let (sender, receiver) = channel();
            let handle = thread::spawn(move || {
                let mut last_timestamp = FileTime::now();

                loop {
                    let timestamp = fs::metadata(path.clone())
                        .map(|m| FileTime::from_last_modification_time(&m))
                        .unwrap_or(FileTime::zero());
                    if last_timestamp != timestamp {
                        let input = fs::read_to_string(path.clone())
                            .map_err(|_e| ())
                            .expect("could not find file");
                        let _ = read_input(input, &sender);
                        last_timestamp = timestamp;
                    }
                }
            });
            self.handle = handle;
            self.receiver = receiver;
        }
    }
}

fn read_file_paths_in_directory(path: &str) -> HashSet<String> {
    let mut file_paths = Vec::new();

    for entry in fs::read_dir(path).unwrap() {
        if entry.is_err() {
            continue;
        }
        let entry = entry.unwrap();
        let file_path = entry.path();

        if file_path.is_file() {
            let path = file_path.to_str().unwrap_or_default().to_string();
            file_paths.push(path);
        }
    }

    file_paths.into_iter().collect()
}

fn read_input(
    input: String,
    sender: &std::sync::mpsc::Sender<ReducedClip>,
) -> Result<(), ()> {
    let parsed = parse_main(&input).map_err(|_e| ())?.1;
    let available_files = read_file_paths_in_directory(&parsed.directory_declaration.directory);
    let reduced = reduce(parsed, &available_files);
    let _ = sender.send(reduced);
    Result::Ok(())
}
