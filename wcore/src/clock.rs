use instant::Instant;

use crate::time::Time;

pub trait Clock {
    fn set_time(&mut self, time: Time);
    fn get_time(&mut self) -> Time;

    fn is_paused(&self) -> bool;
    fn set_paused(&mut self, value: bool, time: Time);
    fn toggle_paused(&mut self, time: Time);

    fn set_length(&mut self, value: Time);
    fn get_length(&self) -> Time;
}

pub struct SyncClock {
    last_pause: Instant,
    last_time: Time,
    paused: bool,
    length: Time,
}

impl SyncClock {
    pub fn new() -> Self {
        return Self {
            last_pause: Instant::now(),
            last_time: Time::zero(),
            paused: true,
            length: Time::zero(),
        };
    }
}

impl Clock for SyncClock {
    fn set_time(&mut self, time: Time) {
        self.last_pause = Instant::now();
        self.last_time = time;
    }

    fn get_time(&mut self) -> Time {
        if self.paused {
            return self.last_time;
        } else {
            let now = instant::Instant::now();
            let diff = now.duration_since(self.last_pause);
            let time = self.last_time + diff.into();

            if time >= self.length {
                self.paused = true;
                self.last_time = self.length;
            }

            return time;
        }
    }

    fn is_paused(&self) -> bool { return self.paused; }
    fn set_paused(&mut self, value: bool, time: Time) {
        self.paused = value;

        self.last_pause = Instant::now();
        self.last_time = time;
     }

    fn toggle_paused(&mut self, time: Time) {
        self.paused = !self.paused;
        
        self.last_pause = Instant::now();
        self.last_time = time;
    }
    
    fn set_length(&mut self, value: Time) { self.length = value; }
    fn get_length(&self) -> Time { return self.length; }
}