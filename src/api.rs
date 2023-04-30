use jammdb::DB;

use crate::utils;

use crate::{utils::Config, ScreenTime};

pub async fn get_inlinehms() -> String {
    let config_file = Config::get_screentime_file_path();
    let dt = utils::get_today_as_str();
    let db = DB::open(config_file).unwrap();
    let tx = db.tx(false).unwrap();

    if let Ok(bucket) = tx.get_bucket("screentime") {
        if let Some(data) = bucket.get(dt) {
            let screentime: ScreenTime = rmp_serde::from_slice(data.kv().value()).unwrap();
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
            "".to_string()
        }
    } else {
        "no data".to_string()
    }
}
