mod temperature_humidity;

use ::uom::temp::*;

use std::marker::PhantomData;

pub trait TempListener {
    fn on_temp_updated(&mut self, temp: Temperature<F>);
}

pub struct TempSensor< R> {
    last_temp: Option<Temperature<F>>,
    reader: PhantomData<R>,
}

pub trait TempReader {
    fn get_temp() -> Temperature<F>;
}

impl<R> TempSensor<R> where R : TempReader {
    pub fn new() -> TempSensor<R> {
        TempSensor {
            last_temp: None,
            reader: PhantomData
        }
    }

    pub fn get_updated_temp(&mut self) -> Option<Temperature<F>> {
        let temp = R::get_temp();
        let changed = match self.last_temp {
            Some(last_temp) => last_temp != temp,
            None => true,
        };

        self.last_temp = Some(temp);
        
        if changed { Some(temp) } else { None }
    }
}

#[cfg(test)]
mod test {
    use ::uom::temp::*;
    use super::*;

    static mut current_temp: f32 = 77.0;

    struct Mock;
    impl TempReader for Mock {
        fn get_temp() -> Temperature<F> {
            unsafe { Temperature::in_f(current_temp) }
        }
    }

    #[test]
    fn updates_listeners_when_the_first_reading_happens() {
        let mut sensor = TempSensor::<Mock>::new();

        assert!(sensor.get_updated_temp() == Some(Temperature::in_f(77.0)));
    }

    #[test]
    fn doesnt_update_listeners_if_the_temperature_hasnt_changed() {
        let mut sensor = TempSensor::<Mock>::new();

        assert!(sensor.get_updated_temp() == Some(Temperature::in_f(77.0)));
        assert!(sensor.get_updated_temp() == None);
    }

    #[test]
    fn updates_listeners_when_the_temperature_changes() {
        let mut sensor = TempSensor::<Mock>::new();

        assert!(sensor.get_updated_temp() == Some(Temperature::in_f(77.0)));

        unsafe { current_temp = 74.9; }

        assert!(sensor.get_updated_temp() == Some(Temperature::in_f(74.9)));
    }
}
