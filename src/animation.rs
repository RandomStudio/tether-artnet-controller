use std::time::{Duration, SystemTime};

use egui::Color32;
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

pub fn animate_colour(start_colour: &Color32, end_colour: &Color32, progress: f32) -> Color32 {
    // TODO: could just use an array here
    let r = linear_interpolate_u8(start_colour.r(), end_colour.r(), progress);
    let g = linear_interpolate_u8(start_colour.g(), end_colour.g(), progress);
    let b = linear_interpolate_u8(start_colour.b(), end_colour.b(), progress);
    let a = linear_interpolate_u8(start_colour.a(), end_colour.a(), progress);
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

fn linear_interpolate_u8(a: u8, b: u8, t: f32) -> u8 {
    let v = a as f32 + t * (b as f32 - a as f32);
    v as u8
}
