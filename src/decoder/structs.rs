extern crate derive_builder;

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize, Serialize, Debug)]
pub struct Telegram {
    time_stamp: u64,
    lat: f64,
    lon: f64,
    station_id: u32,
    line: u32,
    destination_number: u32,
    priority: u32,
    sign_of_deviation: u32,
    value_of_deviation: u32,
    reporting_point: u32,
    request_for_priority: u32,
    run_number: u32,
    reserve: u32,
    train_length: u32,
    junction: u32,
    junction_number: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    name: String,
    lat: f64,
    lon: f64,
    station_id: u32
}

impl Telegram {
    pub fn parse(byte_array: &[u8], station_config: &Config) -> Telegram {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let reporting_point: u32 = ((((byte_array[2] >> 4) as u32) << 12u8) as u8
            | (((byte_array[2] & 0x0f) as u32) << 8) as u8
            | ((byte_array[3] >> 4) << 4)
            | (byte_array[3] & 0x0f)) as u32; //MP Melde Punkt

        Telegram {
            time_stamp: since_the_epoch.as_secs(),
            lat: station_config.lat, //51.027107,
            lon: station_config.lon,//13.723566,
            station_id: station_config.station_id,
            sign_of_deviation: (byte_array[1] >> 7) as u32, //ZV Zeit Vorzeichen
            value_of_deviation: ((byte_array[1] >> 4) & 0x7) as u32, //ZW Zahlen Wert
            reporting_point: reporting_point as u32,        //MP Melde punkt
            priority: (byte_array[4] >> 6) as u32,          //PR Prioritet
            request_for_priority: ((byte_array[4] >> 4) & 0x3) as u32, // HA Anforderung Richtung
            line: (100 * (byte_array[4] & 0xf) + 10 * (byte_array[5] >> 4) + (byte_array[5] & 0xf))
                as u32, // LN Line Nummer
            run_number: (10 * (byte_array[6] >> 4) + (byte_array[6] & 0xf)) as u32, // KN Kurse Nummer
            destination_number: (100 * ((byte_array[7] as u32) >> 4)
                + 10 * ((byte_array[7] as u32) & 0xf)
                + ((byte_array[8] as u32) >> 4)) as u32, // ZN Zielnummer
            reserve: ((byte_array[8] >> 3) & 0x1) as u32,                           // R reserve
            train_length: (byte_array[8] & 0x7) as u32,                             // ZL length
            junction: (reporting_point >> 2) / 10 as u32,                                // 10
            junction_number: ((reporting_point >> 2) - 10 * ((reporting_point >> 2) / 10)) as u32,
        }
    }
}
