extern crate thermostat;
extern crate env_logger;
extern crate chrono;

use std::env;
use std::thread;
use chrono::*;

use thermostat::uom::temp::*;
use thermostat::controller::*;
use thermostat::ac_control::compressor::*;
use thermostat::sensors::*;
use thermostat::platform::*;
use thermostat::controller::config::Config;

static USAGE: &'static str = "Usage: thermostat max_temp_f [min_temp_f] [sleep_duration_s]";

fn main() {
    // initialize logging framework
    env_logger::init().unwrap();

    let hold_temp_string = env::args().nth(1).expect(USAGE);
                               ;
    let hold_temp = Temperature::in_f(hold_temp_string
                                      .parse::<f32>().expect("Invalid hold temperature"));

    let min_temp = Temperature::in_f(env::args().nth(2).unwrap_or(hold_temp_string)
                                      .parse::<f32>().expect("Invalid hold temperature"));

    let sleep_duration_s = env::args().nth(3).unwrap_or("120".to_string())
                                      .parse::<u64>().expect("Invalid sleep duration");

    let mut compressor = Compressor::new();

    let config = Config::new(hold_temp, min_temp);

    let mut controller = Controller::new(&mut compressor, config);

    // TODO: this is nasty because of the mutable borrow.  Need to rethink the observer idea
    let mut temp_sensor = TempSensor::<linux::McuTemp>::new();

    loop {
        if let Some(temp) = temp_sensor.get_updated_temp() {
            println!("Temp changed {}", temp);
            controller.on_temp_updated(temp);
        }

        controller.time_changed(UTC::now());

        thread::sleep(std::time::Duration::from_secs(sleep_duration_s));
    }
}

