pub mod compressor {
    use chrono::*;

    #[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
    pub enum CompressorMode {
        Cool,
        HeatPump,
        Off,
    }

    pub struct Compressor {
        mode: CompressorMode,
        fan_mode: bool,
        min_duration: Duration,
        next_allowed_compressor_change: DateTime<UTC>,
        next_allowed_fan_change: DateTime<UTC>,
    }

    impl Compressor {
        pub fn new() -> Compressor { 
            let now = UTC::now();
            // TODO: possibly use lazy_static crate here
            let min_duration = Duration::minutes(2);
            Compressor { 
                mode: CompressorMode::Off,
                fan_mode: false,
                next_allowed_compressor_change: now - min_duration,
                next_allowed_fan_change: now - min_duration,
                min_duration: min_duration
            }
        }

        #[cfg(test)]
        pub fn set_min_change_duration(&mut self, min_duration: Duration) {
            self.min_duration = min_duration;
        }

        pub fn set_mode(&mut self, mode: CompressorMode) {
            if mode == self.mode { return; }

            let now = UTC::now();
            if self.next_allowed_compressor_change < now {
                self.mode = mode;
                self.next_allowed_compressor_change = now + self.min_duration;
            } else {
                warn!("Compressor toggled too fast. {} {}", now, self.next_allowed_compressor_change);
            }
        }

        pub fn get_mode(&self) -> CompressorMode {
            self.mode
        }

        pub fn set_fan_mode(&mut self, mode: bool) {
            if mode == self.fan_mode { return; }

            let now = UTC::now();
            if self.next_allowed_fan_change < now {
                self.fan_mode = mode;
                self.next_allowed_fan_change = now + self.min_duration;
            } else {
                warn!("Fan toggled too fast. {} {}", now, self.next_allowed_fan_change);
            }
        }

        pub fn get_fan_mode(&mut self) -> bool {
            self.fan_mode
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn limits_compressor_changes_to_2_minutes_minimum() {
            let mut compressor = Compressor::new();

            compressor.set_mode(CompressorMode::Cool);
            compressor.set_mode(CompressorMode::Off);
            compressor.set_fan_mode(true);

            assert_eq!(compressor.get_mode(), CompressorMode::Cool);
            assert_eq!(compressor.get_fan_mode(), true);
        }

        #[test]
        fn limits_fan_changes_to_2_minutes_minimum() {
            let mut compressor = Compressor::new();

            compressor.set_fan_mode(true);
            compressor.set_fan_mode(false);
            compressor.set_mode(CompressorMode::Cool);

            assert_eq!(compressor.get_mode(), CompressorMode::Cool);
            assert_eq!(compressor.get_fan_mode(), true);
        }

    }
}

