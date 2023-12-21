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

    /// Update the animation using delta time, get the value in the range `[0,1]`
    pub fn get_value(&mut self) -> f32 {
        let elapsed = self.start_time.elapsed().unwrap().as_millis() as usize;

        self.tweener.move_to(elapsed)
    }

    pub fn get_progress(&self) -> f32 {
        self.tweener.current_time as f32 / self.tweener.duration as f32
    }

    pub fn get_value_and_done(&mut self) -> (f32, bool) {
        (self.get_value(), self.tweener.is_finished())
    }
}
