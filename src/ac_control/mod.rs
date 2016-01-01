pub mod compressor {

    #[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
    pub enum CompressorMode {
        Cool,
        HeatPump,
        Off,
    }
    pub struct Compressor {
        mode: CompressorMode
    }

    impl Compressor {
        pub fn new() -> Compressor { Compressor { mode: CompressorMode::Off } }

        pub fn set_mode(&mut self, mode: CompressorMode) {
            self.mode = mode;
        }

        pub fn get_mode(&mut self) -> CompressorMode {
            self.mode
        }
    }
}

