use ::uom::temp::Temperature as T;
use ::uom::temp::*;
use ::sensors::*;
use ::ac_control::compressor::*;
use std::ops::Range;
use chrono::*;

pub struct Controller<'a> {
    hold_range: Range<Temperature<F>>,
    min_range: Range<Temperature<F>>,
    compressor: &'a mut Compressor,
}

pub struct Config {
    pub max_temp: Temperature<F>,
    pub min_temp: Temperature<F>,
    pub hold_end: Option<DateTime<UTC>>,
    pub fan_end: Option<DateTime<UTC>>,
}

impl Config {
    pub fn use_hold_mode(&mut self, timeout: Duration) {
        self.hold_end = Some(UTC::now() + timeout);
    }

    pub fn turn_fan_on(&mut self, timeout: Duration) {
        self.fan_end = Some(UTC::now() + timeout);
    }
}

#[derive(PartialEq, Debug)]
pub enum Status {
    TooHot,
    TooCold,
    JustRight,
    Hold
}

const HOLD_RANGE: Range<f32> = Range { start: -0.5, end: 0.9 };

impl<'a> Controller<'a> {
    pub fn new(compressor: &'a mut Compressor, hold_temp: Temperature<F>, min_temp: Temperature<F>) -> Controller { 
        let temp_range = T::in_f(HOLD_RANGE.start)..T::in_f(HOLD_RANGE.end);

        Controller { 
            hold_range: (hold_temp + temp_range.start)..(hold_temp + temp_range.end),
            min_range: (min_temp - temp_range.end)..(min_temp - temp_range.start),
            compressor: compressor
        } 
    }

    pub fn control<T: TempSensor>(&mut self, temp_sensor: &T) {
        let temp = temp_sensor.get_temperature();
        self.temp_changed(temp);
    }

    pub fn temp_changed(&mut self, temp: Temperature<F>) {
        use ::controller::Status::*;
        use ::ac_control::compressor::CompressorMode::*;

        let status = self.check_status(temp);
        match status {
            TooHot => self.compressor.set_mode(Cool),
            TooCold => self.compressor.set_mode(HeatPump),
            JustRight => self.compressor.set_mode(Off),
            Hold => (),
        }
        info!("Status: {:?}", status);
    }

    pub fn check_status(&self, temp: Temperature<F>) -> Status {
        use ::controller::Status::*;
        
        info!("{}-{}", self.min_range.start, self.min_range.end);

        if temp > self.hold_range.end {
            TooHot
        } else if temp >= self.hold_range.start {
            Hold
        } else if temp < self.min_range.start {
            TooCold
        } else if temp <= self.min_range.end {
            Hold
        } else {
            JustRight
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ::uom::temp::*;
    use ::sensors::*;
    use ::ac_control::compressor::*;

    struct MockSensor {
        temp: Temperature<F>,
    }

    impl TempSensor for MockSensor {
        fn get_temperature(&self) -> Temperature<F> { self.temp }
    }

    #[test]
    fn it_turns_on_the_compressor_if_the_temperature_is_above_the_target() {
        let mut compressor = Compressor::new();
        {
            let mut controller = Controller::new(&mut compressor, Temperature::in_f(77.0), Temperature::in_f(74.0));
            let mock = MockSensor { temp: Temperature::in_f(79.0) };

            controller.control(&mock);
        }

        assert_eq!(CompressorMode::Cool, compressor.get_mode());
    }

    #[test]
    fn it_turns_on_the_heat_pump_if_the_temperature_is_below_the_target() {
        let mut compressor = Compressor::new();
        {
            let mut controller = Controller::new(&mut compressor, Temperature::in_f(77.0), Temperature::in_f(74.0));
            let mock = MockSensor { temp: Temperature::in_f(73.0) };

            controller.control(&mock);
        }

        assert_eq!(CompressorMode::HeatPump, compressor.get_mode());
    }

    #[test]
    fn it_turns_off_the_ac_if_the_temperature_is_within_the_range() {
        let mut compressor = Compressor::new();
        compressor.set_mode(CompressorMode::Cool);
        {
            let mut controller = Controller::new(&mut compressor, Temperature::in_f(77.0), Temperature::in_f(74.0));
            let mock = MockSensor { temp: Temperature::in_f(76.0) };

            controller.control(&mock);
        }

        assert_eq!(CompressorMode::Off, compressor.get_mode());
    }

    #[test]
    fn it_doesnt_change_the_ac_or_heat_if_temperature_is_in_a_hold_range() {
        let mut compressor = Compressor::new();
        let mock = MockSensor { temp: Temperature::in_f(76.5) };
        compressor.set_mode(CompressorMode::Cool);
        {
            let mut controller = Controller::new(&mut compressor, Temperature::in_f(77.0), Temperature::in_f(74.0));

            controller.control(&mock);
        }

        assert_eq!(CompressorMode::Cool, compressor.get_mode());

        compressor.set_mode(CompressorMode::Off);

        {
            let mut controller = Controller::new(&mut compressor, Temperature::in_f(77.0), Temperature::in_f(74.0));

            controller.control(&mock);
        }

        assert_eq!(CompressorMode::Off, compressor.get_mode());
    }

    #[test]
    fn it_only_triggers_the_ac_when_1_deg_over_the_hold_temp() {
        let mut compressor = &mut Compressor::new();
        let controller = Controller::new(&mut compressor, Temperature::in_f(77.0), Temperature::in_f(74.0));

        assert_eq!(Status::JustRight,
                   controller.check_status(Temperature::in_f(76.4)));
        assert_eq!(Status::Hold,
                   controller.check_status(Temperature::in_f(76.5)));
        assert_eq!(Status::Hold,
                   controller.check_status(Temperature::in_f(77.9)));
        assert_eq!(Status::TooHot,
                   controller.check_status(Temperature::in_f(78.0)));
    }

    #[test]
    fn it_only_triggers_the_heat_when_1_deg_under_the_hold_temp() {
        let mut compressor = &mut Compressor::new();
        let controller = Controller::new(&mut compressor, Temperature::in_f(77.0), Temperature::in_f(74.0));

        assert_eq!(Status::JustRight,
                   controller.check_status(Temperature::in_f(74.6)));
        assert_eq!(Status::Hold,
                   controller.check_status(Temperature::in_f(74.5)));
        assert_eq!(Status::Hold,
                   controller.check_status(Temperature::in_f(73.1)));
        assert_eq!(Status::TooCold,
                   controller.check_status(Temperature::in_f(73.0)));
    }
}
