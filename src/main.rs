extern crate ffmpeg_next as ffmpeg;

mod output_instructions;
mod program;
mod transformer;
mod video_reader;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Texture;
use sdl2::surface::Surface;
use video_reader::VideoReader;

fn main() -> Result<(), ffmpeg::Error> {
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

    let mut reader1 = VideoReader::new(
        "/Users/sebastianpfluegelmeier/v1.mov".to_string(),
        target_w,
        target_h,
    );
    let mut reader2 = VideoReader::new(
        "/Users/sebastianpfluegelmeier/v2.mov".to_string(),
        target_w,
        target_h,
    );
    let mut reader3 = VideoReader::new(
        "/Users/sebastianpfluegelmeier/v3.mov".to_string(),
        target_w,
        target_h,
    );
    let mut reader4 = VideoReader::new(
        "/Users/sebastianpfluegelmeier/v4.mov".to_string(),
        target_w,
        target_h,
    );


    let mut frame_nr = 0;
    'mainloop: loop {
        let rgb_frame1 = reader1.read_frame();
        let rgb_frame2 = reader2.read_frame();
        let rgb_frame3 = reader3.read_frame();
        let rgb_frame4 = reader4.read_frame();
        let mut rgb_frame = match (frame_nr/4) % 4 {
            0 => rgb_frame1,
            1 => rgb_frame2,
            2 => rgb_frame3,
            _ => rgb_frame4,
        };
        let empty = unsafe { rgb_frame.is_empty() };


        if !empty {
            frame_nr += 1;
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
    }
    panic!();
}
