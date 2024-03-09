use std::time::{Duration, Instant};

pub struct TimeContext {
    init_time: Instant,
    frame_time: Instant,
    durations: Vec<Duration>,
    frame_count: u32
}

const MAX_DELTA_COUNT: usize = 50;

impl TimeContext {
    pub fn new() -> Self {
        let durations  = Vec::from([Duration::from_millis(1)]);

        let init_time = Instant::now();
        let frame_time = init_time.clone();

        Self {
            init_time,
            frame_time,
            durations,
            frame_count: 0
        }
    }

    #[inline]
    pub(crate) fn tick(&mut self) {
        let now = Instant::now();
        let time_since_last = now - self.frame_time;
        self.durations.push(time_since_last);
        self.frame_time = now;
        self.frame_count += 1;

        if self.durations.len() > MAX_DELTA_COUNT {
            self.durations.remove(0);
        }
    }

    #[inline]
    pub fn average_delta(&self) -> Duration {
        self.durations.iter().sum::<Duration>() / self.durations.len() as u32
    }

    #[inline]
    pub fn average_fps(&self) -> f64 {
        1.0 / self.average_delta().as_secs_f64()
    }

    #[inline]
    pub fn ticks(&self) -> u32 {
        self.frame_count
    }

    #[inline]
    pub fn delta(&self) -> Duration {
        self.durations.last().unwrap().clone()
    }

}