pub mod config;

use ::uom::temp::*;
use ::ac_control::compressor::*;
use ::sensors::TempListener;
use self::config::Config;
use chrono::*;

pub struct Controller<'a> {
    config: Config,
    compressor: &'a mut Compressor,
}

#[derive(PartialEq, Debug)]
pub enum Status {
    TooHot,
    TooCold,
    JustRight,
    Hold
}


impl<'a> Controller<'a> {
    pub fn new(compressor: &'a mut Compressor, config: Config) -> Controller { 
        Controller { 
            config: config,
            compressor: compressor
        } 
    }

    pub fn update_config(&mut self, config: Config) {
        self.config = config;
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

    pub fn time_changed(&mut self, time: DateTime<UTC>) {
        self.compressor.set_fan_mode(self.config.is_fan_on(time));
    }

    pub fn check_status(&self, temp: Temperature<F>) -> Status {
        use ::controller::Status::*;
        
        let (min_range, max_range) = self.config.get_temp_ranges();

        if temp > max_range.end {
            TooHot
        } else if temp >= max_range.start {
            Hold
        } else if temp < min_range.start {
            TooCold
        } else if temp <= min_range.end {
            Hold
        } else {
            JustRight
        }
    }
}

impl<'a> TempListener for Controller<'a> {
    fn on_temp_updated(&mut self, temp: Temperature<F>) {
        self.temp_changed(temp);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::config::Config;
    use ::uom::temp::*;
    use ::ac_control::compressor::*;
    use ::uom::temp::Temperature as T;
    use chrono::*;

    #[test]
    fn it_turns_on_the_compressor_if_the_temperature_is_above_the_target() {
        let mut compressor = Compressor::new();
        {
            let config = Config::new(Temperature::in_f(77.0), Temperature::in_f(74.0));
            let mut controller = Controller::new(&mut compressor, config);

            controller.temp_changed(T::in_f(79.0));
        }

        assert_eq!(CompressorMode::Cool, compressor.get_mode());
    }

    #[test]
    fn it_turns_on_the_heat_pump_if_the_temperature_is_below_the_target() {
        let mut compressor = Compressor::new();
        {
            let config = Config::new(Temperature::in_f(77.0), Temperature::in_f(74.0));
            let mut controller = Controller::new(&mut compressor, config);

            controller.temp_changed(T::in_f(73.0));
        }

        assert_eq!(CompressorMode::HeatPump, compressor.get_mode());
    }

    #[test]
    fn it_turns_off_the_ac_if_the_temperature_is_within_the_range() {
        let mut compressor = Compressor::new();
        compressor.set_min_change_duration(Duration::zero());
        compressor.set_mode(CompressorMode::Cool);
        {
            let config = Config::new(Temperature::in_f(77.0), Temperature::in_f(74.0));
            let mut controller = Controller::new(&mut compressor, config);

            controller.temp_changed(T::in_f(76.0));
        }

        assert_eq!(CompressorMode::Off, compressor.get_mode());
    }

    #[test]
    fn it_doesnt_change_the_ac_or_heat_if_temperature_is_in_a_hold_range() {
        let mut compressor = Compressor::new();
        compressor.set_mode(CompressorMode::Cool);
        {
            let config = Config::new(Temperature::in_f(77.0), Temperature::in_f(74.0));
            let mut controller = Controller::new(&mut compressor, config);

            controller.temp_changed(T::in_f(76.5));
        }

        assert_eq!(CompressorMode::Cool, compressor.get_mode());

        let mut compressor = Compressor::new();
        compressor.set_mode(CompressorMode::Off);

        {
            let config = Config::new(Temperature::in_f(77.0), Temperature::in_f(74.0));
            let mut controller = Controller::new(&mut compressor, config);

            controller.temp_changed(T::in_f(76.5));
        }

        assert_eq!(CompressorMode::Off, compressor.get_mode());
    }

    #[test]
    fn it_only_triggers_the_ac_when_1_deg_over_the_hold_temp() {
        let mut compressor = &mut Compressor::new();
        let config = Config::new(Temperature::in_f(77.0), Temperature::in_f(74.0));
        let controller = Controller::new(&mut compressor, config);

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
        let config = Config::new(Temperature::in_f(77.0), Temperature::in_f(74.0));
        let controller = Controller::new(&mut compressor, config);

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
