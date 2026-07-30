use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};

pub fn deserialize(binary: &[u8; 7]) -> Option<NaiveDateTime> {
    NaiveDate::from_ymd_opt(
        (from_bcd(binary[6]) as i32) + 2000,
        from_bcd(binary[5]) as u32,
        from_bcd(binary[4]) as u32,
    )?
    .and_hms_opt(
        from_bcd(binary[2] & 0x3F) as u32,
        from_bcd(binary[1]) as u32,
        from_bcd(binary[0] & 0x7F) as u32,
    )
}

pub fn serialize(datetime: &NaiveDateTime, buffer: &mut [u8; 7]) {
    buffer[0] = to_bcd(datetime.second() as u8) | 0x80;
    buffer[1] = to_bcd(datetime.minute() as u8);
    buffer[2] = to_bcd(datetime.hour() as u8) & 0x3F;
    buffer[3] = 0;
    buffer[4] = to_bcd(datetime.day() as u8);
    buffer[5] = to_bcd(datetime.month() as u8);
    buffer[6] = to_bcd((datetime.year() - 2000) as u8);
}

fn from_bcd(value: u8) -> u8 {
    (value >> 4) * 10 + (value & 0x0F)
}

fn to_bcd(value: u8) -> u8 {
    ((value / 10) << 4) | (value % 10)
}
