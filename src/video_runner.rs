use crate::{
    interpreter::{ClipCommand, Time},
    util::time_to_frac,
};

pub struct VideoRunner {
    fps: f64,
    time: f64,
    bpm: f64,
    beats: f64,
    display_state: DisplayState,
    commands: Vec<(Time, ClipCommand)>,
    commands_idx: usize,
    loop_length: f64,
}

enum DisplayState {
    None,
    Single {
        file: String,
        frame: usize,
    },
    Multi {
        file: String,
        sub: usize,
        frame: usize,
        subs_amt: usize,
    },
}

impl VideoRunner {
    pub fn new(fps: f64, bpm: f64, commands: Vec<(Time, ClipCommand)>, loop_length: f64) -> Self {
        Self {
            fps,
            bpm,
            time: 0.0,
            beats: 0.0,
            commands_idx: 0,
            display_state: DisplayState::None,
            commands,
            loop_length,
        }
    }

    pub fn set_commands(&mut self, commands: Vec<(Time, ClipCommand)>, loop_length: f64) {
        self.commands = commands;
        self.loop_length = loop_length;
    }

    pub fn advance_time(&mut self, seconds: f64) -> FrameCommand {
        'find_command: loop {
            if let Some((time, _)) = &self.commands.get(self.commands_idx) {
                if (time.num as f64 / time.denom as f64) > self.beats {
                    break 'find_command;
                }
            };
            if self.commands_idx > self.commands.len() {
                break 'find_command;
            }
            if let Some((_, command)) = &self.commands.get(self.commands_idx) {
                match command {
                    ClipCommand::PlayClip(name) => {
                        self.display_state = DisplayState::Single {
                            file: name.clone(),
                            frame: 0,
                        }
                    }
                    ClipCommand::PlayClipFrom(name, time) => {
                        self.display_state = DisplayState::Single {
                            file: name.clone(),
                            frame: ((time.num as f64 / time.denom as f64) * self.bpm) as usize,
                        }
                    }
                    ClipCommand::PlayMulti(name, subs_amt) => {
                        self.display_state = DisplayState::Multi {
                            file: name.clone(),
                            sub: 0,
                            subs_amt: *subs_amt,
                            frame: 0,
                        }
                    }
                    ClipCommand::PlayMultiFrom(name, time, subs_amt) => {
                        self.display_state = DisplayState::Multi {
                            file: name.clone(),
                            sub: 0,
                            subs_amt: *subs_amt,
                            frame: ((time.num as f64 / time.denom as f64) * self.bpm) as usize,
                        }
                    }
                    ClipCommand::MultiNext => {
                        if let DisplayState::Multi {
                            file,
                            sub,
                            subs_amt,
                            frame,
                        } = &mut self.display_state
                        {
                            *sub += 1;
                            if sub >= subs_amt {
                                *sub = 0;
                            }
                        }
                    }
                };
            }
            self.commands_idx += 1;
        }

        let command = match &self.display_state {
            DisplayState::None => FrameCommand::ShowNone,
            DisplayState::Single { file, frame } => FrameCommand::ShowSingleFrame {
                file: file.clone(),
                frame: *frame,
            },
            DisplayState::Multi {
                file,
                subs_amt,
                sub,
                frame,
            } => FrameCommand::ShowSingleFrame {
                file: format!("{}_{}", file.clone(), sub),
                frame: frame.clone(),
            },
        };

        self.time += seconds;
        self.beats += self.fps * seconds / self.bpm;
        match &mut self.display_state {
            DisplayState::None => (),
            DisplayState::Single { file, frame } => *frame += 1,
            DisplayState::Multi {
                file,
                sub,
                subs_amt,
                frame,
            } => *frame += 1,
        }
        if self.beats > self.loop_length {
            self.beats -= self.loop_length;
            self.time = 0.0;
            self.display_state = DisplayState::None;
            self.commands_idx = 0;
        }

        command
    }
}

#[derive(Debug, Clone)]
pub enum FrameCommand {
    ShowSingleFrame { file: String, frame: usize },
    ShowNone,
}
