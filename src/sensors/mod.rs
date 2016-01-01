mod temperature_humidity;

use ::uom::temp::*;

pub trait TempSensor {
    fn get_temperature(&self) -> Temperature<F>;
}
