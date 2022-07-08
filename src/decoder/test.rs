//mod decoder;
use crate::decoder::Decoder;
use telegrams::{R09Telegram, RadioStation};
use stop_names::R09Types;
use serde::{Deserialize};
use uuid::Uuid;

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
    r: u32,
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

macro_rules! decode_telegrams_from_file {
    ($file: expr, $decoder: expr ) => {{
        const FILE_STR: &'static str = include_str!($file);
        let parsed: Vec<ValidR09_16Telegram> =
            serde_json::from_str(&FILE_STR).expect("JSON was not well-formatted");

        for (i, item) in parsed.iter().enumerate() {
            let telegram = &item.output;

            let expected_telegram = R09Telegram {
                telegram_type: R09Types::R16,
                delay: Some((2i32 - telegram.zv as i32) * (telegram.zw as i32)),
                reporting_point: telegram.mp,
                junction: telegram.junction,
                direction: telegram.ha as u8,
                request_status: telegram.request_status as u8,
                priority: Some(telegram.pr as u8),
                direction_request: Some(telegram.junction_number as u8),
                line: Some(telegram.ln),
                run_number: Some(telegram.kn),
                destination_number: Some(telegram.zn),
                train_length: Some(telegram.zl as u8),
                vehicle_number: None,
                operator: None

                /*time_stamp: 0,
                line: format!("{:0>3}", telegram.await.ln.to_string()),
                destination_number: format!("{:0>3}", telegram.zn.to_string()),
                priority: telegram.pr,
                sign_of_deviation: telegram.zv,
                value_of_deviation: telegram.zw,
                reporting_point: telegram.mp,
                request_for_priority: telegram.ha,
                run_number: format!("{:0>2}", telegram.kn.to_string()),
                reserve: telegram.r,
                train_length: telegram.zl,
                junction: telegram.junction,
                junction_number: telegram.junction_number,
                request_status: telegram.request_status, */
            };

            let received_telegram = decoder.decode(&item.input.as_ref());

            assert_eq!(received_telegram[0], expected_telegram);

            println!("{}", received_telegram[0]);
            println!("{}/{} OK", i + 1, parsed.len());
        }
    }};
}

#[test]
fn test_decode_valid_r09_16_telegrams() {
    let config = RadioStation {
        id: Uuid::new_v4(),
        name: "test".to_string(),
        lat: 0.0,
        lon: 0.0,
        region: "dvb".to_string(),
    };
    let server = vec!["mockup".to_string()];

    let decoder = Decoder::new(&config, &server).await;
    decode_telegrams_from_file!("../../data/valid_r09_16_telegrams.json", decoder);
}

#[test]
fn test_decode_1bit_error_r09_16_telegrams() {
    let config = RadioStation {
        id: Uuid::new_v4(),
        name: "test".to_string(),
        lat: 0.0,
        lon: 0.0,
        region: "dvb".to_string(),
    };
    let server = vec!["mockup".to_string()];

    let decoder = Decoder::new(&config, &server).await;
    decode_telegrams_from_file!("../../data/1bit_error_r09_16_telegrams.json", decoder);
}
