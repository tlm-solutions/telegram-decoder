#[macro_use]
extern crate serde_derive;

mod structs;

use clap::Parser;
use crc_all::CrcAlgo;
use lazy_static::lazy_static;
use reqwest;
use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::copy;
use std::io::stdout;
use std::net::UdpSocket;
use std::time::{SystemTime, UNIX_EPOCH};
use structs::{Args, Telegram};

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

fn decode(byte_array: &Vec<u8>) -> Option<Telegram> {
    println!("{:?}", byte_array);
    if byte_array.len() >= 5 {
        // 3 + 2 - 1

        let crc_function = |posistion: usize| -> u32 {
            (((byte_array[posistion + 1] as u32) << 8u32 | byte_array[posistion] as u32)
                ^ 0xffffu32) as u32
        };
        let mut length: usize = 0;

        // there are multiple types of packages floating around so we take
        // the telegram which has the correct looking crc code
        println!("Size: {}", byte_array.len());
        for position in 4..(byte_array.len() - 1) {
            println!(
                "crc: {} {} {}",
                position,
                crc_function(position),
                crc16_umts(&byte_array[0..position])
            );
            if crc_function(position) == crc16_umts(&byte_array[0..position]) {
                length = position;
            }
        }

        println!("Length: {}", length);

        // no valid telegram was found aborting
        if length == 0 {
            println!("Package Decodation Failed !");
            return None;
        }

        // packages have different modes in the documentation refered to r1 - 15
        let mode: u8 = byte_array[0] >> 4;

        // lower nibble of the mode
        let r09_type = byte_array[0] & 0xf;

        println!("Package Mode: {}", mode);

        if mode == 9 && r09_type == 1 {
            length = (byte_array[1] & 0xf) as usize;

            if byte_array.len() < 5usize + length {
                return None;
            }

            if crc_function(9) != crc16_umts(&byte_array[0..9]) {
                return None;
            }

            // we have 3 bytes for error correction so the first 6 bytes contain the data
            if length == 6 {
                return Some(Telegram::parse(byte_array));
            }
            return None;
        } else if mode == 9 && length > 4 {
            return None;
        } else {
            return None;
        }
    }
    return None;
}

fn one_bit_error_list(payload: &Vec<u8>) -> Vec<Telegram> {
    let mut collection: Vec<Telegram> = Vec::new();

    let result = decode(payload);

    if result.is_some() {
        collection.push(result.unwrap());
    } else {
        for i in 0..payload.len() {
            for j in 0..8 {
                let mut tmp = payload.clone();
                tmp[i] ^= 1 << j;
                match decode(&tmp) {
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

fn main() {
    const buffer_size: usize = 4096;
    let maximum_size: usize = 20;

    let args = Args::parse();
    //let stop_mapping = String::from(&args.config);

    // let contents = read_to_string(stop_mapping)
    //    .expect("Something went wrong reading the file");

    //let json: HashMap<u32, String> =
    //    serde_json::from_str(&contents).expect("JSON was not well-formatted");

    //decode(&vec![100, 130, 193, 19, 245, 243, 255, 127, 195, 156, 14, 198, 129, 230, 192, 122, 188, 0, 0, 0, 0]);
    //return;

    println!("Starting DVB Dump Telegram Decoder ... ");
    let addr = format!("{}:{}", &args.host, &args.port);
    let socket = UdpSocket::bind(addr).unwrap();

    loop {
        let mut buffer = [0; buffer_size];
        println!("Receiving stuff");
        let (amt, src) = socket.recv_from(&mut buffer).unwrap();
        let mut byte_array: Vec<u8> = vec![];
        println!("Received Data!");
        for i in 0..maximum_size {
            byte_array.push(bit_to_bytes(&buffer[i * 9..(i + 1) * 9 - 1]));
        }

        let response = decode(&byte_array);
        let url = "http://academicstrokes.com/formatted_telegram";
        for telegram in response {
            let client = reqwest::Client::new();
            client.post(url).json(&telegram).send();
        }
    }
}
