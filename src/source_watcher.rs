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
    receiver: Receiver<ReducedClip>,
}

impl SourceWatcher {
    pub fn new(path: String) -> Self {
        let (sender, receiver) = channel();

        thread::spawn(move || {
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

        Self { receiver }
    }

    pub fn get_new_interpreted(&mut self) -> Option<ReducedClip> {
        let mut interpreted = None;
        for i in self.receiver.try_iter() {
            interpreted = Some(i);
        }
        interpreted
    }
}

fn read_input(input: String, sender: &std::sync::mpsc::Sender<ReducedClip>) -> Result<(), ()> {
    let parsed = parse_main(&input).map_err(|_e| ())?.1;
    for declaration in &parsed.declarations {
        println!("{:?}", declaration);
    }
    println!("{:?}\n", parsed.main_expression);
    let interpreted = reduce(parsed);
    interpreted.print();
    let _ = sender.send(interpreted);
    Result::Ok(())
}
