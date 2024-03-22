extern crate ffmpeg_next as ffmpeg;

use crate::interpreter::Interpreter;
use crate::source_watcher::SourceWatcher;

use std::env;
use video_player::play_video;

mod interpreter;
mod parser;
mod reducer;
mod source_watcher;
mod util;
mod video_loader;
mod video_player;
mod video_reader;

fn main() -> Result<(), ffmpeg::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Please provide a file path as a CLI argument");
        return Ok(());
    }
    let source_watcher = SourceWatcher::new(args[1].to_string());
    let fps = 60.0;
    let runner = Interpreter::new(fps, 120.0, Vec::new(), 1.0);

    let _ = play_video(fps, source_watcher, runner);
    Ok(())
}
