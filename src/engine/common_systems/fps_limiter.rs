use std::time::{Duration, Instant};

use foundry::*;

#[derive(AsAny)]
pub struct FpsLimiter {
    min_frame_time: Duration,
    last_frame_time: Instant,
}

impl FpsLimiter {
    pub fn new(max_fps_count: f32) -> System {
        System::new(
            Self {
                min_frame_time: Duration::from_secs_f32(1. / max_fps_count),
                last_frame_time: Instant::now(),
            },
            UpdateFrequency::Fixed(1. / max_fps_count),
        )
    }
}

impl Updatable for FpsLimiter {
    fn update(&mut self, _components: &mut ComponentTable, _delta: f32) {
        // ! fixme: weird slow down behaviour
        let now = Instant::now();
        let since_last_frame = now - self.last_frame_time;
        if self.min_frame_time > since_last_frame {
            println!("sleeping for {:?}", self.min_frame_time - since_last_frame);
            std::thread::sleep(self.min_frame_time - since_last_frame);
        }
        self.last_frame_time = Instant::now();
    }
}