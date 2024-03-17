extern crate ffmpeg_next as ffmpeg;

use std::time::{Duration, Instant};

use crate::source_watcher::SourceWatcher;
use crate::video_loader::VideoLoader;
use crate::interpreter::Interpreter;

use sdl2::keyboard::Keycode;
use sdl2::render::Texture;
use sdl2::surface::Surface;
use sdl2::{event::Event, pixels::PixelFormatEnum};
use video_player::play_video;
use std::env;


mod reducer;
mod parser;
mod source_watcher;
mod util;
mod video_loader;
mod video_reader;
mod interpreter;
mod video_player;

fn main() -> Result<(), ffmpeg::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Please provide a file path as a CLI argument");
        return Ok(());
    }
    let source_watcher = SourceWatcher::new(args[1].to_string());
    let interpreted = source_watcher.get_first_interpreted();
    let filepath = source_watcher.get_file_path().clone();
    let extension = source_watcher.get_extension().clone();
    let fps = 60.0;
    let runner = Interpreter::new(fps, 120.0, interpreted.commands, interpreted.length.into());

    play_video(fps, source_watcher, runner, filepath, extension);
    Ok(())
}