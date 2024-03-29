extern crate derive_builder;

pub use tlms::telegrams::r09::{ R09Telegram, R09Type };

#[derive(Debug)]
pub struct BCD(pub u32);


impl BCD {
    pub fn parse(bytes: &[u8]) -> Option<BCD> {
        let mut number: u32 = 0;

        for val in bytes {
            if *val > 9 {
                return None;
            }

            number = number * 10 + *val as u32;
        }

        Some(BCD(number))
    }
}

pub fn parse_r09_telegram(byte_array: &[u8]) -> Option<R09Telegram> {

    let telegram_type = match byte_array[1] & 0xf {
        4 => R09Type::R14,
        6 => R09Type::R16,
        _ => return None,
    };

    let reporting_point: u32 = (((byte_array[2] >> 4) as u32) << 12u8) as u32
        | (((byte_array[2] & 0x0f) as u32) << 8) as u32
        | ((byte_array[3] >> 4) << 4) as u32
        | (byte_array[3] & 0x0f) as u32; //MP Melde Punkt

    let line = match BCD::parse(&[byte_array[4] & 0xf, byte_array[5] >> 4, byte_array[5] & 0xf]) {
        Some(x) => x.0,
        None => return None,
    };
    let run = match BCD::parse(&[byte_array[6] >> 4, byte_array[6] & 0xf]) {
        Some(x) => x.0,
        None => return None,
    };
    let destination =
        match telegram_type {
            R09Type::R16 => match BCD::parse(&[byte_array[7] >> 4, byte_array[7] & 0xf, byte_array[8] >> 4]) {
                    Some(x) => Some(x.0),
                    None => return None,
                },
            _ => None,
        };
    let train_length = match telegram_type {
        R09Type::R16 => Some(byte_array[8] & 0x7),
        _ => None,
    };

    //TODO: marenz for all variants

    let sign_of_deviation = (byte_array[1] >> 7) as i32; //ZV Zeit Vorzeichen
    let value_of_deviation = ((byte_array[1] >> 4) & 0x7) as i32; //ZW Zahlen Wert

    Some(R09Telegram {
        r09_type: telegram_type,
        delay: Some((sign_of_deviation * -2 + 1) * value_of_deviation),
        reporting_point, //MP Melde punkt
        junction: (reporting_point >> 2) / 10,
        direction: ((reporting_point >> 2) % 10) as u8,
        request_status: (reporting_point & 0x3) as u8,
        priority: Some((byte_array[4] >> 6) as u8), //PR Prioritet
        direction_request: Some(((byte_array[4] >> 4) & 0x3) as u8), // HA Anforderung Richtung
        line: Some(line),                        // LN Line Nummer
        run_number: Some(run),                   // KN Kurse Nummer
        destination_number: destination,         // ZN Zielnummer
        train_length: train_length.map(|v| v.into()),              // ZL length
        vehicle_number: None,
        operator: None,
    })
}
