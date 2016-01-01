use std::env;
use std::thread;
use std::time::Duration;

extern crate thermostat;

use thermostat::uom::temp::*;
use thermostat::controller::*;
use thermostat::ac_control::compressor::*;
use thermostat::sensors::*;

static USAGE: &'static str = "Usage: thermostat hold_temp_f [sleep_duration_s]";
static mut TEMP: u16 = 77;

struct Sensor;

impl TempSensor for Sensor {
    fn get_temperature(&self) -> Temperature<F> {
        unsafe {
            TEMP+=1;
            Temperature::in_f(TEMP as f32)
        }
    }
}

fn main() {
    let hold_temp = Temperature::in_c(env::args().nth(1).expect(USAGE)
                               .parse::<f32>().expect("Invalid hold temperature"));

    let sleep_duration_s = env::args().nth(2).unwrap_or("120".to_string())
                                      .parse::<u64>().expect("Invalid sleep duration");

    let min_temp = hold_temp.to_f() - Temperature::in_f(5.0);

    println!("Temp: {}", hold_temp.to_f());

    let controller = Controller::new(hold_temp.to_f(), min_temp);
    let sensor = Sensor;
    let mut compressor = Compressor::new();

    loop {
        controller.control(&sensor, &mut compressor);

        thread::sleep(Duration::from_secs(sleep_duration_s));
    }
}

