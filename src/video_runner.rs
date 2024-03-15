use crate::{
    interpreter::{ClipCommand, Time},
    util::time_to_frac,
};

pub struct VideoRunner {
    fps: f64,
    time: f64,
    bpm: f64,
    beats: f64,
    current_frame: usize,
    current_file: Option<String>,
    commands: Vec<(Time, ClipCommand)>,
    commands_idx: usize,
    loop_length: f64,
}

impl VideoRunner {
    pub fn new(fps: f64, bpm: f64, commands: Vec<(Time, ClipCommand)>, loop_length: f64) -> Self {
        Self {
            fps,
            bpm,
            time: 0.0,
            beats: 0.0,
            current_frame: 0,
            commands_idx: 0,
            current_file: None,
            commands,
            loop_length
        }
    }

    pub fn advance_time(&mut self, seconds: f64) -> FrameCommand {
        'find_command: loop {
            if let Some((time, _)) = &self.commands.get(self.commands_idx) {
                if (time.num as f64 / time.denom as f64) > self.beats {
                    break 'find_command;
                }
            };
            if self.commands_idx > self.commands.len()  {
                break 'find_command;
            }
            if let Some((_, command)) = &self.commands.get(self.commands_idx) {
                match command {
                    ClipCommand::PlayClip(name) => {
                        self.current_file = Some(name.clone());
                        self.current_frame = 0;
                    }
                    ClipCommand::PlayClipFrom(name, time) => {
                        self.current_file = Some(name.clone());
                        self.current_frame =
                            ((time.num as f64 / time.denom as f64) * self.bpm) as usize
                    }
                };
            }
            self.commands_idx += 1;
        }

        let command = FrameCommand::ShowSingleFrame {
            file: self.current_file.clone(),
            frame: self.current_frame.clone(),
        };

        self.time += seconds;
        self.beats += self.fps * seconds / self.bpm;
        self.current_frame += 1;
        if self.beats > self.loop_length {
            self.beats -= self.loop_length;
            self.time = 0.0;
            self.current_frame = 0;
            self.current_file = None;
            self.commands_idx = 0;
        }

        command
    }
}

#[derive(Debug, Clone)]
pub enum FrameCommand {
    ShowSingleFrame { file: Option<String>, frame: usize },
}
