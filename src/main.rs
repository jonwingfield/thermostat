extern crate thermostat;
extern crate env_logger;
extern crate chrono;
extern crate thermostat_server;
extern crate num;
extern crate rustc_serialize;

use std::env;
use std::thread;
use std::error::Error;
use std::fs::{OpenOptions, File};
use std::io::prelude::*;
use std::io;
use chrono::*;

use std::sync::mpsc::{channel, TryRecvError, Receiver};
use std::sync::{RwLock, Arc};

use num::traits::FromPrimitive;

use thermostat::uom::temp::*;
use thermostat::controller::*;
use thermostat::ac_control::compressor::*;
use thermostat::sensors::*;
use thermostat::platform::*;
use thermostat::controller::config::Config;
use thermostat::controller::config::Schedule;
use thermostat::controller::config::ScheduleLeg;
use thermostat_server::server::Status as StatusDto;
use thermostat_server::server::Config as ConfigDto;
use thermostat_server::server::Schedule as ScheduleDto;

use rustc_serialize::json;


static USAGE: &'static str = "Usage: thermostat max_temp_f [min_temp_f] [sleep_duration_s]";

fn main() {
    // initialize logging framework
    env_logger::init().unwrap();

    let mut switches = linux::ac_control::GpioSwitches::new();
    let mut compressor = Compressor::new(&mut switches);

    let (mut config, sleep_duration_s) = parse_args(); 

    let config_dto = if let Ok(config_dto) = load_config() {
        println!("Updating config");
        update_config(&mut config, &config_dto);
        config_dto
    } else {
        ConfigDto { maxTempF: 79, minTempF: 70, fanDurationHours: 0, schedule: vec!() }
    };

    let (status_lock, rx) = start_server(&config_dto);

    let mut temp_sensor = TempSensor::<linux::McuTemp>::new();

    let temp = temp_sensor.get_updated_temp().expect("Cannot continue without an intitial temperature");
    let mut controller = Controller::new(&mut compressor, config.clone(), temp);
    update_temp(temp, &status_lock);

    loop {
        match rx.try_recv() {
            Ok(config_dto) => {
                println!("Config updated: {}", config_dto);
                update_config(&mut config, &config_dto);
                controller.update_config(config.clone());
                save_config(&config_dto);
            },
            Err(err) if err == TryRecvError::Disconnected => {
                panic!("Web server disconnected!");
            },
            _ => (),
        }

        if let Some(temp) = temp_sensor.get_updated_temp() {
            println!("Temp changed {}", temp);
            controller.on_temp_updated(temp);
            update_temp(temp, &status_lock);
        }

        controller.time_changed(UTC::now());

        thread::sleep(std::time::Duration::from_secs(sleep_duration_s));
    }
}

fn save_config(config: &ConfigDto) {
    let file_opened = OpenOptions::new()
        .write(true)
        .create(true)
        .append(false)
        .open("config.json");

    let result = file_opened.map(|mut file| {
        let json_string = json::encode(config).unwrap();
        file.write_all(json_string.as_bytes())
    });
    
    if let Err(e) = result {
        println!("Could not write to file {}", e.description());
    }
}

fn load_config() -> io::Result<ConfigDto> {
    let mut file = try!(File::open("config.json"));
    let mut s = String::new();
    try!(file.read_to_string(&mut s));
    Ok(json::decode(&s).unwrap())
}

fn update_config(config: &mut Config, config_dto: &ConfigDto) {
    config.max_temp = Temperature::in_f(config_dto.maxTempF as f32);
    config.min_temp = Temperature::in_f(config_dto.minTempF as f32);
    config.set_fan_on(Duration::hours(config_dto.fanDurationHours as i64));
    config.set_schedule(map_schedule(&config_dto.schedule));
}

fn start_server(config: &ConfigDto) -> (Arc<RwLock<StatusDto>>, Receiver<ConfigDto>) {
    let (tx, rx) = channel();
    let status = StatusDto { currentTempF: 0.0, compressorOn: false, fanOn: false };
    let status_lock = Arc::new(RwLock::new(status));
    let status_return = status_lock.clone();
    
    let config_for_server = config.clone();

    thread::spawn(move || { thermostat_server::server::start(config_for_server, tx, status_lock); });
    println!("started!");

    (status_return, rx)
}

fn update_temp(temp: Temperature<F>, status_lock: &Arc<RwLock<StatusDto>>) {
    let mut status = status_lock.write().unwrap();
    status.currentTempF = temp.value();
}

fn map_schedule(schedules: &Vec<ScheduleDto>) -> Schedule {
    // TODO: all of this parsing should take place in the server, and return 401 Bad Request if it
    // doesn't parse
    Schedule::new(schedules.iter().filter_map(|schedule| {
        let start_result = NaiveTime::parse_from_str(&schedule.start, "%I:%M %p");
        let end_result = NaiveTime::parse_from_str(&schedule.end, "%I:%M %p");

        if let (Ok(start), Ok(end)) = (start_result, end_result) {
            Some(ScheduleLeg {
                min_temp: Temperature::in_f(schedule.minTempF as f32),
                max_temp: Temperature::in_f(schedule.maxTempF as f32),
                active_range: start..end, 
                weekdays: schedule.days.iter().filter_map(|d| Weekday::from_i8(*d)).collect()
            })
        } else {
            println!("error parsing {} {}", schedule.start, schedule.end);
            None
        }

    }).collect())
}

fn parse_args() -> (Config, u64) {
    let hold_temp_string = env::args().nth(1).expect(USAGE);
                               ;
    let hold_temp = Temperature::in_f(hold_temp_string
                                      .parse::<f32>().expect("Invalid hold temperature"));

    let min_temp = Temperature::in_f(env::args().nth(2).unwrap_or(hold_temp_string)
                                      .parse::<f32>().expect("Invalid hold temperature"));

    let sleep_duration_s = env::args().nth(3).unwrap_or("120".to_string())
                                      .parse::<u64>().expect("Invalid sleep duration");

    (Config::new(hold_temp, min_temp), sleep_duration_s)
}
