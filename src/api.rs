use jammdb::DB;

use crate::{utils};

use crate::{utils::Config, ScreenTime};

fn get_today_screentime() -> Option<ScreenTime> {
    let config_file = Config::get_screentime_file_path();
    let dt = utils::get_today_as_str();
    let db = DB::open(config_file).unwrap();
    let tx = db.tx(false).unwrap();

    if let Ok(bucket) = tx.get_bucket("screentime") {
        if let Some(data) = bucket.get(dt) {
            let screentime: ScreenTime = rmp_serde::from_slice(data.kv().value()).unwrap();
            Some(screentime)
        } else {
            None
        }
    } else {
        None
    }
}

pub async fn get_inlinehms() -> String {
    let screentime = get_today_screentime();
    if let Some(screentime) = screentime {
        let mut result: Vec<String> = screentime
            .iter()
            .map(|(k, v)| format!("{}: {}", k, utils::seconds_to_hms(*v)))
            .collect();
        result.sort();
        let total = format!(
            "total: {}",
            utils::seconds_to_hms(screentime.values().sum::<u32>())
        );
        format!("{} {}", total, result.join(" "))
    } else {
        "no data".to_string()
    }
}

pub async fn get_json_secs() -> String {
    let screentime = get_today_screentime();
    serde_json::to_string(&screentime).unwrap()
}
