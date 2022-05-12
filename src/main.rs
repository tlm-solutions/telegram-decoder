#[macro_use]
extern crate serde_derive;

mod decoder;
mod structs;

// modules
use decoder::{Config, Decoder};
use structs::Args;

// extern creates
use clap::Parser;

// standard lib
use std::fs::read_to_string;
use std::net::UdpSocket;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread;

fn main() {
    const BUFFER_SIZE: usize = 2048;

    let args = Args::parse();
    let stop_mapping = String::from(&args.config);

    let contents = read_to_string(stop_mapping).expect("Something went wrong reading the file");

    let station_config: Config =
        serde_json::from_str(&contents).expect("JSON was not well-formatted");

    let decoder = Decoder::new(&station_config, &args.server);

    println!("Starting DVB Dump Telegram Decoder ... ");
    let addr = format!("{}:{}", &args.host, &args.port);
    let socket = UdpSocket::bind(addr).unwrap();
    let (tx, rx): (SyncSender<[u8; BUFFER_SIZE]>, Receiver<[u8; BUFFER_SIZE]>) = mpsc::sync_channel(100);

    let _thread = thread::spawn(move || loop {
        let data = rx.recv().unwrap();
        decoder.process(&data)
    });
    loop {
        let mut buffer = [0; BUFFER_SIZE];
        let (_amt, _src) = socket.recv_from(&mut buffer).unwrap();
        tx.send(buffer.clone());
    }
}
