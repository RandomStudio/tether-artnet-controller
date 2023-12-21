use std::time::{Duration, SystemTime};

use tween::{Tween, Tweener};

type StoredTweener = Tweener<f32, usize, Box<dyn Tween<f32>>>;

pub struct Animation {
    pub start_time: SystemTime,
    pub duration: Duration,
    pub tweener: StoredTweener,
}

impl Animation {
    pub fn new(
        duration: Duration,
        start_value: f32,
        end_value: f32,
        tween: Box<dyn Tween<f32>>,
    ) -> Self {
        let duration_ms = duration.as_millis() as usize;
        Animation {
            start_time: SystemTime::now(),
            duration,
            tweener: Tweener::new(start_value, end_value, duration_ms, tween),
        }
    }

    /// Update the animation using delta time, get the progress in the range `[0,1]`
    pub fn get_progress(&mut self) -> f32 {
        let elapsed = self.start_time.elapsed().unwrap().as_millis() as usize;

        let progress = self.tweener.move_to(elapsed);
        progress
    }

    pub fn get_progress_and_done(&mut self, delta_time: usize) -> (f32, bool) {
        (self.get_progress(), self.tweener.is_finished())
    }
}
