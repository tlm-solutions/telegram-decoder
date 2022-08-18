extern crate derive_builder;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "dump-dvb telegram decode server")]
#[clap(author = "revol-xut@protonmail.com")]
#[clap(version = "0.1.0")]
#[clap(about = "Runns specified captures and extracts times.", long_about = None)]
pub struct Args {
    #[clap(short, long, default_value_t = String::from("127.0.0.1"))]
    pub host: String,

    #[clap(short, long, default_value_t = 40000)]
    pub port: u32,

    #[clap(short, long, min_values = 1)]
    pub server: Vec<String>,

    #[clap(short, long, default_value_t = String::from("config.json"))]
    pub config: String,

    #[clap(short, long, action)]
    pub offline: bool

}
