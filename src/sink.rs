use dump_dvb::{
    telegrams::r09::{R09Telegram, R09ReceiveTelegram}, 
    telegrams::raw::{RawTelegram, RawReceiveTelegram},
    receivers::RadioReceiver, 
    telegrams::AuthenticationMeta
};
use std::time::Duration;
use std::sync::mpsc::Receiver;
use log::{warn, error};
use std::env;

use chrono::Utc;
use reqwest::blocking::Client;

#[derive(Clone, Debug)]
pub struct DataSinkConfig {
    pub token: String,
    pub host: String,
    pub station: RadioReceiver
}

impl DataSinkConfig {
    pub fn new(offline: bool, host: &String, station: RadioReceiver) -> DataSinkConfig {
        let token: String;
        if offline {
            token = String::from("");
        } else {
            token = env::var("AUTHENTICATION_TOKEN_PATH")
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

        }
        DataSinkConfig {
            token: token,
            host: host.clone(),
            station: station
        }
    }
}

pub fn send_r09(r09_receiver: &mut Receiver<R09Telegram>, sink: &DataSinkConfig) {
    let client = Client::new();

    loop {
        let wrapped_telegram = r09_receiver.recv();
        match  wrapped_telegram {
            Ok(telegram) => {
                let auth = AuthenticationMeta {
                    station: sink.station.id.clone(),
                    token: sink.token.clone(),
                    time: Utc::now().naive_utc()
                };

                let url = format!("{}/telegram/r09", &sink.host);
                match client
                    .post(&url)
                    .timeout(Duration::from_millis(750))
                    .json(&R09ReceiveTelegram {
                        auth: auth.clone(),
                        data: telegram.clone(),
                    })
                    .send()
                {
                    Err(_) => {
                        warn!("Connection Timeout! {}", &sink.host);
                    }
                    _ => {}
                }
            }
            Err(e) => {
                error!("received following error from r09 pipeline {:?}", e);
            }
        }
    }
}

pub fn send_raw(raw_receiver: &mut Receiver<RawTelegram>, sink: &DataSinkConfig) {
    let client = Client::new();

    loop {
        let wrapped_telegram = raw_receiver.recv();
        match wrapped_telegram {
            Ok(telegram) => {
                let auth = AuthenticationMeta {
                    station: sink.station.id.clone(),
                    token: sink.token.clone(),
                    time: Utc::now().naive_utc()
                };

                let url = format!("{}/telegram/raw", &sink.host);
                match client
                    .post(&url)
                    .timeout(Duration::from_millis(750))
                    .json(&RawReceiveTelegram {
                        auth: auth.clone(),
                        data: telegram.clone(),
                    })
                    .send()
                {
                    Err(_) => {
                        warn!("Connection Timeout! {}", &sink.host);
                    }
                    _ => {}
                }
            }
            Err(e) => {
                error!("received following error from raw pipeline {:?}", e);
            }
        }
    }
}
