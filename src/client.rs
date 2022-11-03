use std::io::{stdout, Write};

use clap::ValueEnum;
use config::Config;

use crate::{consts::PORT, Command, ScreentimeConfig};

pub fn handle_client_mode(command: Command, config: &ScreentimeConfig) {
    let command = command
        .to_possible_value()
        .expect("it've been already parsed, so it won't error")
        .get_name()
        .to_string()
        .replace("-", "_");

    let url = format!(
        "http://127.0.0.1:{}/{}",
        config.config.get_int(PORT).unwrap(),
        command,
    );
    let resp = tinyget::get(url).send().unwrap();

    stdout()
        .write_all(resp.as_bytes())
        .expect("could not write to stdout");
}
