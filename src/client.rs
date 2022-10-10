use std::io::{stdout, Write};

use config::Config;

use crate::consts::PORT;

pub fn handle_client_mode(option: &str, config: &Config) {
    let url = format!(
        "http://127.0.0.1:{}/{}",
        config.get_int(PORT).unwrap(),
        option
    );
    let resp = tinyget::get(url).send().unwrap();

    stdout()
        .write_all(resp.as_bytes())
        .expect("could not write to stdout");
}
