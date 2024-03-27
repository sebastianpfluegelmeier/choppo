

use std::time::{Duration};

use crate::bpm_controller::{BpmController};
use crate::interpreter::{self, Interpreter};
use crate::source_watcher::SourceWatcher;
use crate::time_controller::{TimeController};
use crate::video_loader::VideoLoader;


use sdl2::keyboard::Keycode;

use sdl2::render::{Texture, TextureValueError};
use sdl2::surface::Surface;
use sdl2::{event::Event, pixels::PixelFormatEnum};

pub fn play_video(
    fps: f64,
    mut source_watcher: SourceWatcher,
    mut runner: Interpreter,
    mut bpm_controller: BpmController,
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
    let _frame_duration = Duration::from_secs_f64(1.0 / fps);
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut time_controller = TimeController::new(fps);
    'mainloop: loop {
        time_controller.frame_start();

        if bpm_controller.get_quit() {
            break 'mainloop;
        }
        if bpm_controller.get_reset() {
            runner.reset_beat();
        }
        runner.set_bpm(bpm_controller.get_bpm());
        bpm_controller.tick();
        if let Some(clip) = source_watcher.get_new_interpreted() {
            runner.set_commands(clip.commands, clip.length.into());
        }
        let commands = runner.advance_time(1.0 / fps);
        if !time_controller.skip_frame() {
            canvas.clear();
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
        } 

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => {
                    break 'mainloop;
                }
                Event::KeyDown { .. } => bpm_controller.consume_event(&event),
                Event::KeyUp { .. } => bpm_controller.consume_event(&event),
                _ => {}
            }
        }
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
