
use ::uom::temp::Temperature as T;
use ::uom::temp::*;
use chrono::*;
use std::ops::Range;

/// Intended to be copied into Controller, not moved or borrowed
#[derive(Clone, Copy)]
pub struct Config {
    pub max_temp: Temperature<F>,
    pub min_temp: Temperature<F>,
    hold_end: Option<DateTime<UTC>>,
    fan_end: Option<DateTime<UTC>>,
}

// TODO: not sure if this belongs here or in the Controller. It protects the compressor, so it may even
// belong there
const HOLD_RANGE: Range<f32> = Range { start: -0.5, end: 0.9 };

#[derive(Clone, Copy)]
pub struct ScheduleLeg {
    weekday: Weekday,
    active_range: Range<NaiveTime>,
}

impl Config {
    pub fn new(max_temp: T<F>, min_temp: T<F>) -> Config  {
        Config {
            max_temp: max_temp,
            min_temp: min_temp,
            hold_end: None,
            fan_end: None
        }
    }

    pub fn set_hold_mode(&mut self, timeout: Duration) {
        self.hold_end = Some(UTC::now() + timeout);
    }

    pub fn set_fan_on(&mut self, timeout: Duration) {
        self.fan_end = Some(UTC::now() + timeout);
    }

    pub fn cancel_hold_mode(&mut self) {
        self.hold_end = None
    }

    pub fn cancel_fan_mode(&mut self) {
        self.fan_end = None
    }

    pub fn is_fan_on(&self, time: DateTime<UTC>) -> bool {
        match self.fan_end {
            Some(fan_end) => time < fan_end,
            None => false 
        }
    }

    pub fn is_hold_mode(&self, time: DateTime<UTC>) -> bool {
        match self.hold_end {
            Some(hold_end) => time < hold_end,
            None => false 
        }
    }

    /// Returns a tuple of (minRange, maxRange) specifying the allowable ranges of temperatures
    /// before turning on AC, Heat, ETC
    pub fn get_temp_ranges(&self) -> (Range<T<F>>, Range<T<F>>) {
        let temp_range = T::in_f(HOLD_RANGE.start)..T::in_f(HOLD_RANGE.end);

        (
            (self.min_temp - temp_range.end)..(self.min_temp - temp_range.start),
            (self.max_temp + temp_range.start)..(self.max_temp + temp_range.end)
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::*;
    use ::uom::temp::Temperature as T;

    #[test] 
    fn fan_on_when_within_timeout() {
        let mut config = Config::new(T::in_f(25.0), T::in_f(24.0)); 

        config.set_fan_on(Duration::seconds(10));

        assert!(config.is_fan_on(UTC::now() + Duration::seconds(9)));
    }

    #[test] 
    fn fan_off_when_timeout_exceeded() {
        let mut config = Config::new(T::in_f(25.0), T::in_f(24.0)); 

        config.set_fan_on(Duration::seconds(10));

        assert!(!config.is_fan_on(UTC::now() + Duration::seconds(11)));
    }

    #[test]
    fn fan_off_when_no_timeout_set() {
        let mut config = Config::new(T::in_f(25.0), T::in_f(24.0)); 

        assert!(!config.is_fan_on(UTC::now()));
    }

    #[test] 
    fn hold_on_when_within_timeout() {
        let mut config = Config::new(T::in_f(25.0), T::in_f(24.0)); 

        config.set_hold_mode(Duration::seconds(10));

        assert!(config.is_hold_mode(UTC::now() + Duration::seconds(9)));
    }

    #[test] 
    fn hold_off_when_timeout_exceeded() {
        let mut config = Config::new(T::in_f(25.0), T::in_f(24.0)); 

        config.set_hold_mode(Duration::seconds(10));

        assert!(!config.is_hold_mode(UTC::now() + Duration::seconds(11)));
    }

    #[test]
    fn hold_off_when_no_timeout_set() {
        let mut config = Config::new(T::in_f(25.0), T::in_f(24.0)); 

        assert!(!config.is_hold_mode(UTC::now()));
    }
}

