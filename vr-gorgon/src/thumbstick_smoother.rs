use std::cmp::Ordering;
use std::time::Instant;

const HYSTERESIS_LOW: f32 = 0.2;
const HYSTERESIS_HIGH: f32 = 0.5;
const REPEAT_PERIOD: f32 = 0.5;

pub struct ThumbstickSmoother {
    last_pulse: Option<Instant>,
    last_state: Ordering,
}

impl Default for ThumbstickSmoother {
    fn default() -> Self {
        Self {
            last_pulse: None,
            last_state: Ordering::Equal,
        }
    }
}

impl ThumbstickSmoother {
    pub fn smooth_input(&mut self, delta: f32) -> Ordering {
        match self.last_state {
            Ordering::Greater => {
                // our last smoothed input was a + pulse
                if delta < -HYSTERESIS_HIGH {
                    self.last_state = Ordering::Less;
                    self.last_pulse = Some(Instant::now());
                    Ordering::Less
                } else if delta < HYSTERESIS_LOW {
                    self.last_state = Ordering::Equal;
                    Ordering::Equal
                } else {
                    // still high
                    self.maybe_autorepeat(Ordering::Greater)
                }
            }
            Ordering::Less => {
                // our last smoothed input was a - pulse
                if delta > HYSTERESIS_HIGH {
                    self.last_state = Ordering::Greater;
                    self.last_pulse = Some(Instant::now());
                    Ordering::Greater
                } else if delta > -HYSTERESIS_LOW {
                    self.last_state = Ordering::Equal;
                    Ordering::Equal
                } else {
                    // still low
                    self.maybe_autorepeat(Ordering::Less)
                }
            }
            Ordering::Equal => {
                // our last smoothed input was neutral
                if delta < -HYSTERESIS_HIGH {
                    self.last_state = Ordering::Less;
                    self.last_pulse = Some(Instant::now());
                    Ordering::Less
                } else if delta > HYSTERESIS_HIGH {
                    self.last_state = Ordering::Greater;
                    self.last_pulse = Some(Instant::now());
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            }
        }
    }

    fn maybe_autorepeat(&mut self, repeat_val: Ordering) -> Ordering {
        let now = Instant::now();
        match self.last_pulse {
            None => {
                self.last_pulse = Some(now);
                Ordering::Equal
            }
            Some(last_pulse) => {
                if now.duration_since(last_pulse).as_secs_f32() >= REPEAT_PERIOD {
                    self.last_pulse = Some(now);
                    repeat_val
                } else {
                    Ordering::Equal
                }
            }
        }
    }
}
