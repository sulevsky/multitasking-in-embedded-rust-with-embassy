use chrono::NaiveDateTime;

use crate::clock_serde::{deserialize, serialize};

const RTC_ADDR: u8 = 0x68;

pub fn read<T: embedded_hal::i2c::I2c>(i2c: &mut T) -> NaiveDateTime {
    let mut buf = [0u8; 7];
    i2c.write_read(RTC_ADDR, &[0x00], &mut buf).unwrap();
    deserialize(&buf).unwrap()
}

pub fn write<T: embedded_hal::i2c::I2c>(i2c: &mut T, datetime: &NaiveDateTime) {
    let mut buf = [0u8; 8];
    buf[0] = 0x00;
    serialize(datetime, (&mut buf[1..]).try_into().unwrap());

    i2c.write(RTC_ADDR, &buf).unwrap();
}
