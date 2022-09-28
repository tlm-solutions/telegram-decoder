extern crate serde_derive;

mod decoder;
mod structs;
mod sink;

// modules
use decoder::Decoder;
use structs::Args;
use sink::{DataSinkConfig, send_r09, send_raw};

// dump-dvb crate
use dump_dvb::{
    receivers::RadioReceiver,
    telegrams::{
        r09::{
            R09Telegram,
        },
        raw::{
            RawTelegram,
        }
    },
};

// extern creates
use clap::Parser;
use env_logger;
use log::info;

// standard lib
use std::thread;
use std::fs::read_to_string;
use std::net::UdpSocket;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};

fn main() {
    env_logger::init();

    const BUFFER_SIZE: usize = 2048;
    let args = Args::parse();

    let stop_mapping = String::from(&args.config);
    let contents = read_to_string(stop_mapping).expect("Something went wrong reading the file");

    let station_config: RadioReceiver =
        serde_json::from_str(&contents).expect("JSON was not well-formatted");
    
    let mut senders_r09: Vec<SyncSender<R09Telegram>> = Vec::new(); 

    let mut senders_raw: Vec<SyncSender<RawTelegram>> = Vec::new(); 

    for server in &args.server {
        let r09_sink_config = DataSinkConfig::new(args.offline, &server, station_config.clone());
        let raw_sink_config = r09_sink_config.clone();

        let (sender_r09, mut receiver_r09) = mpsc::sync_channel(50);
        senders_r09.push(sender_r09);

        let _thread_send_r09 = thread::spawn(move || {
            send_r09(&mut receiver_r09, &r09_sink_config.clone());
        });

        let (sender_raw, mut receiver_raw) = mpsc::sync_channel(50);
        senders_raw.push(sender_raw);

        let _thread_send_raw = thread::spawn(move || {
            send_raw(&mut receiver_raw, &raw_sink_config.clone());
        });
    }

    let mut decoder = Decoder::new(&senders_r09, &senders_raw, args.disable_error_correction);

    info!("Starting DVB Dump Telegram Decoder ... ");
    if args.disable_error_correction {
        info!("Error correction is disabled ...");
    }

    let addr = format!("{}:{}", &args.host, &args.port);
    let socket = UdpSocket::bind(addr).unwrap();
    let (tx, rx): (SyncSender<[u8; BUFFER_SIZE]>, Receiver<[u8; BUFFER_SIZE]>) =
        mpsc::sync_channel(400);

    let _thread_decode = thread::spawn(move || {
        loop {
            let data = rx.recv().unwrap();
            decoder.process(&data);
        }
    });

    loop {
        let mut buffer = [0; BUFFER_SIZE];
        let (_amt, _src) = socket.recv_from(&mut buffer).unwrap();
        tx.send(buffer.clone()).unwrap();
    }
}
