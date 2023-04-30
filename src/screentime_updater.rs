use chrono::{DateTime, Duration, Utc};
use jammdb::DB;
use single_value_channel::Receiver;

use crate::{
    utils::{self, Config},
    ScreenTime,
};

pub async fn run_screentime_updater(mut rx: Receiver<DateTime<Utc>>) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        update_screentime(&mut rx);
    }
}

fn update_screentime(rx: &mut Receiver<DateTime<Utc>>) {
    let title = utils::get_focused_program_name();

    let config_file = Config::get_screentime_file_path();

    let now = Utc::now();
    if now - *rx.latest() > Duration::seconds(30) {
        return;
    }

    let db = DB::open(config_file).unwrap();

    {
        let tx = db.tx(true).unwrap();
        tx.get_or_create_bucket("screentime").unwrap();
        tx.commit().unwrap();
    }

    let dt = utils::get_today_as_str();
    let mut screentime: ScreenTime;
    {
        let tx = db.tx(false).unwrap();
        let bucket = tx.get_bucket("screentime").unwrap();
        if let Some(data) = bucket.get(&dt) {
            screentime = rmp_serde::from_slice(data.kv().value()).unwrap();
            *screentime.entry(title).or_insert(0) += 1;
        } else {
            screentime = ScreenTime::new();
            *screentime.entry(title).or_insert(0) += 1;
        }
    }
    let tx = db.tx(true).unwrap();
    let bucket = tx.get_or_create_bucket("screentime").unwrap();
    bucket
        .put(dt, rmp_serde::to_vec(&screentime).unwrap())
        .unwrap();
    tx.commit().unwrap();
}
