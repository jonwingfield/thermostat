use ::uom::temp::*;
use ::sensors::TempReader;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;

pub struct McuTemp;

// TODO: move these functions into a hardware-specific module
fn parse_temp(buffer: String) -> Option<Temperature<F>> {
    if let Some(index) = buffer.find(", ") {
        let (temp, _) = buffer.split_at(index);

        if let Some(index) = temp.find(' ') {
            let (_, temp) = temp.split_at(index+1);

            let tempf = Temperature::in_c(temp.parse::<f32>().unwrap()/10.0) .to_f();
            return Some(tempf);
        }
    }

    None
}

fn open_sensor() -> File {
    OpenOptions::new().read(true).write(true).open(Path::new("/dev/ttymcu0")).unwrap()
}

fn read_sensor(mut mcu: File) -> String {
    mcu.write(b"get_temp\n").unwrap();

    let mut reader = BufReader::new(mcu);
    let mut buffer = String::new();
    reader.read_line(&mut buffer).unwrap();

    buffer
}

impl TempReader for McuTemp {
    fn get_temp() -> Temperature<F> {
        // TODO: eliminate these unwraps
        parse_temp(read_sensor(open_sensor())).unwrap()
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use ::uom::temp::*;

#[test]
fn parse_valid_temp() {
    let temp_str = "Temp 229, Humidity 345";

    assert!(parse_temp(temp_str.to_string()) == Some(Temperature::in_c(22.9).to_f()));
}

#[test]
fn parse_invalid_temp() {
    let temp_str = "Temp229, Humidity 345";

    assert!(parse_temp(temp_str.to_string()) == None);
}
// }
