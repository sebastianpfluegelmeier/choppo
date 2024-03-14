extern crate ffmpeg_next as ffmpeg;

use std::time::{Duration, Instant};

use crate::video_loader::VideoLoader;
use crate::video_runner::VideoRunner;
use crate::{interpreter::interpret, parser::parse_main, video_reader::VideoReader};
use sdl2::keyboard::Keycode;
use sdl2::render::Texture;
use sdl2::surface::Surface;
use sdl2::{event::Event, pixels::PixelFormatEnum};

mod interpreter;
mod parser;
mod program;
mod util;
mod video_loader;
mod video_reader;
mod video_runner;

fn main() -> Result<(), ffmpeg::Error> {
    let input = "
        directory = '/Users/sebastianpfluegelmeier/test/';
        extension = '.mp4';
        clip a = 'a'[1.1:1.3] | 'b';
        a[:4]
    ";

    let parsed = parse_main(input).unwrap().1;
    println!("{:?}", parsed);
    let path = parsed.directory_declaration.directory.clone();
    let extension = parsed.extension_declaration.extension.clone();
    let interpreted = interpret(parsed);
    println!("interpreted {:?}", interpreted);
    let mut runner = VideoRunner::new(60.0, 120.0, interpreted.commands,interpreted.length.into());

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let target_w = 1920 / 2;
    let target_h = 1080 / 2;

    let window = video_subsystem
        .window("visuals", target_w, target_h)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut video_loader = VideoLoader::new(target_w, target_h);
    let mut previous_frame_time = Instant::now();
    let frame_duration = Duration::from_secs_f64(1.0 / 60.0);
    'mainloop: loop {
        let cmd = runner.advance_time(1.0 / 60.0);
        let video = match cmd {
            video_runner::FrameCommand::ShowSingleFrame { file, frame } => {
                video_loader.load(&format!("{}{}{}", &path, &file, &extension), frame)
            }
        };

        if let Some(video) = video {
            let texture = frame_to_texture(video, target_w, target_h, &texture_creator);

            let _ = canvas.copy(&texture, None, None);
        }
        canvas.present();

        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => {
                    break 'mainloop;
                }
                _ => {}
            }
        }

        let elapsed_frame_time = previous_frame_time.elapsed();
        if elapsed_frame_time < frame_duration {
            std::thread::sleep(frame_duration - elapsed_frame_time);
        }
        previous_frame_time = Instant::now();
    }
    'bloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'bloop,
                _ => {}
            }
        }
    }
    panic!();
}

fn frame_to_texture(
    mut rgb_frame: ffmpeg::frame::Video,
    target_w: u32,
    target_h: u32,
    texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
) -> Texture<'_> {
    let stride = rgb_frame.stride(0);

    let data = rgb_frame.data_mut(0);

    let surface = Surface::from_data(
        data,
        target_w,
        target_h,
        stride as u32,
        PixelFormatEnum::RGBA32,
    )
    .unwrap();
    let texture = Texture::from_surface(&surface, texture_creator).unwrap();
    texture
}
