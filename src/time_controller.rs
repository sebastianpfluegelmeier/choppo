use std::{
    thread::sleep,
    time::{Duration, Instant},
};

// TODO: fix this, somethings fucked here

pub struct TimeController {
    frame_start_time: Instant,
    last_frame_start_time: Instant,
    overtime: Duration,
    fps: f64,
}

impl TimeController {
    pub fn new(fps: f64) -> Self {
        Self {
            frame_start_time: Instant::now(),
            last_frame_start_time: Instant::now(),
            overtime: Duration::ZERO,
            fps,
        }
    }

    pub fn frame_start(&mut self) {
        self.last_frame_start_time = self.frame_start_time;
        self.frame_start_time = Instant::now();
        let last_frame_duration = self
            .frame_start_time
            .duration_since(self.last_frame_start_time);
        let mut sleep_time = Duration::ZERO;
        if last_frame_duration < self.frame_duration() {
            sleep_time += self.frame_duration() - last_frame_duration;
        }
        if self.overtime < sleep_time {
            sleep_time -= self.overtime;
            self.overtime = Duration::ZERO;
        } else {
            self.overtime -= sleep_time;
            sleep_time = Duration::ZERO;
        }
        sleep(sleep_time);
    }

    pub fn skip_frame(&self) -> bool {
        self.overtime > self.frame_duration() * 2
    }

    fn frame_duration(&self) -> Duration {
        Duration::from_secs_f64(1.0 / self.fps)
    }
}
