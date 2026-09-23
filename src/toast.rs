use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(6);

#[derive(Default)]
pub struct SavedToastTimer {
    deadline: Option<Instant>,
    remaining: Duration,
    paused: bool,
}

impl SavedToastTimer {
    pub fn restart(&mut self, now: Instant, paused: bool) {
        self.remaining = TIMEOUT;
        self.paused = paused;
        self.deadline = (!paused).then_some(now + TIMEOUT);
    }

    pub fn set_paused(&mut self, now: Instant, paused: bool) {
        if self.paused == paused || self.remaining.is_zero() {
            return;
        }
        if paused {
            self.remaining = self
                .deadline
                .map(|deadline| deadline.saturating_duration_since(now))
                .unwrap_or(self.remaining);
            self.deadline = None;
        } else {
            self.deadline = Some(now + self.remaining);
        }
        self.paused = paused;
    }

    pub fn dismiss(&mut self) {
        self.deadline = None;
        self.remaining = Duration::ZERO;
        self.paused = false;
    }

    pub fn expired(&self, now: Instant) -> bool {
        self.deadline.is_some_and(|deadline| now >= deadline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_preserves_remaining_time_and_replacement_restarts_it() {
        let now = Instant::now();
        let mut timer = SavedToastTimer::default();
        timer.restart(now, false);
        timer.set_paused(now + Duration::from_secs(2), true);
        assert!(!timer.expired(now + Duration::from_secs(20)));
        timer.set_paused(now + Duration::from_secs(20), false);
        assert!(!timer.expired(now + Duration::from_secs(23)));
        assert!(timer.expired(now + Duration::from_secs(24)));

        timer.restart(now + Duration::from_secs(25), false);
        assert!(!timer.expired(now + Duration::from_secs(30)));
        assert!(timer.expired(now + Duration::from_secs(31)));
    }

    #[test]
    fn dismiss_clears_paused_state_for_the_next_notification() {
        let now = Instant::now();
        let mut timer = SavedToastTimer::default();
        timer.restart(now, true);
        timer.dismiss();
        timer.restart(now + Duration::from_secs(1), false);
        assert!(timer.expired(now + Duration::from_secs(7)));
    }
}
