pub mod structs;
mod test;

pub use structs::{Config, Telegram};

use g2poly::G2Poly;
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
        
        let response = self.decode(data_copy);
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

    pub fn decode(&self, data:&[u8]) -> Vec<Telegram>{
		// minimum message size is 3 bytes + 2 bytes crc
		const MINIMUM_SIZE: usize = 5;
        // C09 fixed size of 4 bytes + variable length 4 bits (15 bytes) + 2 bytes CRC
        // max length 4 + 15 + 2 = 21
        const MAXIMUM_SIZE: usize = 21;

        let mut byte_array: Vec<u8> = Vec::new();

        for i in 0..MAXIMUM_SIZE {
            if (i + 1) * 9 - 1 >= data.len() {
                break;
            }
            byte_array.push(Decoder::bit_to_bytes(&data[i * 9..(i + 1) * 9 - 1]));
        }

        let mut collection: Vec<Telegram> = Vec::new();

		// Abort if we don't have enough data for a packet
		if byte_array.len() < MINIMUM_SIZE {
			return collection;
		}

		// try decoding for every possible length
        for telegram_length in MINIMUM_SIZE..(byte_array.len() - 1) {
            let mut telegram_array = Vec::new();
            telegram_array.extend_from_slice(&byte_array[0..telegram_length]);

			// invert crc
			telegram_array[telegram_length - 2] ^= 0xff;
			telegram_array[telegram_length - 1] ^= 0xff;

			let rem = Decoder::crc16_remainder(&telegram_array);

			if rem == G2Poly(0) {
				// no errors, decode
				match self.parse_telegram(&telegram_array[0..(telegram_length - 2)]) {
					Some(telegram) => {
						collection.push(telegram);
					}
					None => {}
				};
			} else {
				// errors. try to fix them
			}
        }

        return collection;
    }

	// data is a vector of data without crc
    // TODO: change this into a vector. There is the possibilty for a valid R09 telegram being a
    // valid C09 telegram and vice versa.
    fn parse_telegram(&self, data: &[u8]) -> Option<Telegram> {
        let mode: u8 = data[0] >> 4;
        let length: usize = data.len();

        // length has to be at least 3 bytes
        if length < 3 {
            return None;
        }

        // these modes may only have length 3 (R telegrams) or 4 (C telegrams)
        // lower bound is already checked above
        if mode <= 4 || mode >= 10 {
            if length > 4 {
                return None;
            }
        }

        if mode == 9 {
            // parse R09.x
            if 3 + (data[1] & 0xf) as usize == length {
                return self.parse_r09(&data);
            }
            // parse C09.x
            if 4 + (data[2] & 0xf) as usize == length {
                // TODO
                return None;
            }
        }

        // TODO: parse every other mode

		return None;
	}

    fn parse_r09(&self, data: &[u8]) -> Option<Telegram> {
        // lower nibble of the mode
        let r09_type = data[0] & 0xf;

        // decode R09.1x
        if r09_type == 1 {
            // TODO: if BCD is not BCD, throw it out
            return Some(Telegram::parse(data, &self.station_config));
        }

        return None;
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

    fn crc16_remainder(data: &Vec<u8>) -> G2Poly {
        const POLY: G2Poly = G2Poly(0x16f63);
        const ALPHA: G2Poly = G2Poly(0b10);

        let mut rem = G2Poly(0);

        for i in 0..data.len() {
            for j in 0..8 {
                let offset : u64 = (i * 8).try_into().unwrap();
                if 1 == (data[data.len() - i - 1] >> (7-j)) & 0x1 {
                    rem = (rem + ALPHA.pow_mod(j+offset, POLY)) % POLY;
                }
            }
        }

        rem
    }
}
