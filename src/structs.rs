extern crate derive_builder;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "telegram-decoder")]
#[clap(author = "hello@tlm.solutions")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "Extracts telegram from raw bit streams and sends them.", long_about = None)]
pub struct Args {
    #[clap(short, long, default_value_t = String::from("127.0.0.1"))]
    pub host: String,

    #[clap(short, long, default_value_t = 40000)]
    pub port: u32,

    #[clap(short, long)]
    pub server: Vec<String>,

    #[clap(short, long, default_value_t = String::from("config.json"))]
    pub config: String,

    #[clap(short, long, action)]
    pub offline: bool,

    #[clap(short, long, action)]
    pub disable_error_correction: bool,

    #[clap(short, long, action)]
    pub verbose: bool,
}
