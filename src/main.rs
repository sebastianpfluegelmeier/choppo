extern crate ffmpeg_next as ffmpeg;

mod output_instructions;
mod parser;
mod program;
mod transformer;
mod video_reader;
mod interpreter;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::surface::Surface;
use std::time::{Duration, Instant};
use video_reader::VideoReader;

use crate::{interpreter::interpret, parser::parse_main};

fn main() -> Result<(), ffmpeg::Error> {
    let input = "
        directory = 'dir';
        extension = '.mov';
        beat a = .-..-..-;
        beat b = 332;
        beat c = a | b;
        'vid'
    ";

    let parsed = parse_main(input).unwrap().1;
    // println!("parsed {:?}", parsed);
    interpret(parsed);
    todo!()
    // let sdl_context = sdl2::init().unwrap();
    // let video_subsystem = sdl_context.video().unwrap();

    // let target_w = 1920 / 2;
    // let target_h = 1080 / 2;

    // let window = video_subsystem
    //     .window("visuals", target_w, target_h)
    //     .position_centered()
    //     .build()
    //     .unwrap();

    // let mut canvas = window.into_canvas().build().unwrap();

    // let texture_creator = canvas.texture_creator();
    // let files = vec![
    //     "/Users/sebastianpfluegelmeier/v1.mov",
    //     "/Users/sebastianpfluegelmeier/v2.mov",
    //     "/Users/sebastianpfluegelmeier/v3.mov",
    //     "/Users/sebastianpfluegelmeier/v4.mov",
    // ];
    // let mut readers: Vec<VideoReader> = files
    //     .iter()
    //     .map(|f| VideoReader::new(f.to_string(), target_w, target_h))
    //     .collect();

    // let mut frame_nr = 0;

    // const FPS: u32 = 60;
    // let frame_duration: Duration = Duration::from_secs(1) / FPS;

    // let mut previous_frame_time = Instant::now();

    // 'mainloop: loop {
    //     let base_frame = readers[0].read_frame();
    //     let rgb_frame2 = readers[1].read_frame();
    //     let rgb_frame3 = readers[2].read_frame();
    //     let rgb_frame4 = readers[3].read_frame();
    //     let rgb_frame = match (frame_nr / 4) % 3 {
    //         0 => rgb_frame2,
    //         1 => rgb_frame3,
    //         _ => rgb_frame4,
    //     };

    //     if let (Some(base_frame), Some(rgb_frame)) = (base_frame, rgb_frame) {
    //         frame_nr += 1;
    //         let base_texture = frame_to_texture(base_frame, target_w, target_h, &texture_creator);
    //         let mut texture = frame_to_texture(rgb_frame, target_w, target_h, &texture_creator);
    //         texture.set_alpha_mod(100);

    //         let _ = canvas.copy(&base_texture, None, None);
    //         let _ = canvas.copy(&texture, None, None);
    //         canvas.present();
    //     } else {
    //         break 'mainloop;
    //     }

    //     for event in sdl_context.event_pump().unwrap().poll_iter() {
    //         match event {
    //             Event::Quit { .. }
    //             | Event::KeyDown {
    //                 keycode: Option::Some(Keycode::Escape),
    //                 ..
    //             } => {
    //                 break 'mainloop;
    //             }
    //             _ => {}
    //         }
    //     }

    //     let elapsed_frame_time = previous_frame_time.elapsed();
    //     if elapsed_frame_time < frame_duration {
    //         std::thread::sleep(frame_duration - elapsed_frame_time);
    //     }
    //     previous_frame_time = Instant::now();
    // }
    // 'bloop: loop {
    //     for event in sdl_context.event_pump().unwrap().poll_iter() {
    //         match event {
    //             Event::Quit { .. }
    //             | Event::KeyDown {
    //                 keycode: Option::Some(Keycode::Escape),
    //                 ..
    //             } => break 'bloop,
    //             _ => {}
    //         }
    //     }
    // }
    // panic!();
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
