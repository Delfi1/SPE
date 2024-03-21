use std::time::{Duration, Instant};

pub struct TimeContext {
    init_time: Instant,
    current: Instant,
    frames: Vec<Duration>,
}

const MAX_FRAMES: usize = 200;

impl TimeContext {
    pub(super) fn new() -> Self {
        let init_time = Instant::now();
        let current = init_time.clone();
        let frames = Vec::from([Duration::from_secs_f64(0.01)]);

        Self { init_time, current, frames }
    }

    pub(in crate::engine) fn tick(&mut self) {
        let now = Instant::now();
        let elapsed = self.current.elapsed();
        self.current = now;

        self.frames.push(elapsed);
        if self.frames.len() >= MAX_FRAMES {
            self.frames.remove(0);
        }
    }

    pub fn init_time(&self) -> Duration {
        self.init_time.elapsed()
    }

    pub fn average_delta(&self) -> Duration {
        let sum: Duration = self.frames.iter().sum();
        let len = self.frames.len() as u32;

        sum / len
    }

    pub fn delta(&self) -> Duration {
        self.frames.last().unwrap().clone()
    }

    pub fn average_fps(&self) -> u32 {
        let average = 1.0 / self.average_delta().as_secs_f64();
        average.round() as u32
    }
}