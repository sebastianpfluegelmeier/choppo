extern crate ffmpeg_next as ffmpeg;

mod output_instructions;
mod program;
mod transformer;

use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::{Point, Rect};
use sdl2::render::Texture;
use sdl2::surface::Surface;
use std::env;

fn main() -> Result<(), ffmpeg::Error> {
    ffmpeg::init().unwrap();

    if let Ok(mut ictx) = input(&env::args().nth(1).expect("Cannot open file.")) {
        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;
        let video_stream_index = input.index();

        let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
        let mut decoder = context_decoder.decoder().video()?;

        let target_w = 1920/2;
        let target_h = 1080/2;

        let mut scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            target_w,
            target_h,
            Flags::BILINEAR,
        )?;
         

        let mut frame_index = 0;

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("rust-sdl2 demo", target_w, target_h)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        let texture_creator = canvas.texture_creator();

        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut quit_next_frame = false;

        let mut receive_and_process_decoded_frames =
            |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
                let mut decoded = Video::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    if quit_next_frame {
                        decoder.skip_frame(ffmpeg::Discard::All)
                    }
                    break_on_quit(&mut event_pump, &mut quit_next_frame);

                    let mut rgb_frame = Video::empty();
                    scaler.run(&decoded, &mut rgb_frame)?;

                    let stride = rgb_frame.stride(0);

                    let data = rgb_frame.data_mut(0);

                    let surface =
                        Surface::from_data(data, target_w, target_h, stride as u32, PixelFormatEnum::RGB24).unwrap();
                    let texture = Texture::from_surface(&surface, &texture_creator).unwrap();

                    let _ = canvas.copy(&texture, None, None);
                    canvas.present();
                    frame_index += 1;
                }
                Ok(())
            };

        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet)?;
                receive_and_process_decoded_frames(&mut decoder)?;
            }
        }
        decoder.send_eof()?;
        receive_and_process_decoded_frames(&mut decoder)?;
    }

    Ok(())
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
