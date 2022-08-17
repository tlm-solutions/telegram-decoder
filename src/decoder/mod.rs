pub mod structs;
#[cfg(test)]
mod test;

pub use structs::{parse_r09_telegram};

use dump_dvb::{
    receivers::RadioReceiver,
    telegrams::{
        AuthenticationMeta, 
        r09::{
            R09ReceiveTelegram, 
            R09Telegram,
        },
    },
};

use g2poly::G2Poly;
use reqwest;
use std::collections::HashMap;
use std::env;
use std::time::Duration;

pub struct Decoder {
    server: Vec<String>,
    station_config: RadioReceiver,
    maps: Vec<HashMap<u64, Vec<u8>>>,
    token: String,
}

impl Decoder {
    pub async fn new(config: &RadioReceiver, server: &Vec<String>) -> Decoder {
        let mut maps: Vec<HashMap<u64, Vec<u8>>> = Vec::new();

        for len in 5..22 {
            let mut map: HashMap<u64, Vec<u8>> = HashMap::new();

            // 1 bit errors
            for i in 0..(len * 8) {
                let mut data = vec![0u8; len];
                let idx = (i / 8) as usize;
                let pos = i % 8;
                data[idx] ^= 1 << pos;

                let value: u64 = Decoder::crc16_remainder(&data).await.0;

                if let Some(_) = map.get(&value) {
                    assert!(false);
                } else {
                    map.insert(value, data);
                }
            }

            // 2 bit errors
            for i in 0..(len * 8) {
                for j in (i + 1)..(len * 8) {
                    let mut data = vec![0u8; len];
                    let idx = (i / 8) as usize;
                    let pos = i % 8;
                    data[idx] ^= 1 << pos;
                    let idx = (j / 8) as usize;
                    let pos = j % 8;
                    data[idx] ^= 1 << pos;

                    let value: u64 = Decoder::crc16_remainder(&data).await.0;

                    if let Some(_) = map.get(&value) {
                        assert!(false);
                    } else {
                        map.insert(value, data);
                    }
                }
            }

            // 3 bit errors
            // this might work sometimes...
            // so the algorithm for creating the map is a bit more complicated
            if len < 14 {
                let mut blacklist: Vec<u64> = Vec::new();
                let mut map_3bit: HashMap<u64, Vec<u8>> = HashMap::new();

                for i in 0..(len * 8) {
                    for j in (i + 1)..(len * 8) {
                        for k in (j + 1)..(len * 8) {
                            let mut data = vec![0u8; len];
                            let idx = (i / 8) as usize;
                            let pos = i % 8;
                            data[idx] ^= 1 << pos;
                            let idx = (j / 8) as usize;
                            let pos = j % 8;
                            data[idx] ^= 1 << pos;
                            let idx = (k / 8) as usize;
                            let pos = k % 8;
                            data[idx] ^= 1 << pos;

                            let value: u64 = Decoder::crc16_remainder(&data).await.0;

                            // only try to add it, if the value is not allready corrected by 1 or 2 bit
                            // error correction
                            if None == map.get(&value) {
                                if let Some(_) = map_3bit.get(&value) {
                                    // throw out the value if it occurs multiple times (hamming
                                    // distance too low)
                                    map_3bit.remove(&value);
                                    blacklist.push(value);
                                } else {
                                    // add the value if it is not in the blacklist (hamming distance
                                    // too low)
                                    if blacklist.iter().all(|&v| v != value) {
                                        map_3bit.insert(value, data);
                                    }
                                }
                            }
                        }
                    }
                }

                // extend the map with 3 bit error correction values
                map.extend(map_3bit);
            }

            maps.push(map)
        }

        let token: String = env::var("AUTHENTICATION_TOKEN_PATH")
            .map(|token_path| {
                String::from_utf8_lossy(&std::fs::read(token_path).unwrap())
                    .parse::<String>()
                    .unwrap()
            })
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .to_string();

        Decoder {
            station_config: config.clone(),
            server: server.clone(),
            maps: maps,
            token: token,
        }
    }

