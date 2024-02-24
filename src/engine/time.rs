use std::time::{Duration, Instant};

pub struct TimeContext {
    init_instant: Instant,
    last_instant: Instant,
    frame_durations: Vec<Duration>,
    frame_count: usize,
}

const MAX_DELTA_COUNT: usize = 200;

impl TimeContext {
    pub(super) fn new() -> Self {
        let init_instant = Instant::now();
        let last_instant = init_instant.clone();
        let frame_count = 0;

        let frame_durations = Vec::from([Duration::from_millis(1)]);

        Self { init_instant, last_instant, frame_durations, frame_count }
    }

    pub(super) fn raw_delta(&self) -> Duration {
        Instant::now().duration_since(self.last_instant)
    }

    pub fn delta(&self) -> Duration {
        self.frame_durations.last().unwrap().clone()
    }

    pub fn average_delta(&self) -> Duration {
        let sum: Duration = self.frame_durations.iter().sum();

        sum / u32::try_from(self.frame_durations.len()).unwrap()
    }

    pub fn fps(&self) -> f64 {
        let duration_frame = self.delta();
        let seconds_frame = duration_frame.as_secs_f64();
        1.0 / seconds_frame
    }

    pub fn average_fps(&self) -> f64 {
        let duration_per_frame = self.average_delta();
        let seconds_per_frame = duration_per_frame.as_secs_f64();
        1.0 / seconds_per_frame
    }

    pub fn time_since_start(&self) -> Duration {
        self.init_instant.elapsed()
    }

    pub fn ticks(&self) -> usize {
        self.frame_count
    }

    pub(super) fn tick(&mut self) {
        let now = Instant::now();
        let time_since_last = now - self.last_instant;
        self.frame_durations.push(time_since_last);
        self.last_instant = now;
        self.frame_count += 1;

        if self.frame_durations.len() > MAX_DELTA_COUNT {
            self.frame_durations.remove(0);
        }
    }
}