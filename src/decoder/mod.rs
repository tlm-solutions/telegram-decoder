pub mod structs;
mod test;

pub use structs::{Config, Telegram};

use crc_all::CrcAlgo;
use lazy_static::lazy_static;
use reqwest;

pub struct Decoder {
    server: String,
    station_config: Config,
}

impl Decoder {
    pub fn new(config: &Config, server: &String) -> Decoder {
        Decoder {
            station_config: config.clone(),
            server: server.clone(),
        }
    }

    pub fn process(&self, data: &[u8]) {
        let data_copy = data.clone();
        
        let response = self.full_decodation(data_copy);
        if response.len() == 0 {
            return;
        }

        let client = reqwest::Client::new();
        let url = format!("{}/formatted_telegram", &self.server);
        let rt = tokio::runtime::Runtime::new().unwrap();
        for telegram in response {
            println!("Telegram: {}", telegram);

            rt.block_on(client.post(&url).json(&telegram).send());
        }
    }

    pub fn full_decodation(&self, data:&[u8]) -> Vec<Telegram>{
        let mut byte_array: Vec<u8> = Vec::new();
        const MAXIMUM_SIZE: usize = 20;


        for i in 0..MAXIMUM_SIZE {
            if (i + 1) * 9 - 1 >= data.len() {
                break;
            }
            byte_array.push(Decoder::bit_to_bytes(&data[i * 9..(i + 1) * 9 - 1]));
        }

        self.correct_one_bit_errors(&byte_array)
    }

    // converts a list of bits into a single byte
    fn bit_to_bytes(data: &[u8]) -> u8 {
        if data.len() != 8 {
            return 0;
        } else {
            let mut byte_value: u8 = 0;

            for bit in 0..8u8 {
                if data[bit as usize] > 0 {
                    byte_value += 1 << bit;
                }
            }

            return byte_value;
        }
    }

    fn crc16_umts(data: &[u8]) -> u32 {
        lazy_static! {
            static ref CRC16_UMTS: CrcAlgo<u32> = CrcAlgo::<u32>::new(0x16f63, 16, 0x0, 0x0, true);
        }

        let crc = &mut 0u32;
        CRC16_UMTS.init_crc(crc);
        CRC16_UMTS.update_crc(crc, data)
    }

    fn decode(&self, byte_array: &Vec<u8>) -> Option<Telegram> {
        if byte_array.len() >= 5 {
            let crc_function = |posistion: usize| -> u32 {
                (((byte_array[posistion + 1] as u32) << 8u32 | byte_array[posistion] as u32)
                    ^ 0xffffu32) as u32
            };
            let mut length: usize = 0;

            // there are multiple types of packages floating around so we take
            // the telegram which has the correct looking crc code
            for position in 4..(byte_array.len() - 1) {
                if crc_function(position) == Decoder::crc16_umts(&byte_array[0..position]) {
                    length = position;
                }
            }

            // no valid telegram was found aborting
            if length == 0 {
                return None;
            }

            // packages have different modes in the documentation refered to r1 - 15
            let mode: u8 = byte_array[0] >> 4;

            // lower nibble of the mode
            let r09_type = byte_array[0] & 0xf;

            if mode == 9 && r09_type == 1 {
                length = (byte_array[1] & 0xf) as usize;
                if byte_array.len() < 5usize + length {
                    return None;
                }

                if crc_function(9) != Decoder::crc16_umts(&byte_array[0..9]) {
                    return None;
                }

                // we have 3 bytes for error correction so the first 6 bytes contain the data
                if length == 6 {
                    return Some(Telegram::parse(byte_array, &self.station_config));
                }
                return None;
            } else {
                return None;
            }
        }
        return None;
    }

    fn correct_one_bit_errors(&self, payload: &Vec<u8>) -> Vec<Telegram> {
        let mut collection: Vec<Telegram> = Vec::new();

        let result = self.decode(payload);

        if result.is_some() {
            collection.push(result.unwrap());
        } else {
            for i in 0..payload.len() {
                for j in 0..8 {
                    let mut tmp = payload.clone();
                    tmp[i] ^= 1 << j;
                    match self.decode(&tmp) {
                        Some(telegram) => {
                            collection.push(telegram);
                        }
                        None => {}
                    };
                }
            }
        }

        return collection;
    }
}
