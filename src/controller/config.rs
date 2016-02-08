
use ::uom::temp::Temperature as T;
use ::uom::temp::*;
use chrono::*;
use std::ops::Range;

/// Intended to be copied into Controller, not moved or borrowed
#[derive(Clone)]
pub struct Config {
    pub max_temp: Temperature<F>,
    pub min_temp: Temperature<F>,
    hold_end: Option<DateTime<UTC>>,
    fan_end: Option<DateTime<UTC>>,
    schedule: Schedule,
}

// TODO: not sure if this belongs here or in the Controller. It protects the compressor, so it may even
// belong there
const HOLD_RANGE: Range<f32> = Range { start: -0.5, end: 0.9 };

#[derive(Clone)]
pub struct ScheduleLeg {
    pub min_temp: Temperature<F>,
    pub max_temp: Temperature<F>,
    pub weekdays: Vec<Weekday>,
    pub active_range: Range<NaiveTime>,
}

#[derive(Clone)]
pub struct Schedule {
    legs: Vec<ScheduleLeg>,
}

impl Schedule {
    pub fn new(legs: Vec<ScheduleLeg>) -> Schedule {
        Schedule { legs: legs }
    }

    pub fn get_active_leg(&self, current_datetime: DateTime<UTC>) -> Option<&ScheduleLeg> {
        let weekday = current_datetime.weekday();
        // shift the current time because the scheule legs use a NaiveTime
        let time = current_datetime.with_timezone(&Local).time();
        self.legs.iter().find(|leg| {
            leg.weekdays.iter().find(|w| **w == weekday) != None && leg.active_range.start <= time && leg.active_range.end >= time 
        })
    }
}

impl Config {
    pub fn new(max_temp: T<F>, min_temp: T<F>) -> Config  {
        Config {
            max_temp: max_temp,
            min_temp: min_temp,
            hold_end: None,
            fan_end: None,
            schedule: Schedule::new(vec![]),
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

    /// Returns a tuple of (minRange, maxRange) specifying the allowable ranges of temperatures
    /// before turning on AC, Heat, ETC
    pub fn get_temp_ranges(&self, time: DateTime<UTC>) -> (Range<T<F>>, Range<T<F>>) {
        let temp_range = T::in_f(HOLD_RANGE.start)..T::in_f(HOLD_RANGE.end);
        let (min_temp, max_temp) = match self.schedule.get_active_leg(time) {
            Some(active_leg) => {
                info!("Schedule active!");
                (active_leg.min_temp, active_leg.max_temp)
            },
            None => (self.min_temp, self.max_temp),
        };

        (
            (min_temp - temp_range.end)..(min_temp - temp_range.start),
            (max_temp + temp_range.start)..(max_temp + temp_range.end)
        )
    }

    pub fn set_schedule(&mut self, schedule: Schedule) {
        self.schedule = schedule;
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