    pub async fn process(&self, data: &[u8]) {
        let data_copy = data.clone();

        let response = self.decode(data_copy).await;
        if response.len() == 0 {
            return;
        }

        let auth = AuthenticationMeta {
            station: self.station_config.id.clone(),
            token: self.token.clone(),
        };

        let client = reqwest::Client::new();
        for telegram in response {
            for server in &self.server {
                let url = format!("{}/telegram/r09", &server);
                match client
                    .post(&url)
                    .timeout(Duration::new(2, 0))
                    .json(&R09ReceiveTelegram {
                        auth: auth.clone(),
                        data: telegram.clone(),
                    })
                    .send()
                    .await
                {
                    Err(_) => {
                        println!("Connection Timeout! {}", &server);
                    }
                    _ => {}
                }
            }
        }
    }

    pub async fn decode(&self, data: &[u8]) -> Vec<R09Telegram> {
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
            byte_array.push(Decoder::bit_to_bytes(&data[i * 9..(i + 1) * 9 - 1]).await);
        }

        let mut collection: Vec<R09Telegram> = Vec::new();

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

            let rem = Decoder::crc16_remainder(&telegram_array).await;

            let mut telegrams: Vec<Vec<u8>> = Vec::new();

            if rem == G2Poly(0) {
                // no errors, decode
                telegrams.push((&telegram_array[0..(telegram_length - 2)]).to_vec())
            } else {
                // errors. try to fix them
                if let Some(error) = self.maps[telegram_length - MINIMUM_SIZE].get(&rem.0) {
                    assert_eq!(error.len(), telegram_length);

                    let mut repaired_telegram = telegram_array.clone();
                    for i in 0..error.len() {
                        repaired_telegram[i] ^= error[i];
                    }

                    assert_eq!(
                        Decoder::crc16_remainder(&repaired_telegram).await,
                        G2Poly(0)
                    );

                    telegrams.push((&repaired_telegram[0..(telegram_length - 2)]).to_vec())
                }
            }

            for telegram in telegrams {
                match Decoder::parse_telegram(&telegram).await {
                    Some(telegram) => {
                        println!("Decoder R09 Telegram: {:?}", telegram);
                        collection.push(telegram);
                    }
                    None => {}
                };
            }
        }

        return collection;
    }

    // data is a vector of data without crc
    // TODO: change this into a vector. There is the possibilty for a valid R09 telegram being a
    // valid C09 telegram and vice versa.
    async fn parse_telegram(data: &[u8]) -> Option<R09Telegram> {
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
                return Decoder::parse_r09(&data).await;
            }
            // parse C09.x
            if 4 + (data[2] & 0xf) as usize == length {
                let c09_type = data[2] >> 4;
                let c09_length = data[2] & 0xf;

                // TODO
                println!("[!] Recevied C09.{}.{}", c09_type, c09_length);

                return None;
            }

            return None;
        }

        // We removed the one variable length telegrams of the R-series R09, others are 3 bytes
        // long.
        // The C-series has C09, which is variable length, but we don't know anything about C05-C08.
        // They are probably only 4 bytes long, like other ones from the C-series, but we don't
        // know.
        match length {
            3 => println!("[!] Received R {}", mode),
            _ => println!("[!] Received C {}", mode),
        };

        return None;
    }

    async fn parse_r09(data: &[u8]) -> Option<R09Telegram> {
        // lower nibble of the mode
        let r09_type = data[0] & 0xf;
        let r09_length = data[1] & 0xf;

        assert_eq!(3 + r09_length as usize, data.len());

        // decode R09.1x
        if r09_type == 1 && r09_length == 6 {
            // TODO: if BCD is not BCD, throw it out
            return parse_r09_telegram(data);
        } else {
            println!("[!] Recevied R09.{}.{}", r09_type, r09_length);
        }

        return None;
    }

    // converts a list of bits into a single byte
    async fn bit_to_bytes(data: &[u8]) -> u8 {
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

    async fn crc16_remainder(data: &Vec<u8>) -> G2Poly {
        const POLY: G2Poly = G2Poly(0x16f63);
        const ALPHA: G2Poly = G2Poly(0b10);

        let mut rem = G2Poly(0);

        for i in 0..data.len() {
            for j in 0..8 {
                let offset: u64 = (i * 8).try_into().unwrap();
                if 1 == (data[data.len() - i - 1] >> (7 - j)) & 0x1 {
                    rem = (rem + ALPHA.pow_mod(j + offset, POLY)) % POLY;
                }
            }
        }

        rem
    }
}
