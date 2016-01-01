use ::uom::temp::*;
use ::sensors::*;
use ::ac_control::compressor::*;

pub struct Controller {
    hold_temp: Temperature<F>,
    min_temp: Temperature<F>,
}

pub enum Status {
    TooHot,
    TooCold,
    JustRight
}

impl Controller {
    pub fn new(hold_temp: Temperature<F>, min_temp: Temperature<F>) -> Controller { 
        Controller { hold_temp: hold_temp, min_temp: min_temp } 
    }

    pub fn control<T: TempSensor>(&self, temp_sensor: &T, compressor: &mut Compressor) {
        use ::controller::Status::*;
        use ::ac_control::compressor::CompressorMode::*;
        let temp = temp_sensor.get_temperature();

        match self.check_status(temp) {
            TooHot => compressor.set_mode(Cool),
            TooCold => compressor.set_mode(HeatPump),
            JustRight => compressor.set_mode(Off),
        }
    }

    pub fn check_status(&self, temp: Temperature<F>) -> Status {
        use ::controller::Status::*;
        if temp > self.hold_temp {
            TooHot
        } else if temp < self.min_temp {
            TooCold
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
        let controller = Controller::new(Temperature::in_f(77.0), Temperature::in_f(74.0));
        let mock = MockSensor { temp: Temperature::in_f(79.0) };
        let mut compressor = Compressor::new();

        controller.control(&mock, &mut compressor);

        assert_eq!(CompressorMode::Cool, compressor.get_mode());
    }

    #[test]
    fn it_turns_on_the_heat_pump_if_the_temperature_is_below_the_target() {
        let controller = Controller::new(Temperature::in_f(77.0), Temperature::in_f(74.0));
        let mock = MockSensor { temp: Temperature::in_f(73.0) };
        let mut compressor = Compressor::new();

        controller.control(&mock, &mut compressor);

        assert_eq!(CompressorMode::HeatPump, compressor.get_mode());
    }

    #[test]
    fn it_turns_off_the_ac_if_the_temperature_is_within_the_range() {
        let controller = Controller::new(Temperature::in_f(77.0), Temperature::in_f(74.0));
        let mock = MockSensor { temp: Temperature::in_f(74.0) };
        let mut compressor = Compressor::new();
        compressor.set_mode(CompressorMode::Cool);

        controller.control(&mock, &mut compressor);

        assert_eq!(CompressorMode::Off, compressor.get_mode());
    }
}
