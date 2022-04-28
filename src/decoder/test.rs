//mod decoder;
use crate::decoder::{Decoder, Config, Telegram};

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
    request_status: u32
}

#[derive(Deserialize)]
struct ValidR09_16Telegram {
    input: Vec<u8>,
    output: TelegramFrame
}

// Note this useful idiom: importing names from outer (for mod tests) scope.
#[test]
fn test_decode_valid_r09_16_telegrams() {
    let config = Config {
        name: "TEST".to_string(),
        lat: 0.,
        lon: 0.,
        station_id: 0,
    };

    let server = "mockup".to_string();
    let decoder = Decoder::new(&config, &server);
    const FILE_STR: &'static str = include_str!("../../data/valid_r09_16_telegrams.json");
    let parsed : Vec<ValidR09_16Telegram> = serde_json::from_str(&FILE_STR).expect("JSON was not well-formatted");

    for (i, item) in parsed.iter().enumerate() {
        let telegram = &item.output;

        let expected_telegram = Telegram {
            time_stamp: 0,
            lat: 0.0,
            lon: 0.0,
            station_id: 0,
            line: telegram.ln,
            destination_number: telegram.zn,
            priority: telegram.pr,
            sign_of_deviation: telegram.zv,
            value_of_deviation: telegram.zw,
            reporting_point: telegram.mp,
            request_for_priority: telegram.ha,
            run_number: telegram.kn,
            reserve: telegram.r,
            train_length: telegram.zl,
            junction: telegram.junction,
            junction_number: telegram.junction_number,
            request_status: telegram.request_status,
        };

        let received_telegram = decoder.full_decodation(&item.input.as_ref());
        
        assert_eq!(received_telegram[0], expected_telegram);

        println!("{}/{} OK", i + 1, parsed.len());
    }
}
