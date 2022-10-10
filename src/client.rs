use config::Config;

use crate::consts::PORT;

pub fn handle_client_mode(option: &str, config: &Config) {
    let url = format!("http://127.0.0.1:{}", config.get_int(PORT).unwrap());
    let resp = tinyget::get(url).send().unwrap();
    let seconds: u64 = resp.as_str().unwrap().parse().unwrap();
    let to_print = match option {
        "hms" => {
            let hours = seconds / 3600;
            let seconds = seconds % 3600;
            let minutes = seconds / 60;
            let seconds = seconds % 60;

            format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds)
        }
        _ => seconds.to_string(),
    };

    print!("{}", to_print);
}
