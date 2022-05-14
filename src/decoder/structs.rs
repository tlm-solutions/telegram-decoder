extern crate derive_builder;

use hex_string::nibble_to_hexchar;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct BCD(pub u32);

#[derive(Deserialize, Serialize, Debug)]
pub struct Telegram {
    pub time_stamp: u64,
    pub line: String,
    pub destination_number: String,
    pub priority: u32,
    pub sign_of_deviation: u32,
    pub value_of_deviation: u32,
    pub reporting_point: u32,
    pub request_for_priority: u32,
    pub run_number: String,
    pub reserve: u32,
    pub train_length: u32,
    pub junction: u32,
    pub junction_number: u32,
    pub request_status: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub station_id: u32,
}

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

impl Telegram {
    pub fn parse(byte_array: &[u8]) -> Option<Telegram> {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let reporting_point: u32 = (((byte_array[2] >> 4) as u32) << 12u8) as u32
            | (((byte_array[2] & 0x0f) as u32) << 8) as u32
            | ((byte_array[3] >> 4) << 4) as u32
            | (byte_array[3] & 0x0f) as u32; //MP Melde Punkt

        let line = match BCD::parse(&[byte_array[4] & 0xf, byte_array[5] >> 4, byte_array[5] & 0xf])
        {
            Some(x) => x.0,
            None => return None,
        };
        let run_number = match BCD::parse(&[byte_array[6] >> 4, byte_array[6] & 0xf]) {
            Some(x) => x.0,
            None => return None,
        };
        let destination_number =
            match BCD::parse(&[byte_array[7] >> 4, byte_array[7] & 0xf, byte_array[8] >> 4]) {
                Some(x) => x.0,
                None => return None,
            };
        let reserve = match ((byte_array[8] >> 3) & 0x1) as u32 {
            0 => 0,
            _ => return None,
        };

        Some(Telegram {
            time_stamp: since_the_epoch.as_secs(),
            sign_of_deviation: (byte_array[1] >> 7) as u32, //ZV Zeit Vorzeichen
            value_of_deviation: ((byte_array[1] >> 4) & 0x7) as u32, //ZW Zahlen Wert
            reporting_point: reporting_point,               //MP Melde punkt
            priority: (byte_array[4] >> 6) as u32,          //PR Prioritet
            request_for_priority: ((byte_array[4] >> 4) & 0x3) as u32, // HA Anforderung Richtung
            line: format!("{:03}", line),                   // LN Line Nummer
            run_number: format!("{:02}", run_number),       // KN Kurse Nummer
            destination_number: format!("{:03}", destination_number), // ZN Zielnummer
            reserve: reserve,                               // R reserve
            train_length: (byte_array[8] & 0x7) as u32,     // ZL length
            junction: (reporting_point >> 2) / 10,
            junction_number: (reporting_point >> 2) % 10,
            request_status: reporting_point & 0x3 as u32,
        })
    }
}

impl PartialEq for Telegram {
    fn eq(&self, other: &Self) -> bool {
        self.sign_of_deviation == other.sign_of_deviation
            && self.value_of_deviation == other.value_of_deviation
            && self.reporting_point == other.reporting_point
            && self.priority == other.priority
            && self.request_for_priority == other.request_for_priority
            && self.line == other.line
            && self.run_number == other.run_number
            && self.destination_number == other.destination_number
            && self.reserve == other.reserve
            && self.train_length == other.train_length
            && self.junction == other.junction
            && self.junction_number == other.junction_number
            && self.request_status == other.request_status
    }
}

impl fmt::Display for Telegram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const FILE_STR: &'static str = include_str!("../../stops.json");
        let parsed: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&FILE_STR).expect("JSON was not well-formatted");
        let junction_string = self.junction.to_string();
        // TODO: get the region_id from a config
        let region_id = "0";
        let junction = parsed
            .get(region_id)
            .unwrap()
            .as_object()
            .unwrap()
            .get(&junction_string)
            .map(|u| {
                let name = u
                    .as_object()
                    .unwrap()
                    .get("name")
                    .unwrap()
                    .as_str()
                    .unwrap();

                match name {
                    "" => &junction_string,
                    _ => name,
                }
            })
            .unwrap_or(&junction_string);

        write!(
            f,
            "Line {} Run {} Destination {} - {} / {} / {}",
            self.line,
            self.run_number,
            self.destination_number,
            junction,
            self.junction_number,
            self.request_status
        )
    }
}
