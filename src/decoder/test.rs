//mod decoder;
//use decoder::{Decoder, Config, Telegram};
//
extern crate derive_builder;


#[derive(Deserialize, Serialize, Debug)]
struct UnitTestData {
    test: Vec<u8>
}
// Note this useful idiom: importing names from outer (for mod tests) scope.
#[test]
fn test_decoding1() {
    let config = Config {
        name: "TEST".to_string(),
        lat: 0.,
        lon: 0.,
        station_id: 0,
    };

    let server = "mockup".to_string();
    let decoder = Decoder::new(&config, &server);
    const FILE_STR: &'static str = include_str!("../../data/unit_test1.json");
    let parsed: UnitTestData =
        serde_json::from_str(&FILE_STR).expect("JSON was not well-formatted");
    
    let raw_data = parsed.test.as_ref();

    let expected_telegram = Telegram {
        time_stamp: 1651077557,
        lat: 0.0,
        lon: 0.0,
        station_id: 0,
        line: 73,
        destination_number: 365,
        priority: 0,
        sign_of_deviation: 1,
        value_of_deviation: 1,
        reporting_point: 1,
        request_for_priority: 0,
        run_number: 1,
        reserve: 0,
        train_length: 0,
        junction: 0,
        junction_number: 0,
    };
    let received_telegram = decoder.full_decodation(&raw_data);

    assert_eq!(received_telegram[0], expected_telegram);
}

#[test]
fn test_decoding2() {
    let config = Config {
        name: "TEST".to_string(),
        lat: 0.,
        lon: 0.,
        station_id: 0,
    };

    let server = "mockup".to_string();
    let decoder = Decoder::new(&config, &server);
    const FILE_STR: &'static str = include_str!("../../data/unit_test2.json");
    let parsed: UnitTestData =
        serde_json::from_str(&FILE_STR).expect("JSON was not well-formatted");
    
    let raw_data = parsed.test.as_ref();

    let expected_telegram = Telegram { 
        time_stamp: 1651077554, 
        lat: 0., 
        lon: 0., 
        station_id: 0,
        line: 103,
        destination_number: 365, 
        priority: 0, 
        sign_of_deviation: 0, 
        value_of_deviation: 1, 
        reporting_point: 62,
        request_for_priority: 0,
        run_number: 34, 
        reserve: 0, 
        train_length: 0, 
        junction: 1, 
        junction_number: 5 
    };

    let received_telegram = decoder.full_decodation(&raw_data);

    assert_eq!(received_telegram[0], expected_telegram);
}
