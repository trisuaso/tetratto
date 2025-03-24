use chrono::{TimeZone, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

/// Get a [`u128`] timestamp
pub fn unix_epoch_timestamp() -> u128 {
    let right_now = SystemTime::now();
    let time_since = right_now
        .duration_since(UNIX_EPOCH)
        .expect("Time travel is not allowed");

    time_since.as_millis()
}

/// Get a [`i64`] timestamp from the given `year` epoch
pub fn epoch_timestamp(year: u32) -> i64 {
    let now = Utc::now().timestamp_millis();
    let then = Utc
        .with_ymd_and_hms(year as i32, 1, 1, 0, 0, 0)
        .unwrap()
        .timestamp_millis();

    now - then
}
