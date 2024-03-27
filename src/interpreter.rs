use crate::reducer::{ClipCommand, Time};

pub struct Interpreter {
    fps: f64,
    time: f64,
    bpm: f64,
    beats: f64,
    display_state: Vec<DisplayState>,
    commands: Vec<(Time, ClipCommand)>,
    commands_idx: usize,
    loop_length: f64,
}

#[derive(Debug)]
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
        extension: String,
    },
}

impl Interpreter {
    pub fn new(fps: f64, bpm: f64, commands: Vec<(Time, ClipCommand)>, loop_length: f64) -> Self {
        Self {
            fps,
            bpm,
            time: 0.0,
            beats: 0.0,
            commands_idx: 0,
            display_state: vec![DisplayState::None],
            commands,
            loop_length,
        }
    }

    pub fn set_commands(&mut self, commands: Vec<(Time, ClipCommand)>, loop_length: f64) {
        self.commands = commands;
        self.loop_length = loop_length;
    }

    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = bpm;
    }

    pub fn reset_beat(&mut self) {
        self.beats = 0.0;
        self.time = 0.0;
        self.display_state.clear();
        self.commands_idx = 0;
    }

    pub fn advance_time(&mut self, seconds: f64) -> Vec<FrameCommand> {
        'find_command: loop {
            if let Some((time, _)) = &self.commands.get(self.commands_idx) {
                if (time.num as f64 / time.denom as f64) > self.beats {
                    break 'find_command;
                }
            };
            if self.commands_idx > self.commands.len() {
                break 'find_command;
            }
            if let Some((_, command)) = self.commands.get(self.commands_idx) {
                let layer = command.layer();
                if self.display_state.len() <= layer {
                    self.display_state.push(DisplayState::None);
                }
                match command {
                    ClipCommand::PlayClip(name, layer) => {
                        self.display_state[*layer] = DisplayState::Single {
                            file: name.clone(),
                            frame: 0,
                        }
                    }
                    ClipCommand::PlayClipFrom(name, layer, time) => {
                        self.display_state[*layer] = DisplayState::Single {
                            file: name.clone(),
                            frame: ((time.num as f64 / time.denom as f64) * self.bpm) as usize,
                        }
                    }
                    ClipCommand::PlayMulti(name, subs_amt, extension) => {
                        self.display_state[0] = DisplayState::Multi {
                            file: name.clone(),
                            sub: 0,
                            subs_amt: *subs_amt,
                            frame: 0,
                            extension: extension.clone(),
                        }
                    }
                    ClipCommand::PlayMultiFrom(name, time, subs_amt, extension) => {
                        self.display_state[0] = DisplayState::Multi {
                            file: name.clone(),
                            sub: 0,
                            subs_amt: *subs_amt,
                            frame: ((time.num as f64 / time.denom as f64) * self.bpm) as usize,
                            extension: extension.clone(),
                        }
                    }
                    ClipCommand::MultiNext(layer) => {
                        if let Some(DisplayState::Multi {
                            file: _,
                            sub,
                            subs_amt,
                            frame: _,
                            extension: _,
                        }) = &mut self.display_state.get_mut(*layer)
                        {
                            *sub += 1;
                            if sub >= subs_amt {
                                *sub = 0;
                            }
                        }
                    }
                    ClipCommand::Stop(layer) => self.display_state[*layer] = DisplayState::None,
                };
            }
            self.commands_idx += 1;
        }

        let mut commands = Vec::new();
        for display_state in &self.display_state {
            let command = match display_state {
                DisplayState::None => FrameCommand::ShowNone,
                DisplayState::Single { file, frame } => FrameCommand::ShowSingleFrame {
                    file: file.clone(),
                    frame: *frame,
                },
                DisplayState::Multi {
                    file,
                    subs_amt: _,
                    sub,
                    frame,
                    extension,
                } => FrameCommand::ShowSingleFrame {
                    file: format!("{}_{}{}", file.clone(), sub, extension),
                    frame: *frame,
                },
            };
            commands.push(command);
        }

        self.time += seconds;
        self.beats += self.bpm * seconds / (self.fps * 4.0);
        for display_state in &mut self.display_state {
            match display_state {
                DisplayState::None => (),
                DisplayState::Single { file: _, frame } => *frame += 1,
                DisplayState::Multi {
                    file: _,
                    sub: _,
                    subs_amt: _,
                    frame,
                    extension: _,
                } => *frame += 1,
            }
        }
        if self.beats > self.loop_length {
            self.beats -= self.loop_length;
            self.time = 0.0;
            self.display_state.clear();
            self.commands_idx = 0;
        }

        commands
    }
}

#[derive(Debug, Clone)]
pub enum FrameCommand {
    ShowSingleFrame { file: String, frame: usize },
    ShowNone,
}
