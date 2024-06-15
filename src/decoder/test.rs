use crate::decoder::Decoder;
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::mpsc;
use std::sync::mpsc::SyncSender;
use tlms::telegrams::r09::{R09Telegram, R09Type};

extern crate derive_builder;

#[derive(Deserialize)]
struct TelegramFrame {
    zv: u32,
    zw: u32,
    mp: u32,
    pr: u32,
    ha: u32,
    ln: u32,
    kn: u32,
    zn: u32,
    zl: u32,
    junction: u32,
    junction_number: u32,
    request_status: u32,
}

#[derive(Deserialize)]
struct ValidR09_16Telegram {
    input: Vec<u8>,
    output: TelegramFrame,
}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

macro_rules! decode_telegrams_from_file {
    ($file: expr, $disable_error_correction: expr) => {{
        let (sender_r09, receiver_r09) = mpsc::sync_channel(1);
        let mut senders_r09: Vec<SyncSender<R09Telegram>> = Vec::new();
        senders_r09.push(sender_r09);

        let mut decoder = Decoder::new(&senders_r09, &Vec::new(), $disable_error_correction);

        const FILE_STR: &'static str = include_str!($file);

        let parsed: Vec<ValidR09_16Telegram> =
            serde_json::from_str(&FILE_STR).expect("JSON was not well-formatted");

        for (i, item) in parsed.iter().enumerate() {
            let telegram = &item.output;

            let expected_telegram = R09Telegram {
                r09_type: R09Type::R16,
                delay: Some((telegram.zv as i32 * -2i32 + 1i32) * (telegram.zw as i32)),
                reporting_point: telegram.mp,
                junction: telegram.junction,
                direction: telegram.junction_number as u8,
                request_status: telegram.request_status as u8,
                priority: Some(telegram.pr as u8),
                direction_request: Some(telegram.ha as u8),
                line: Some(telegram.ln),
                run_number: Some(telegram.kn),
                destination_number: Some(telegram.zn),
                train_length: Some(telegram.zl as i32),
                vehicle_number: None,
                operator: None,
            };

            decoder.process(&item.input.as_ref());
            let received_telegram = receiver_r09.recv().unwrap();

            assert_eq!(
                calculate_hash(&received_telegram),
                calculate_hash(&expected_telegram)
            );

            println!("{}/{} OK", i + 1, parsed.len());
        }
    }};
}

#[test]
fn test_decode_valid_r09_16_telegrams() {
    decode_telegrams_from_file!("../../data/valid_r09_16_telegrams.json", true);
}

#[test]
fn test_decode_1bit_error_r09_16_telegrams() {
    decode_telegrams_from_file!("../../data/1bit_error_r09_16_telegrams.json", false);
}
