use chrono::{DateTime, Utc};
use single_value_channel::Updater;

pub async fn run_last_usage_time_updater(tx: Updater<DateTime<Utc>>) {
    rdev::listen(move |_| {
        tx.update(Utc::now()).unwrap();
    })
    .unwrap();
}
