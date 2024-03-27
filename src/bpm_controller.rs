use sdl2::{event::Event, keyboard::Keycode, sys::KeyCode};
use std::sync::mpsc::{Receiver, Sender};

pub struct BpmController {
    bpm: f64,
    temp_bpm_offset: f64,
    ctrl: bool,
    option: bool,
    command: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    accelleration: f64,
    reset: bool,
    quit: bool,
}

pub enum BpmMessage {
    ResetBeat,
    Quit,
    SetBpm(f64),
}

impl BpmController {
    pub fn new(init_bpm: f64) -> Self {
        BpmController {
            bpm: init_bpm,
            temp_bpm_offset: 0.0,
            ctrl: false,
            option: false,
            command: false,
            left: false,
            right: false,
            up: false,
            down: false,
            reset: false,
            quit: false,
            accelleration: 0.0
        }
    }

    pub fn tick(&mut self) {
        if self.command && self.ctrl && self.option {
            if self.up {
                self.bpm += self.accelleration;
                self.accelleration += 0.0006;
                println!("bpm: {:.2}", self.bpm);
            }
            if self.down {
                self.bpm -= self.accelleration;
                self.accelleration += 0.0006;
                println!("bpm: {:.2}", self.bpm);
            }
            if self.left {
                self.temp_bpm_offset += 0.03;
                self.temp_bpm_offset = self.temp_bpm_offset.min(20.0);
            }
            if self.right {
                self.temp_bpm_offset -= 0.03;
                self.temp_bpm_offset = self.temp_bpm_offset.max(-20.0);
            }
        } else {
            self.accelleration = 0.0;
            self.temp_bpm_offset = 0.0;
        }

    }

    pub fn get_bpm(&self) -> f64 {
        self.bpm + self.temp_bpm_offset
    }

    pub fn get_reset(&mut self) -> bool {
        let r = self.reset;
        self.reset = false;
        r
    }

    pub fn get_quit(&self) -> bool {
        self.quit
    }

    pub fn consume_event(&mut self, event: &Event) {
        match event {
            Event::KeyDown{ keycode: Some(Keycode::LAlt), ..} => self.option = true,
            Event::KeyUp{keycode: Some(Keycode::LAlt), ..} => self.option = false,
            Event::KeyDown{ keycode: Some(Keycode::LCtrl), ..} => self.ctrl = true,
            Event::KeyUp{keycode: Some(Keycode::LCtrl), ..} => self.ctrl = false,
            Event::KeyDown{ keycode: Some(Keycode::LGui), ..} => self.command = true,
            Event::KeyUp{keycode: Some(Keycode::LGui), ..} => self.command = false,
            Event::KeyDown{ keycode: Some(Keycode::Left), ..} => self.left = true,
            Event::KeyUp{keycode: Some(Keycode::Left), ..} => self.left = false,
            Event::KeyDown{ keycode: Some(Keycode::Right), ..} => self.right = true,
            Event::KeyUp{keycode: Some(Keycode::Right), ..} => self.right = false,
            Event::KeyDown{ keycode: Some(Keycode::Up), ..} => self.up = true,
            Event::KeyUp{keycode: Some(Keycode::Up), ..} => self.up = false,
            Event::KeyDown{ keycode: Some(Keycode::Down), ..} => self.down = true,
            Event::KeyUp{keycode: Some(Keycode::Down), ..} => self.down = false,
            Event::KeyDown{ keycode: Some(Keycode::R), ..} => self.bpm = self.bpm.round(),
            Event::KeyDown{ keycode: Some(Keycode::Space), ..} => {
                self.reset = true;
            }
            Event::KeyDown{keycode: Some(Keycode::X), ..} => {
                self.quit = true;
            }
            _ => ()
        }
    }
}
