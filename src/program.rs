use std::collections::HashMap;

pub struct Program {
    pub bpm: f64,
    pub framerate: f64,
    pub instructions: HashMap<Timespan, Instruction>
}

pub enum Timespan {
    All,
    Span(Time, Time),
}
pub enum Instruction {
    PlayVideo(Video),
    PlayTrack(Track),
}

pub struct Video(String);
pub struct Track(usize);

pub struct Time {pub beat: usize, pub frame: usize}
