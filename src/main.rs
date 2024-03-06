extern crate ffmpeg_next as ffmpeg;

mod output_instructions;
mod program;
mod transformer;
mod video_reader;

use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::surface::Surface;
use std::env;
use video_reader::VideoReader;

fn main() -> Result<(), ffmpeg::Error> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let target_w = 1920 / 2;
    let target_h = 1080 / 2;

    let window = video_subsystem
        .window("rust-sdl2 demo", target_w, target_h)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();

    let mut reader = VideoReader::new(
        "/Users/sebastianpfluegelmeier/Desktop/concrete_lowqual.mov".to_string(),
        target_w,
        target_h,
    );

    let mut quit = false;

    'mainloop: loop {
        let mut rgb_frame = reader.read_frame();
        let empty = unsafe { rgb_frame.is_empty() };

        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => {
                    quit = true;
                }
                _ => {}
            }
        }
        if quit {
            break 'mainloop;
        }

        if !empty {
            let stride = rgb_frame.stride(0);

            let data = rgb_frame.data_mut(0);

            let surface = Surface::from_data(
                data,
                target_w,
                target_h,
                stride as u32,
                PixelFormatEnum::RGB24,
            )
            .unwrap();
            let texture = Texture::from_surface(&surface, &texture_creator).unwrap();

            let _ = canvas.copy(&texture, None, None);
            canvas.present();
        }
    }
    panic!();
}

fn break_on_quit(event_pump: &mut sdl2::EventPump, quit_next_frame: &mut bool) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                *quit_next_frame = true;
            }
            _ => {}
        }
    }
}
