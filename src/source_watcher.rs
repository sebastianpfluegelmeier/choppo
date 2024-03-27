use filetime::FileTime;

use crate::{
    parser::parse_main,
    reducer::{reduce, ReducedClip},
};
use std::{
    fs,
    sync::mpsc::{channel, Receiver},
    thread::{self, JoinHandle},
};

pub struct SourceWatcher {
    receiver: Receiver<ReducedClip>,
    handle: JoinHandle<()>,
    path: String
}

impl SourceWatcher {
    pub fn new(path: String) -> Self {
        let (_sender, receiver) = channel();
        let handle = thread::spawn(||{});
        Self { receiver, handle, path}
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

fn read_input(input: String, sender: &std::sync::mpsc::Sender<ReducedClip>) -> Result<(), ()> {
    let parsed = parse_main(&input).map_err(|_e| ())?.1;
    let reduced = reduce(parsed);
    reduced.print();
    let _ = sender.send(reduced);
    Result::Ok(())
}
