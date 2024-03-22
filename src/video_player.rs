use std::time::{Duration, Instant};

use crate::interpreter::{self, Interpreter};
use crate::source_watcher::SourceWatcher;
use crate::video_loader::VideoLoader;

use sdl2::keyboard::Keycode;

use sdl2::render::{Texture, TextureValueError};
use sdl2::surface::Surface;
use sdl2::{event::Event, pixels::PixelFormatEnum};

pub fn play_video(
    fps: f64,
    mut source_watcher: SourceWatcher,
    mut runner: Interpreter,
) -> Result<(), ()> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let target_w = 1920 / 2;
    let target_h = 1080 / 2;

    let window = video_subsystem
        .window("visuals", target_w, target_h)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut video_loader = VideoLoader::new(target_w, target_h);
    let mut previous_frame_time = Instant::now();
    let frame_duration = Duration::from_secs_f64(1.0 / fps);
    'mainloop: loop {
        canvas.clear();
        if let Some(clip) = source_watcher.get_new_interpreted() {
            runner.set_commands(clip.commands, clip.length.into());
        }
        let commands = runner.advance_time(1.0 / fps);
        let mut layer = 0;
        for cmd in commands {
            let video = match cmd.clone() {
                interpreter::FrameCommand::ShowSingleFrame { file, frame } => {
                    video_loader.load(&file, frame, layer)
                }
                _ => None,
            };
            if let Some(video) = video {
                let mut texture = frame_to_texture(video, target_w, target_h, &texture_creator)
                    .map_err(|_| ())?;
                texture.set_blend_mode(sdl2::render::BlendMode::Blend);
                canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                let _ = canvas.copy(&texture, None, None);
            }
            layer += 1;
        }

        canvas.present();
        for event in sdl_context.event_pump().map_err(|_e| ())?.poll_iter() {
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
            let sleep_time = frame_duration - elapsed_frame_time;
            std::thread::sleep(sleep_time);
        } else {
            println!("too slow, {:?} overtime", elapsed_frame_time - frame_duration);
        }
        previous_frame_time = Instant::now();
    }
    Ok(())
}

fn frame_to_texture(
    mut rgb_frame: ffmpeg::frame::Video,
    target_w: u32,
    target_h: u32,
    texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
) -> Result<Texture<'_>, TextureValueError> {
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
    let texture = Texture::from_surface(&surface, texture_creator)?;
    Ok(texture)
}
