use crate::{
    utils::{self, Config},
    ScreenTime,
};
use chrono::NaiveDate;
use jammdb::DB;

#[derive(Debug)]
enum ClientRequest {
    Hms(String),
    Incorrect,
}

pub fn execute(args: Vec<String>) {
    let request = match &args[..] {
        [arg, date] if *arg == String::from("hms") => ClientRequest::Hms(date.to_string()),
        _ => ClientRequest::Incorrect,
    };
    match request {
        ClientRequest::Hms(date) => {
            let date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
            draw_screentime(date);
        }
        ClientRequest::Incorrect => {
            println!("Incorrect request");
        }
    }
}

fn draw_screentime(date: NaiveDate) {
    let config = Config::get_screentime_file_path();
    let db = DB::open(config).unwrap();
    let tx = db.tx(false).unwrap();
    let bucket = tx.get_bucket("screentime").unwrap();
    let screentime = if let Some(data) = bucket.get(&date.to_string()) {
        let data: ScreenTime = rmp_serde::from_slice(data.kv().value()).unwrap();
        Some(data)
    } else {
        None
    };
    if let Some(screentime) = screentime {
        let mut dupa: Vec<_> = screentime.into_iter().collect();
        dupa.sort_by(|a, b| a.1.cmp(&b.1));
        for (name, seconds) in dupa {
            let hms = utils::seconds_to_hms(seconds);
            let dots = ".".repeat((seconds / 60) as usize);
            println!("{name}: {dots} {hms}");
        }
    } else {
        println!("No data for this date");
    }
}
